//! Admission queue entry: registration, refresh, observation, and resource admission.

use super::{
    configured_gpu_device_index, free_mib_for_device, now_epoch_ms, preemption_relevant,
    pressure_for, pressure_reason_codes, prune_expired, snapshot_materially_changed,
    AdmissionQueue, CoordinatorState, ResourceCoordinator,
};

use crate::domain::resource_adapter_registry::ResourceAdapterRegistry;
use dashmap::DashMap;
use oclive_kernel_contracts::{ResourceAdapterController, ResourceSnapshotSource};
use oclive_kernel_types::{
    AppError, ResourceAdapterDescriptor, ResourceAdapterOperation, ResourceAdapterRegistration,
    ResourceAdapterRegistrationSource, ResourceAdapterTransitionRequest, ResourceAdmissionDecision,
    ResourceAdmissionMode, ResourceAdmissionRequest, ResourceAdmissionResult, ResourceControlMode,
    ResourceCoordinationDiagnostics, ResourceCoordinatorPolicy, ResourceLeaseDiagnostic,
    ResourceLeaseState, ResourcePreemptionRecord, ResourcePressureLevel, ResourcePriority,
    ResourceSnapshot,
};
use parking_lot::{Mutex, RwLock};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

impl ResourceCoordinator {
    #[must_use]
    pub fn new(
        policy: ResourceCoordinatorPolicy,
        snapshot_source: Arc<dyn ResourceSnapshotSource>,
    ) -> Self {
        let admission_queue = AdmissionQueue::new(policy.queue_aging_quantum_ms);
        Self {
            policy,
            snapshot_source,
            state: Mutex::new(CoordinatorState::default()),
            adapter_registry: ResourceAdapterRegistry::new(),
            adapter_controllers: RwLock::new(BTreeMap::new()),
            transition_grants: RwLock::new(BTreeMap::new()),
            state_revision: AtomicU64::new(1),
            next_lease_id: AtomicU64::new(1),
            adapter_operation_locks: DashMap::new(),
            admission_queue,
        }
    }

    #[must_use]
    pub fn policy(&self) -> &ResourceCoordinatorPolicy {
        &self.policy
    }

    /// Register a resource-sensitive runtime or activity observer.
    ///
    /// # Errors
    ///
    /// Returns a validation error for malformed or conflicting descriptors.
    pub fn register_adapter(&self, descriptor: ResourceAdapterDescriptor) -> Result<(), String> {
        self.adapter_registry.register(descriptor)?;
        self.bump_revision();
        Ok(())
    }

    /// Register a descriptor owned by a third-party host extension or
    /// directory-plugin bridge.
    ///
    /// Ownership is namespace-scoped and remains visible in diagnostics.
    /// This does not grant lifecycle access or install executable code.
    ///
    /// # Errors
    ///
    /// Rejects builtin claims, namespace impersonation, malformed descriptors,
    /// and conflicting re-registration.
    pub fn register_third_party_adapter(
        &self,
        registration: ResourceAdapterRegistration,
    ) -> Result<(), String> {
        if registration.source == ResourceAdapterRegistrationSource::Builtin {
            return Err("third-party resource adapter source must not be builtin".into());
        }
        self.adapter_registry.register_owned(registration)?;
        self.bump_revision();
        Ok(())
    }

    /// Bind a third-party managed descriptor to its owner-provided controller.
    ///
    /// # Errors
    ///
    /// Rejects controllers that are not owned by `source_id` before applying
    /// the same single-writer checks as builtin controllers.
    pub fn register_third_party_adapter_controller(
        &self,
        source_id: &str,
        controller: Arc<dyn ResourceAdapterController>,
    ) -> Result<(), String> {
        let adapter_id = controller.adapter_id();
        let Some((source, owner_id)) = self.adapter_registry.registration_owner(adapter_id) else {
            return Err(format!(
                "resource adapter controller target {adapter_id} is unregistered"
            ));
        };
        if source == ResourceAdapterRegistrationSource::Builtin || owner_id != source_id {
            return Err(format!(
                "resource adapter controller {adapter_id} is not owned by {source_id}"
            ));
        }
        self.register_adapter_controller(controller)
    }

    /// Register the single authoritative lifecycle controller for an adapter.
    ///
    /// # Errors
    ///
    /// Rejects controllers without a matching managed descriptor or duplicate
    /// control ownership.
    pub fn register_adapter_controller(
        &self,
        controller: Arc<dyn ResourceAdapterController>,
    ) -> Result<(), String> {
        let adapter_id = controller.adapter_id().trim();
        if adapter_id.is_empty() || adapter_id != controller.adapter_id() {
            return Err("resource adapter controller id is invalid".into());
        }
        let descriptor = self
            .adapter_registry
            .descriptor(adapter_id)
            .ok_or_else(|| {
                format!("resource adapter controller target {adapter_id} is unregistered")
            })?;
        if descriptor.control_mode != ResourceControlMode::Managed {
            return Err(format!(
                "resource adapter controller target {adapter_id} is observe-only"
            ));
        }
        let mut controllers = self.adapter_controllers.write();
        if controllers.contains_key(adapter_id) {
            return Err(format!(
                "resource adapter controller {adapter_id} already registered"
            ));
        }
        controllers.insert(adapter_id.to_string(), controller);
        drop(controllers);
        self.bump_revision();
        Ok(())
    }

    /// Authorize one registered adapter to request specific operations from a
    /// managed target. Registration never grants control implicitly.
    ///
    /// # Errors
    ///
    /// Rejects missing adapters/controllers, self-control, empty grants, and
    /// lifecycle operations not declared by the target.
    pub fn register_adapter_transition_grant(
        &self,
        requested_by_adapter_id: &str,
        target_adapter_id: &str,
        operations: impl IntoIterator<Item = ResourceAdapterOperation>,
    ) -> Result<(), String> {
        if requested_by_adapter_id.trim().is_empty()
            || target_adapter_id.trim().is_empty()
            || requested_by_adapter_id == target_adapter_id
        {
            return Err("resource adapter transition grant ids are invalid".into());
        }
        if !self.adapter_registry.contains(requested_by_adapter_id) {
            return Err(format!(
                "resource transition grant requester {requested_by_adapter_id} is unregistered"
            ));
        }
        let descriptor = self
            .adapter_registry
            .descriptor(target_adapter_id)
            .ok_or_else(|| {
                format!("resource transition grant target {target_adapter_id} is unregistered")
            })?;
        if !self
            .adapter_controllers
            .read()
            .contains_key(target_adapter_id)
        {
            return Err(format!(
                "resource transition grant target {target_adapter_id} has no controller"
            ));
        }
        let operations = operations.into_iter().collect::<BTreeSet<_>>();
        if operations.is_empty()
            || operations
                .iter()
                .any(|operation| !descriptor.lifecycle_operations.contains(operation))
        {
            return Err(format!(
                "resource transition grant for {target_adapter_id} contains unsupported operations"
            ));
        }
        self.transition_grants
            .write()
            .entry((
                requested_by_adapter_id.to_string(),
                target_adapter_id.to_string(),
            ))
            .or_default()
            .extend(operations);
        self.bump_revision();
        Ok(())
    }

    #[must_use]
    pub fn state_revision(&self) -> u64 {
        self.state_revision.load(Ordering::Acquire)
    }

    pub async fn lock_adapter_operation(
        &self,
        adapter_id: &str,
    ) -> tokio::sync::OwnedMutexGuard<()> {
        let lock = self
            .adapter_operation_locks
            .entry(adapter_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();
        lock.lock_owned().await
    }

    pub async fn refresh(&self) -> ResourceCoordinationDiagnostics {
        let snapshot = self.snapshot_source.snapshot().await;
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        let pruned = prune_expired(&mut state.leases, now_ms);
        let snapshot_changed = snapshot_materially_changed(&state.last_snapshot, &snapshot);
        let previous_pressure = state.last_pressure;
        state.last_pressure = pressure_for(&snapshot, &self.policy, configured_gpu_device_index());
        state.last_reason_codes = pressure_reason_codes(
            &snapshot,
            &self.policy,
            configured_gpu_device_index(),
            state.last_pressure,
        );
        state.last_snapshot = snapshot;
        if pruned || snapshot_changed || previous_pressure != state.last_pressure {
            self.bump_revision();
        }
        self.diagnostics_from_state(&state)
    }

    /// Record an already-running external request without launching a device probe.
    ///
    /// Observe-only adapters cannot promise capacity and must not add `nvidia-smi`
    /// process latency to a foreground token path. Cold-load adapters use
    /// [`Self::admit`] instead.
    pub fn begin_observed_activity(
        &self,
        adapter_id: impl Into<String>,
        workload_id: impl Into<String>,
        profile_id: Option<String>,
        priority: ResourcePriority,
    ) -> String {
        let now_ms = now_epoch_ms();
        let adapter_id = adapter_id.into();
        let lease_id = format!(
            "resource-lease-{}",
            self.next_lease_id.fetch_add(1, Ordering::Relaxed)
        );
        let lease = ResourceLeaseDiagnostic {
            lease_id: lease_id.clone(),
            adapter_id,
            workload_id: workload_id.into(),
            profile_id,
            gpu_device_index: None,
            reservation_mib: 0,
            actual_mib: 0,
            ram_reservation_mib: 0,
            actual_ram_mib: 0,
            cpu_thread_reservation: 0,
            actual_cpu_threads: 0,
            priority,
            control_mode: oclive_kernel_types::ResourceControlMode::ObserveOnly,
            state: ResourceLeaseState::Active,
            granted_at_ms: now_ms,
            expires_at_ms: Some(now_ms.saturating_add(self.policy.active_lease_ttl_ms)),
            reason_codes: vec!["external_activity_observed".into()],
        };
        self.state.lock().leases.insert(lease_id.clone(), lease);
        self.bump_revision();
        lease_id
    }

    pub async fn admit(&self, request: ResourceAdmissionRequest) -> ResourceAdmissionResult {
        let Some((_permit, queue_wait_ms)) = self
            .admission_queue
            .acquire(&request, self.policy.admission_queue_timeout_ms)
            .await
        else {
            let state = self.state.lock();
            return ResourceAdmissionResult {
                decision: ResourceAdmissionDecision::Denied,
                lease: None,
                snapshot: state.last_snapshot.clone(),
                pressure: state.last_pressure,
                queue_wait_ms: self.policy.admission_queue_timeout_ms,
                preempted_adapters: Vec::new(),
                reason_codes: vec!["resource_admission_queue_timeout".into()],
            };
        };
        let mut result = self.admit_once(request.clone()).await;
        if result.decision == ResourceAdmissionDecision::Denied
            && self.policy.automatic_preemption
            && preemption_relevant(&result)
        {
            let candidates = self.preemption_candidates(&request);
            let mut preempted = Vec::new();
            let mut preemption_failed = false;
            for candidate in candidates {
                let transition = ResourceAdapterTransitionRequest {
                    adapter_id: candidate.adapter_id.clone(),
                    operation: candidate.operation,
                    requested_by_adapter_id: request.adapter_id.clone(),
                    profile_id: candidate.profile_id.clone(),
                    expected_revision: None,
                    reason: Some("higher-priority resource admission".into()),
                };
                match self.transition_adapter(&transition).await {
                    Ok(_) => {
                        preempted.push(ResourcePreemptionRecord {
                            adapter_id: candidate.adapter_id,
                            operation: candidate.operation,
                            restore_operation: candidate.restore_operation,
                            profile_id: candidate.profile_id,
                        });
                        result = self.admit_once(request.clone()).await;
                        if result.decision != ResourceAdmissionDecision::Denied {
                            result.preempted_adapters = preempted.clone();
                            break;
                        }
                    }
                    Err(error) => {
                        preemption_failed = true;
                        tracing::warn!(
                            target: "oclive_resource",
                            adapter_id = candidate.adapter_id,
                            %error,
                            "automatic resource preemption failed"
                        );
                    }
                }
            }
            if result.decision == ResourceAdmissionDecision::Denied && !preempted.is_empty() {
                if let Err(error) = self
                    .restore_preempted_adapters(&request.adapter_id, &preempted)
                    .await
                {
                    tracing::error!(
                        target: "oclive_resource",
                        %error,
                        "automatic resource preemption rollback failed"
                    );
                    result
                        .reason_codes
                        .push("resource_preemption_rollback_failed".into());
                    result.preempted_adapters = preempted.clone();
                }
            }
            if preemption_failed
                && !result
                    .reason_codes
                    .iter()
                    .any(|reason| reason == "resource_preemption_failed")
            {
                result
                    .reason_codes
                    .push("resource_preemption_failed".into());
            }
        }
        result.queue_wait_ms = queue_wait_ms;
        result
    }

    /// Restore adapters previously yielded by automatic admission preemption.
    ///
    /// Recovery is attempted in reverse order through the same exact
    /// requester-to-target grants used during preemption.
    ///
    /// # Errors
    ///
    /// Returns an aggregate unavailable error after attempting every restore
    /// when one or more controllers fail.
    pub async fn restore_preempted_adapters(
        &self,
        requested_by_adapter_id: &str,
        records: &[ResourcePreemptionRecord],
    ) -> Result<(), AppError> {
        let mut failures = Vec::new();
        for record in records.iter().rev() {
            let request = ResourceAdapterTransitionRequest {
                adapter_id: record.adapter_id.clone(),
                operation: record.restore_operation,
                requested_by_adapter_id: requested_by_adapter_id.to_string(),
                profile_id: record.profile_id.clone(),
                expected_revision: None,
                reason: Some("preempting resource workload released".into()),
            };
            if let Err(error) = self.transition_adapter(&request).await {
                failures.push(format!("{}:{error}", record.adapter_id));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(AppError::RemoteServiceUnavailable(format!(
                "resource_preemption_restore_failed:{}",
                failures.join("|")
            )))
        }
    }

    async fn admit_once(&self, request: ResourceAdmissionRequest) -> ResourceAdmissionResult {
        if let Some(profile_id) = request.profile_id.as_deref() {
            if self.adapter_registry.contains(&request.adapter_id)
                && !self
                    .adapter_registry
                    .profile_is_registered(&request.adapter_id, profile_id)
            {
                return ResourceAdmissionResult {
                    decision: ResourceAdmissionDecision::Denied,
                    lease: None,
                    snapshot: ResourceSnapshot::unavailable(
                        "not_evaluated",
                        "resource_profile_unregistered",
                    ),
                    pressure: ResourcePressureLevel::Unknown,
                    queue_wait_ms: 0,
                    preempted_adapters: Vec::new(),
                    reason_codes: vec!["resource_profile_unregistered".into()],
                };
            }
        }
        let snapshot = self.snapshot_source.snapshot().await;
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        let pruned = prune_expired(&mut state.leases, now_ms);
        let snapshot_changed = snapshot_materially_changed(&state.last_snapshot, &snapshot);
        let previous_pressure = state.last_pressure;
        state.last_pressure = pressure_for(&snapshot, &self.policy, request.gpu_device_index);
        state.last_snapshot = snapshot.clone();
        if pruned || snapshot_changed || previous_pressure != state.last_pressure {
            self.bump_revision();
        }

        let reused_lease = state
            .leases
            .values_mut()
            .find(|lease| {
                lease.adapter_id == request.adapter_id
                    && lease.workload_id == request.workload_id
                    && lease.profile_id == request.profile_id
                    && lease.gpu_device_index == request.gpu_device_index
                    && lease.reservation_mib == request.reservation_mib
                    && lease.ram_reservation_mib == request.ram_reservation_mib
                    && lease.cpu_thread_reservation == request.cpu_thread_reservation
                    && lease.control_mode == request.control_mode
            })
            .map(|lease| {
                let ttl = match lease.state {
                    ResourceLeaseState::Reserved => self.policy.pending_lease_ttl_ms,
                    ResourceLeaseState::Active => self.policy.active_lease_ttl_ms,
                };
                lease.priority = lease.priority.max(request.priority);
                lease.expires_at_ms = if lease.state == ResourceLeaseState::Active
                    && lease.control_mode == oclive_kernel_types::ResourceControlMode::Managed
                {
                    None
                } else {
                    Some(now_ms.saturating_add(ttl))
                };
                lease.clone()
            });
        if let Some(lease) = reused_lease {
            state.last_reason_codes.clear();
            self.bump_revision();
            return ResourceAdmissionResult {
                decision: ResourceAdmissionDecision::Reused,
                lease: Some(lease),
                snapshot,
                pressure: state.last_pressure,
                queue_wait_ms: 0,
                preempted_adapters: Vec::new(),
                reason_codes: Vec::new(),
            };
        }

        let pending_gpu_mib = state
            .leases
            .values()
            .filter(|lease| {
                lease.state == ResourceLeaseState::Reserved
                    && lease.gpu_device_index == request.gpu_device_index
            })
            .map(|lease| lease.reservation_mib)
            .sum::<u64>();
        let pending_ram_mib = state
            .leases
            .values()
            .filter(|lease| lease.state == ResourceLeaseState::Reserved)
            .map(|lease| lease.ram_reservation_mib)
            .sum::<u64>();
        let reserved_cpu_threads = state
            .leases
            .values()
            .map(|lease| u64::from(lease.actual_cpu_threads.max(lease.cpu_thread_reservation)))
            .sum::<u64>();
        let selected_free_mib = free_mib_for_device(&snapshot, request.gpu_device_index);
        let required_gpu_mib = self
            .policy
            .gpu_safety_reserve_mib
            .saturating_add(pending_gpu_mib)
            .saturating_add(request.reservation_mib);
        let mut denied_reasons = Vec::new();
        let mut unverified_reasons = Vec::new();
        if request.reservation_mib > 0 {
            if !snapshot.available {
                unverified_reasons.push("gpu_snapshot_unavailable".to_string());
            } else if selected_free_mib.is_none() {
                denied_reasons.push("gpu_device_unavailable".to_string());
            } else if selected_free_mib.is_some_and(|free_mib| free_mib < required_gpu_mib) {
                denied_reasons.push("insufficient_gpu_headroom".to_string());
            }
        }
        if request.ram_reservation_mib > 0 {
            match snapshot.system_memory.as_ref() {
                Some(memory)
                    if memory.available_mib
                        < self
                            .policy
                            .system_memory_safety_reserve_mib
                            .saturating_add(pending_ram_mib)
                            .saturating_add(request.ram_reservation_mib) =>
                {
                    denied_reasons.push("insufficient_system_memory_headroom".into());
                }
                Some(_) => {}
                None => unverified_reasons.push("system_memory_snapshot_unavailable".into()),
            }
        }
        if request.cpu_thread_reservation > 0 {
            match snapshot.cpu.as_ref() {
                Some(cpu)
                    if u64::from(cpu.logical_cores)
                        < u64::from(self.policy.cpu_safety_reserve_threads)
                            .saturating_add(reserved_cpu_threads)
                            .saturating_add(u64::from(request.cpu_thread_reservation)) =>
                {
                    denied_reasons.push("insufficient_cpu_thread_headroom".into());
                }
                Some(_) => {}
                None => unverified_reasons.push("cpu_snapshot_unavailable".into()),
            }
        }
        let unverified_allowed = request.admission_mode == ResourceAdmissionMode::ObserveOnly
            || self.policy.allow_unverified_admission;
        let (decision, reason_codes) = if !denied_reasons.is_empty()
            && request.admission_mode != ResourceAdmissionMode::ObserveOnly
        {
            (ResourceAdmissionDecision::Denied, denied_reasons)
        } else if !unverified_reasons.is_empty() && !unverified_allowed {
            (ResourceAdmissionDecision::Denied, unverified_reasons)
        } else if !denied_reasons.is_empty() || !unverified_reasons.is_empty() {
            denied_reasons.extend(unverified_reasons);
            (ResourceAdmissionDecision::GrantedUnverified, denied_reasons)
        } else {
            (ResourceAdmissionDecision::Granted, Vec::new())
        };

        state.last_reason_codes = reason_codes.clone();
        if decision == ResourceAdmissionDecision::Denied {
            return ResourceAdmissionResult {
                decision,
                lease: None,
                snapshot,
                pressure: state.last_pressure,
                queue_wait_ms: 0,
                preempted_adapters: Vec::new(),
                reason_codes,
            };
        }

        let lease_id = format!(
            "resource-lease-{}",
            self.next_lease_id.fetch_add(1, Ordering::Relaxed)
        );
        let lease = ResourceLeaseDiagnostic {
            lease_id: lease_id.clone(),
            adapter_id: request.adapter_id,
            workload_id: request.workload_id,
            profile_id: request.profile_id,
            gpu_device_index: request.gpu_device_index,
            reservation_mib: request.reservation_mib,
            actual_mib: 0,
            ram_reservation_mib: request.ram_reservation_mib,
            actual_ram_mib: 0,
            cpu_thread_reservation: request.cpu_thread_reservation,
            actual_cpu_threads: 0,
            priority: request.priority,
            control_mode: request.control_mode,
            state: ResourceLeaseState::Reserved,
            granted_at_ms: now_ms,
            expires_at_ms: Some(now_ms.saturating_add(self.policy.pending_lease_ttl_ms)),
            reason_codes: reason_codes.clone(),
        };
        state.leases.insert(lease_id, lease.clone());
        self.bump_revision();
        ResourceAdmissionResult {
            decision,
            lease: Some(lease),
            snapshot,
            pressure: state.last_pressure,
            queue_wait_ms: 0,
            preempted_adapters: Vec::new(),
            reason_codes,
        }
    }
}
