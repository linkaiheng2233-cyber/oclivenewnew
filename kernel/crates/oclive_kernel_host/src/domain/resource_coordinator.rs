//! Host-owned resource admission, leases, pressure, and diagnostics.
//!
//! This is a control-plane facility. It does not execute model, voice, or
//! rendering business data paths; concrete adapters remain responsible for
//! starting and stopping their own runtimes.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use oclive_kernel_contracts::{
    ResourceAdapterController, ResourceAdapterRegistrar, ResourceSnapshotSource,
};
use oclive_kernel_types::{
    AppError, ResourceAdapterDescriptor, ResourceAdapterOperation, ResourceAdapterRegistration,
    ResourceAdapterRegistrationSource, ResourceAdapterTransitionRequest,
    ResourceAdapterTransitionResponse, ResourceAdmissionDecision, ResourceAdmissionMode,
    ResourceAdmissionQueueDiagnostics, ResourceAdmissionQueueItem, ResourceAdmissionRequest,
    ResourceAdmissionResult, ResourceCandidatePlan, ResourceCandidateTransition,
    ResourceControlMode, ResourceCoordinationDiagnosticState, ResourceCoordinationDiagnostics,
    ResourceCoordinatorPolicy, ResourceLeaseDiagnostic, ResourceLeaseState,
    ResourcePreemptionRecord, ResourcePressureLevel, ResourcePriority, ResourceSnapshot,
    RESOURCE_COORDINATION_SCHEMA_VERSION,
};
use parking_lot::{Mutex, RwLock};

use super::resource_adapter_registry::ResourceAdapterRegistry;
use super::resource_plan::{compile_resource_candidate_plan, CompileResourceCandidatePlanInput};

#[derive(Debug)]
struct CoordinatorState {
    last_snapshot: ResourceSnapshot,
    last_pressure: ResourcePressureLevel,
    last_reason_codes: Vec<String>,
    leases: BTreeMap<String, ResourceLeaseDiagnostic>,
}

impl Default for CoordinatorState {
    fn default() -> Self {
        Self {
            last_snapshot: ResourceSnapshot::unavailable(
                "not_evaluated",
                "resource_snapshot_not_evaluated",
            ),
            last_pressure: ResourcePressureLevel::Unknown,
            last_reason_codes: vec!["resource_snapshot_not_evaluated".into()],
            leases: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Default)]
struct AdmissionQueueState {
    active_ticket_id: Option<u64>,
    queued: BTreeMap<u64, ResourceAdmissionQueueItem>,
}

#[derive(Debug)]
struct AdmissionQueue {
    state: Mutex<AdmissionQueueState>,
    notify: tokio::sync::Notify,
    next_ticket_id: AtomicU64,
    aging_quantum_ms: u64,
}

impl AdmissionQueue {
    fn new(aging_quantum_ms: u64) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(AdmissionQueueState::default()),
            notify: tokio::sync::Notify::new(),
            next_ticket_id: AtomicU64::new(1),
            aging_quantum_ms: aging_quantum_ms.max(1),
        })
    }

    async fn acquire(
        self: &Arc<Self>,
        request: &ResourceAdmissionRequest,
        timeout_ms: u64,
    ) -> Option<(AdmissionQueuePermit, u64)> {
        let ticket_id = self.next_ticket_id.fetch_add(1, Ordering::Relaxed);
        let enqueued_at_ms = now_epoch_ms();
        self.state.lock().queued.insert(
            ticket_id,
            ResourceAdmissionQueueItem {
                ticket_id,
                adapter_id: request.adapter_id.clone(),
                workload_id: request.workload_id.clone(),
                priority: request.priority,
                enqueued_at_ms,
            },
        );
        self.notify.notify_waiters();
        let mut waiting = AdmissionQueueWaiter {
            queue: Arc::clone(self),
            ticket_id,
            acquired: false,
        };
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms.max(1));
        loop {
            let notified = self.notify.notified();
            let selected = {
                let mut state = self.state.lock();
                let candidate = (state.active_ticket_id.is_none())
                    .then(|| select_queue_candidate(&state.queued, self.aging_quantum_ms))
                    .flatten();
                if candidate == Some(ticket_id) {
                    state.active_ticket_id = Some(ticket_id);
                    state.queued.remove(&ticket_id);
                    true
                } else {
                    false
                }
            };
            if selected {
                waiting.acquired = true;
                return Some((
                    AdmissionQueuePermit {
                        queue: Arc::clone(self),
                        ticket_id,
                    },
                    now_epoch_ms().saturating_sub(enqueued_at_ms),
                ));
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return None;
            }
        }
    }

    fn diagnostics(&self) -> ResourceAdmissionQueueDiagnostics {
        let state = self.state.lock();
        ResourceAdmissionQueueDiagnostics {
            active_ticket_id: state.active_ticket_id,
            queued: state.queued.values().cloned().collect(),
        }
    }

    fn abandon(&self, ticket_id: u64, acquired: bool) {
        let mut state = self.state.lock();
        let changed = if acquired {
            if state.active_ticket_id == Some(ticket_id) {
                state.active_ticket_id = None;
                true
            } else {
                false
            }
        } else {
            state.queued.remove(&ticket_id).is_some()
        };
        drop(state);
        if changed {
            self.notify.notify_waiters();
        }
    }
}

struct AdmissionQueueWaiter {
    queue: Arc<AdmissionQueue>,
    ticket_id: u64,
    acquired: bool,
}

impl Drop for AdmissionQueueWaiter {
    fn drop(&mut self) {
        if !self.acquired {
            self.queue.abandon(self.ticket_id, false);
        }
    }
}

struct AdmissionQueuePermit {
    queue: Arc<AdmissionQueue>,
    ticket_id: u64,
}

impl Drop for AdmissionQueuePermit {
    fn drop(&mut self) {
        self.queue.abandon(self.ticket_id, true);
    }
}

fn select_queue_candidate(
    queued: &BTreeMap<u64, ResourceAdmissionQueueItem>,
    aging_quantum_ms: u64,
) -> Option<u64> {
    let now_ms = now_epoch_ms();
    queued
        .values()
        .max_by_key(|item| {
            let base = match item.priority {
                ResourcePriority::Resident => 0_u64,
                ResourcePriority::BackgroundWarmup => 1,
                ResourcePriority::ForegroundMedia => 2,
                ResourcePriority::ForegroundInteractive => 3,
            };
            let age_boost = now_ms
                .saturating_sub(item.enqueued_at_ms)
                .checked_div(aging_quantum_ms.max(1))
                .unwrap_or(0)
                .min(3);
            (
                base.saturating_add(age_boost).min(3),
                u64::MAX - item.ticket_id,
            )
        })
        .map(|item| item.ticket_id)
}

pub struct ResourceCoordinator {
    policy: ResourceCoordinatorPolicy,
    snapshot_source: Arc<dyn ResourceSnapshotSource>,
    state: Mutex<CoordinatorState>,
    adapter_registry: ResourceAdapterRegistry,
    adapter_controllers: RwLock<BTreeMap<String, Arc<dyn ResourceAdapterController>>>,
    transition_grants: RwLock<BTreeMap<(String, String), BTreeSet<ResourceAdapterOperation>>>,
    state_revision: AtomicU64,
    next_lease_id: AtomicU64,
    adapter_operation_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    admission_queue: Arc<AdmissionQueue>,
}

#[must_use]
pub fn configured_gpu_device_index() -> Option<u32> {
    std::env::var("OCLIVE_GPU_DEVICE_INDEX")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .or_else(|| {
            std::env::var("CUDA_VISIBLE_DEVICES")
                .ok()
                .and_then(|value| value.split(',').next()?.trim().parse::<u32>().ok())
        })
}

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

    pub fn activate(&self, lease_id: &str, actual_mib: Option<u64>) -> bool {
        self.activate_with_usage(lease_id, actual_mib, None, None)
    }

    /// Confirm runtime activation with optional measured GPU, RAM, and CPU
    /// usage. Missing measurements retain the admitted reservation.
    pub fn activate_with_usage(
        &self,
        lease_id: &str,
        actual_mib: Option<u64>,
        actual_ram_mib: Option<u64>,
        actual_cpu_threads: Option<u16>,
    ) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        let Some(lease) = state.leases.get_mut(lease_id) else {
            return false;
        };
        lease.state = ResourceLeaseState::Active;
        lease.actual_mib = actual_mib.unwrap_or(lease.reservation_mib);
        lease.actual_ram_mib = actual_ram_mib.unwrap_or(lease.ram_reservation_mib);
        lease.actual_cpu_threads = actual_cpu_threads.unwrap_or(lease.cpu_thread_reservation);
        lease.expires_at_ms =
            if lease.control_mode == oclive_kernel_types::ResourceControlMode::Managed {
                None
            } else {
                Some(now_ms.saturating_add(self.policy.active_lease_ttl_ms))
            };
        self.bump_revision();
        true
    }

    pub fn release(&self, lease_id: &str) -> bool {
        let removed = self.state.lock().leases.remove(lease_id).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn release_workload(&self, adapter_id: &str, workload_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id || lease.workload_id != workload_id);
        let released = before.saturating_sub(state.leases.len());
        if released > 0 {
            self.bump_revision();
        }
        released
    }

    pub fn release_adapter(&self, adapter_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id);
        let released = before.saturating_sub(state.leases.len());
        if released > 0 {
            self.bump_revision();
        }
        released
    }

    /// Attach a stable operational reason to every current lease for one adapter.
    ///
    /// Returns the number of leases that gained the reason. Repeated reasons are
    /// idempotent so retries do not grow diagnostics without bound.
    pub fn record_adapter_reason(&self, adapter_id: &str, reason_code: &str) -> usize {
        let reason_code = reason_code.trim();
        if adapter_id.trim().is_empty() || reason_code.is_empty() {
            return 0;
        }
        let mut state = self.state.lock();
        let mut updated = 0;
        for lease in state
            .leases
            .values_mut()
            .filter(|lease| lease.adapter_id == adapter_id)
        {
            if !lease
                .reason_codes
                .iter()
                .any(|existing| existing == reason_code)
            {
                lease.reason_codes.push(reason_code.to_string());
                updated += 1;
            }
        }
        if updated > 0 {
            self.bump_revision();
        }
        updated
    }

    #[must_use]
    pub fn adapter_has_reason(&self, adapter_id: &str, reason_code: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state.leases.values().any(|lease| {
            lease.adapter_id == adapter_id
                && lease
                    .reason_codes
                    .iter()
                    .any(|existing| existing == reason_code)
        })
    }

    #[must_use]
    pub fn has_active_priority(&self, minimum: ResourcePriority) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state
            .leases
            .values()
            .any(|lease| lease.state == ResourceLeaseState::Active && lease.priority >= minimum)
    }

    #[must_use]
    pub fn has_active_adapter(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state.leases.values().any(|lease| {
            lease.state == ResourceLeaseState::Active && lease.adapter_id == adapter_id
        })
    }

    #[must_use]
    /// Whether the adapter has a reserved or active lease after TTL pruning.
    pub fn has_adapter_lease(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state
            .leases
            .values()
            .any(|lease| lease.adapter_id == adapter_id)
    }

    /// Execute one lifecycle operation through the adapter's authoritative
    /// controller after descriptor, caller, profile, revision, and lock checks.
    ///
    /// # Errors
    ///
    /// Returns stable invalid-parameter or unavailable errors when the request
    /// is stale, unsupported, uncontrolled, or rejected by the runtime.
    pub async fn transition_adapter(
        &self,
        request: &ResourceAdapterTransitionRequest,
    ) -> Result<ResourceAdapterTransitionResponse, AppError> {
        let controller = self.validate_transition(
            &request.adapter_id,
            request.operation,
            request.profile_id.as_deref(),
            Some(&request.requested_by_adapter_id),
            false,
        )?;
        // Only the lifecycle target is a single-writer resource. Callers may
        // already hold their own adapter lock across admission and use (the
        // bundled voice path does), so recursively locking the requester here
        // would deadlock automatic preemption.
        let _guards = self
            .lock_adapter_operations([request.adapter_id.as_str()])
            .await;
        self.ensure_expected_revision(request.expected_revision)?;
        let outcome = controller
            .transition(
                request.operation,
                request.profile_id.as_deref(),
                request.reason.as_deref(),
            )
            .await?;
        let lease_changed = if transition_releases_residency(request.operation) {
            self.release_adapter(&request.adapter_id) > 0
        } else {
            false
        };
        if !outcome.already_in_state && !lease_changed {
            self.bump_revision();
        }
        Ok(ResourceAdapterTransitionResponse {
            schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
            adapter_id: request.adapter_id.clone(),
            operation: request.operation,
            requested_by_adapter_id: request.requested_by_adapter_id.clone(),
            already_in_state: outcome.already_in_state,
            recovery_scheduled: outcome.recovery_scheduled,
            state_revision: self.state_revision(),
        })
    }

    /// Execute a previously compiled candidate plan as one serialized batch.
    ///
    /// The method is intentionally not exposed as an HTTP command yet. It is
    /// the generic host foundation used by controlled callers after a plan has
    /// been reviewed. Completed steps are rolled back in reverse order when a
    /// later step fails; rollback failure is reported and never treated as
    /// confirmed release.
    ///
    /// # Errors
    ///
    /// Rejects stale, blocked, non-executable, unsupported, or failed plans.
    pub async fn execute_candidate_plan(
        &self,
        plan: &ResourceCandidatePlan,
    ) -> Result<Vec<ResourceAdapterTransitionResponse>, AppError> {
        if plan.state == oclive_kernel_types::ResourceCandidatePlanState::Blocked {
            return Err(AppError::InvalidParameter("resource_plan_blocked".into()));
        }
        if !plan.executable {
            return Err(AppError::InvalidParameter(
                "resource_plan_not_executable".into(),
            ));
        }
        let current_plan = self.diagnostics_snapshot().candidate_plan;
        if current_plan.plan_id != plan.plan_id
            || current_plan.compiled_from_revision != plan.compiled_from_revision
            || current_plan.selections != plan.selections
            || current_plan.transitions != plan.transitions
        {
            return Err(AppError::InvalidParameter(
                "resource_plan_candidate_mismatch".into(),
            ));
        }
        let adapter_ids = plan
            .transitions
            .iter()
            .map(|transition| transition.adapter_id.as_str());
        let _guards = self.lock_adapter_operations(adapter_ids).await;
        self.ensure_expected_revision(Some(plan.compiled_from_revision))?;

        let prepared = plan
            .transitions
            .iter()
            .map(|transition| {
                self.validate_transition(
                    &transition.adapter_id,
                    transition.operation,
                    transition.profile_id.as_deref(),
                    None,
                    false,
                )
                .map(|controller| (transition, controller))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut completed: Vec<(&ResourceCandidateTransition, bool)> = Vec::new();
        let mut responses = Vec::with_capacity(plan.transitions.len());
        for (transition, controller) in prepared {
            let reason =
                (!transition.reason_codes.is_empty()).then(|| transition.reason_codes.join(","));
            match controller
                .transition(
                    transition.operation,
                    transition.profile_id.as_deref(),
                    reason.as_deref(),
                )
                .await
            {
                Ok(outcome) => {
                    let lease_changed = if transition_releases_residency(transition.operation) {
                        self.release_adapter(&transition.adapter_id) > 0
                    } else {
                        false
                    };
                    if !outcome.already_in_state && !lease_changed {
                        self.bump_revision();
                    }
                    completed.push((transition, !outcome.already_in_state));
                    responses.push(ResourceAdapterTransitionResponse {
                        schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
                        adapter_id: transition.adapter_id.clone(),
                        operation: transition.operation,
                        requested_by_adapter_id: transition
                            .requested_by_adapter_id
                            .clone()
                            .unwrap_or_else(|| "host.resource_coordinator".into()),
                        already_in_state: outcome.already_in_state,
                        recovery_scheduled: outcome.recovery_scheduled,
                        state_revision: self.state_revision(),
                    });
                }
                Err(error) => {
                    let rollback_errors = self.rollback_completed(&completed).await;
                    let rollback_detail = if rollback_errors.is_empty() {
                        "rollback_confirmed".to_string()
                    } else {
                        format!("rollback_failed:{}", rollback_errors.join("|"))
                    };
                    return Err(AppError::RemoteServiceUnavailable(format!(
                        "resource_plan_transition_failed:{}:{error};{rollback_detail}",
                        transition.adapter_id
                    )));
                }
            }
        }
        Ok(responses)
    }

    #[must_use]
    pub fn diagnostics_snapshot(&self) -> ResourceCoordinationDiagnostics {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        self.diagnostics_from_state(&state)
    }

    async fn rollback_completed(
        &self,
        completed: &[(&ResourceCandidateTransition, bool)],
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for (transition, changed_state) in completed.iter().rev() {
            if !changed_state {
                continue;
            }
            let Some(rollback_operation) = transition.rollback_operation else {
                errors.push(format!(
                    "{}:resource_plan_rollback_unavailable",
                    transition.adapter_id
                ));
                continue;
            };
            let controller = match self.validate_transition(
                &transition.adapter_id,
                rollback_operation,
                transition.rollback_profile_id.as_deref(),
                None,
                true,
            ) {
                Ok(controller) => controller,
                Err(error) => {
                    errors.push(format!("{}:{error}", transition.adapter_id));
                    continue;
                }
            };
            if let Err(error) = controller
                .transition(
                    rollback_operation,
                    transition.rollback_profile_id.as_deref(),
                    Some("resource plan rollback"),
                )
                .await
            {
                errors.push(format!("{}:{error}", transition.adapter_id));
            } else {
                let lease_changed = if transition_releases_residency(rollback_operation) {
                    self.release_adapter(&transition.adapter_id) > 0
                } else {
                    false
                };
                if !lease_changed {
                    self.bump_revision();
                }
            }
        }
        errors
    }

    fn validate_transition(
        &self,
        adapter_id: &str,
        operation: ResourceAdapterOperation,
        profile_id: Option<&str>,
        requested_by_adapter_id: Option<&str>,
        allow_nonselectable_profile: bool,
    ) -> Result<Arc<dyn ResourceAdapterController>, AppError> {
        if adapter_id.trim().is_empty() || adapter_id.trim() != adapter_id {
            return Err(AppError::InvalidParameter(
                "resource_transition_adapter_id_invalid".into(),
            ));
        }
        if let Some(requested_by) = requested_by_adapter_id {
            if requested_by == adapter_id {
                return Err(AppError::InvalidParameter(
                    "resource_transition_self_request".into(),
                ));
            }
            if !self.adapter_registry.contains(requested_by) {
                return Err(AppError::InvalidParameter(
                    "resource_transition_requester_unregistered".into(),
                ));
            }
        }
        let descriptor = self
            .adapter_registry
            .descriptor(adapter_id)
            .ok_or_else(|| {
                AppError::InvalidParameter("resource_transition_adapter_unregistered".into())
            })?;
        if descriptor.control_mode != ResourceControlMode::Managed {
            return Err(AppError::InvalidParameter(
                "resource_transition_control_unavailable".into(),
            ));
        }
        if !descriptor.lifecycle_operations.contains(&operation) {
            return Err(AppError::InvalidParameter(
                "resource_transition_operation_unsupported".into(),
            ));
        }
        if let Some(profile_id) = profile_id {
            let profile = descriptor
                .profiles
                .iter()
                .find(|profile| profile.profile_id == profile_id)
                .ok_or_else(|| {
                    AppError::InvalidParameter("resource_profile_unregistered".into())
                })?;
            if !allow_nonselectable_profile
                && matches!(
                    operation,
                    ResourceAdapterOperation::Start | ResourceAdapterOperation::Resume
                )
                && !profile.coordinator_selectable
            {
                return Err(AppError::InvalidParameter(
                    "resource_profile_not_coordinator_selectable".into(),
                ));
            }
        }
        if let Some(requested_by) = requested_by_adapter_id {
            let authorized = self
                .transition_grants
                .read()
                .get(&(requested_by.to_string(), adapter_id.to_string()))
                .is_some_and(|operations| operations.contains(&operation));
            if !authorized {
                return Err(AppError::InvalidParameter(
                    "resource_transition_not_authorized".into(),
                ));
            }
        }
        self.adapter_controllers
            .read()
            .get(adapter_id)
            .cloned()
            .ok_or_else(|| {
                AppError::RemoteServiceUnavailable(
                    "resource_transition_controller_unavailable".into(),
                )
            })
    }

    fn ensure_expected_revision(&self, expected: Option<u64>) -> Result<(), AppError> {
        if expected.is_some_and(|expected| expected != self.state_revision()) {
            return Err(AppError::RemoteServiceUnavailable(
                "resource_plan_stale_revision".into(),
            ));
        }
        Ok(())
    }

    async fn lock_adapter_operations<'a>(
        &self,
        adapter_ids: impl IntoIterator<Item = &'a str>,
    ) -> Vec<tokio::sync::OwnedMutexGuard<()>> {
        let ids = adapter_ids
            .into_iter()
            .filter(|adapter_id| !adapter_id.trim().is_empty())
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let mut guards = Vec::with_capacity(ids.len());
        for adapter_id in ids {
            guards.push(self.lock_adapter_operation(&adapter_id).await);
        }
        guards
    }

    fn controller_ids(&self) -> BTreeSet<String> {
        self.adapter_controllers.read().keys().cloned().collect()
    }

    fn preemption_candidates(
        &self,
        request: &ResourceAdmissionRequest,
    ) -> Vec<AutomaticPreemptionCandidate> {
        if !self.adapter_registry.contains(&request.adapter_id) {
            return Vec::new();
        }
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        if state.leases.values().any(|lease| {
            lease.state == ResourceLeaseState::Active
                && lease.adapter_id != request.adapter_id
                && lease.priority >= request.priority
        }) {
            return Vec::new();
        }
        let grants = self.transition_grants.read();
        let controllers = self.adapter_controllers.read();
        let candidates = state
            .leases
            .values()
            .filter(|lease| {
                lease.state == ResourceLeaseState::Active
                    && lease.control_mode == ResourceControlMode::Managed
                    && lease.adapter_id != request.adapter_id
                    && lease.priority < request.priority
            })
            .filter_map(|lease| {
                let descriptor = self.adapter_registry.descriptor(&lease.adapter_id)?;
                let operation = descriptor.automatic_preemption?;
                let restore_operation = restore_operation_for(operation)?;
                let authorized = grants
                    .get(&(request.adapter_id.clone(), lease.adapter_id.clone()))
                    .is_some_and(|operations| {
                        operations.contains(&operation) && operations.contains(&restore_operation)
                    });
                if !authorized || !controllers.contains_key(&lease.adapter_id) {
                    return None;
                }
                Some(AutomaticPreemptionCandidate {
                    adapter_id: lease.adapter_id.clone(),
                    profile_id: lease.profile_id.clone(),
                    operation,
                    restore_operation,
                    priority: lease.priority,
                    releasable_mib: lease.actual_mib.max(lease.reservation_mib),
                    releasable_ram_mib: lease.actual_ram_mib.max(lease.ram_reservation_mib),
                    releasable_cpu_threads: lease
                        .actual_cpu_threads
                        .max(lease.cpu_thread_reservation),
                    granted_at_ms: lease.granted_at_ms,
                })
            })
            .collect::<Vec<_>>();
        let mut by_adapter = BTreeMap::new();
        for candidate in candidates {
            by_adapter
                .entry(candidate.adapter_id.clone())
                .and_modify(|existing: &mut AutomaticPreemptionCandidate| {
                    existing.releasable_mib = existing
                        .releasable_mib
                        .saturating_add(candidate.releasable_mib);
                    existing.releasable_ram_mib = existing
                        .releasable_ram_mib
                        .saturating_add(candidate.releasable_ram_mib);
                    existing.releasable_cpu_threads = existing
                        .releasable_cpu_threads
                        .saturating_add(candidate.releasable_cpu_threads);
                    existing.priority = existing.priority.max(candidate.priority);
                    existing.granted_at_ms = existing.granted_at_ms.max(candidate.granted_at_ms);
                })
                .or_insert(candidate);
        }
        let mut candidates = by_adapter.into_values().collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| {
            (
                candidate.priority,
                std::cmp::Reverse(
                    candidate
                        .releasable_mib
                        .saturating_add(candidate.releasable_ram_mib),
                ),
                std::cmp::Reverse(candidate.releasable_cpu_threads),
                std::cmp::Reverse(candidate.granted_at_ms),
                candidate.adapter_id.clone(),
            )
        });
        candidates
    }

    fn diagnostics_from_state(&self, state: &CoordinatorState) -> ResourceCoordinationDiagnostics {
        let mut diagnostics = diagnostics_from_state(
            &self.policy,
            &self.adapter_registry,
            &self.controller_ids(),
            self.state_revision(),
            state,
        );
        diagnostics.admission_queue = self.admission_queue.diagnostics();
        diagnostics
    }

    fn bump_revision(&self) {
        self.state_revision.fetch_add(1, Ordering::AcqRel);
    }
}

#[derive(Debug)]
struct AutomaticPreemptionCandidate {
    adapter_id: String,
    profile_id: Option<String>,
    operation: ResourceAdapterOperation,
    restore_operation: ResourceAdapterOperation,
    priority: ResourcePriority,
    releasable_mib: u64,
    releasable_ram_mib: u64,
    releasable_cpu_threads: u16,
    granted_at_ms: u64,
}

impl ResourceAdapterRegistrar for ResourceCoordinator {
    fn register_adapter(&self, registration: ResourceAdapterRegistration) -> Result<(), AppError> {
        self.register_third_party_adapter(registration)
            .map_err(AppError::InvalidParameter)
    }

    fn register_controller(
        &self,
        source_id: &str,
        controller: Arc<dyn ResourceAdapterController>,
    ) -> Result<(), AppError> {
        self.register_third_party_adapter_controller(source_id, controller)
            .map_err(AppError::InvalidParameter)
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

fn preemption_relevant(result: &ResourceAdmissionResult) -> bool {
    result.reason_codes.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "insufficient_gpu_headroom"
                | "insufficient_system_memory_headroom"
                | "insufficient_cpu_thread_headroom"
        )
    })
}

fn restore_operation_for(operation: ResourceAdapterOperation) -> Option<ResourceAdapterOperation> {
    match operation {
        ResourceAdapterOperation::Suspend => Some(ResourceAdapterOperation::Resume),
        ResourceAdapterOperation::Unload => Some(ResourceAdapterOperation::Start),
        ResourceAdapterOperation::Observe
        | ResourceAdapterOperation::Start
        | ResourceAdapterOperation::Resume
        | ResourceAdapterOperation::Release => None,
    }
}

fn transition_releases_residency(operation: ResourceAdapterOperation) -> bool {
    matches!(
        operation,
        ResourceAdapterOperation::Suspend
            | ResourceAdapterOperation::Unload
            | ResourceAdapterOperation::Release
    )
}

fn prune_expired(leases: &mut BTreeMap<String, ResourceLeaseDiagnostic>, now_ms: u64) -> bool {
    let before = leases.len();
    leases.retain(|_, lease| {
        lease
            .expires_at_ms
            .is_none_or(|expires_at| expires_at > now_ms)
    });
    before != leases.len()
}

fn pressure_for(
    snapshot: &ResourceSnapshot,
    policy: &ResourceCoordinatorPolicy,
    gpu_device_index: Option<u32>,
) -> ResourcePressureLevel {
    let (gpu, ram) = pressure_levels(snapshot, policy, gpu_device_index);
    if [gpu, ram].contains(&ResourcePressureLevel::Critical) {
        ResourcePressureLevel::Critical
    } else if [gpu, ram].contains(&ResourcePressureLevel::Elevated) {
        ResourcePressureLevel::Elevated
    } else if [gpu, ram].contains(&ResourcePressureLevel::Unknown) {
        ResourcePressureLevel::Unknown
    } else {
        ResourcePressureLevel::Normal
    }
}

fn pressure_levels(
    snapshot: &ResourceSnapshot,
    policy: &ResourceCoordinatorPolicy,
    gpu_device_index: Option<u32>,
) -> (ResourcePressureLevel, ResourcePressureLevel) {
    let gpu = if snapshot.available && !snapshot.gpu_devices.is_empty() {
        let selected_free_mib = free_mib_for_device(snapshot, gpu_device_index).unwrap_or(0);
        if selected_free_mib <= policy.gpu_safety_reserve_mib {
            ResourcePressureLevel::Critical
        } else if selected_free_mib <= policy.gpu_safety_reserve_mib.saturating_mul(2) {
            ResourcePressureLevel::Elevated
        } else {
            ResourcePressureLevel::Normal
        }
    } else {
        ResourcePressureLevel::Unknown
    };
    let ram = snapshot
        .system_memory
        .as_ref()
        .map_or(ResourcePressureLevel::Unknown, |memory| {
            if memory.available_mib <= policy.system_memory_safety_reserve_mib {
                ResourcePressureLevel::Critical
            } else if memory.available_mib
                <= policy.system_memory_safety_reserve_mib.saturating_mul(2)
            {
                ResourcePressureLevel::Elevated
            } else {
                ResourcePressureLevel::Normal
            }
        });
    (gpu, ram)
}

fn pressure_reason_codes(
    snapshot: &ResourceSnapshot,
    policy: &ResourceCoordinatorPolicy,
    gpu_device_index: Option<u32>,
    pressure: ResourcePressureLevel,
) -> Vec<String> {
    let mut reasons = snapshot.reason_codes.clone();
    let (gpu, ram) = pressure_levels(snapshot, policy, gpu_device_index);
    match gpu {
        ResourcePressureLevel::Elevated => reasons.push("gpu_headroom_elevated".into()),
        ResourcePressureLevel::Critical => reasons.push("gpu_headroom_critical".into()),
        ResourcePressureLevel::Unknown | ResourcePressureLevel::Normal => {}
    }
    match ram {
        ResourcePressureLevel::Elevated => reasons.push("system_memory_headroom_elevated".into()),
        ResourcePressureLevel::Critical => reasons.push("system_memory_headroom_critical".into()),
        ResourcePressureLevel::Unknown | ResourcePressureLevel::Normal => {}
    }
    if pressure == ResourcePressureLevel::Unknown && reasons.is_empty() {
        reasons.push("resource_pressure_unknown".into());
    }
    reasons
}

fn free_mib_for_device(snapshot: &ResourceSnapshot, requested_index: Option<u32>) -> Option<u64> {
    requested_index.map_or_else(
        || {
            snapshot
                .gpu_devices
                .iter()
                .min_by_key(|device| device.device_index)
                .map(|device| device.free_mib)
        },
        |index| {
            snapshot
                .gpu_devices
                .iter()
                .find(|device| device.device_index == index)
                .map(|device| device.free_mib)
        },
    )
}

fn snapshot_materially_changed(previous: &ResourceSnapshot, next: &ResourceSnapshot) -> bool {
    previous.source != next.source
        || previous.available != next.available
        || previous.gpu_devices != next.gpu_devices
        || previous.system_memory != next.system_memory
        || previous.cpu != next.cpu
        || previous.reason_codes != next.reason_codes
}

fn diagnostics_from_state(
    policy: &ResourceCoordinatorPolicy,
    adapter_registry: &ResourceAdapterRegistry,
    controller_ids: &BTreeSet<String>,
    state_revision: u64,
    state: &CoordinatorState,
) -> ResourceCoordinationDiagnostics {
    let coordination_state = if state.last_snapshot.source == "not_evaluated" {
        ResourceCoordinationDiagnosticState::NotEvaluated
    } else {
        match state.last_pressure {
            ResourcePressureLevel::Normal => ResourceCoordinationDiagnosticState::Ready,
            ResourcePressureLevel::Unknown | ResourcePressureLevel::Elevated => {
                ResourceCoordinationDiagnosticState::Degraded
            }
            ResourcePressureLevel::Critical => ResourceCoordinationDiagnosticState::Blocked,
        }
    };
    let (adapters, registry_reason_codes) = adapter_registry.diagnostics(&state.leases);
    let scheduling = adapter_registry.scheduling_diagnostics(&policy.scheduling);
    let leases = state.leases.values().cloned().collect::<Vec<_>>();
    let candidate_plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
        state_revision,
        policy,
        snapshot: &state.last_snapshot,
        gpu_device_index: configured_gpu_device_index(),
        adapters: &adapters,
        leases: &leases,
        scheduling: &scheduling,
        controller_ids,
    });
    let mut reason_codes = state.last_reason_codes.clone();
    for reason in registry_reason_codes {
        if !reason_codes.contains(&reason) {
            reason_codes.push(reason);
        }
    }
    ResourceCoordinationDiagnostics {
        schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
        state_revision,
        state: coordination_state,
        pressure: state.last_pressure,
        policy: policy.clone(),
        snapshot: state.last_snapshot.clone(),
        adapters,
        leases,
        scheduling,
        candidate_plan,
        admission_queue: ResourceAdmissionQueueDiagnostics::default(),
        reason_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use oclive_kernel_contracts::{ResourceAdapterController, ResourceAdapterControllerOutcome};
    use oclive_kernel_types::{
        CpuSnapshot, GpuDeviceSnapshot, ResourceAdapterDescriptor, ResourceAdapterDomain,
        ResourceAdapterKind, ResourceAdapterOperation, ResourceAdapterRegistration,
        ResourceAdapterRegistrationSource, ResourceAdmissionMode, ResourceControlMode,
        ResourceExecutionTarget, ResourceOperatingProfile, ResourceResidencyMode,
        ResourceResidencyPreference, ResourceSchedulingCommand, ResourceSchedulingIntent,
        ResourceSchedulingStrategy, ResourceSnapshot, SystemMemorySnapshot,
    };
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    struct FixedSnapshot(ResourceSnapshot);

    struct MutableSnapshot(Arc<RwLock<ResourceSnapshot>>);

    struct GatedSnapshot {
        snapshot: ResourceSnapshot,
        calls: AtomicUsize,
        first_started: Arc<tokio::sync::Notify>,
        release_first: Arc<tokio::sync::Notify>,
    }

    struct RecordingController {
        adapter_id: String,
        events: Arc<Mutex<Vec<String>>>,
        fail_operations: BTreeSet<ResourceAdapterOperation>,
        delay_ms: u64,
        in_flight: AtomicUsize,
        max_in_flight: AtomicUsize,
    }

    struct SnapshotController {
        adapter_id: String,
        snapshot: Arc<RwLock<ResourceSnapshot>>,
        occupied_free_mib: u64,
        released_free_mib: u64,
        events: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingController {
        fn new(adapter_id: &str, events: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                adapter_id: adapter_id.into(),
                events,
                fail_operations: BTreeSet::new(),
                delay_ms: 0,
                in_flight: AtomicUsize::new(0),
                max_in_flight: AtomicUsize::new(0),
            }
        }

        fn failing(
            adapter_id: &str,
            events: Arc<Mutex<Vec<String>>>,
            operation: ResourceAdapterOperation,
        ) -> Self {
            let mut controller = Self::new(adapter_id, events);
            controller.fail_operations.insert(operation);
            controller
        }

        fn delayed(adapter_id: &str, events: Arc<Mutex<Vec<String>>>, delay_ms: u64) -> Self {
            let mut controller = Self::new(adapter_id, events);
            controller.delay_ms = delay_ms;
            controller
        }
    }

    #[async_trait]
    impl ResourceAdapterController for RecordingController {
        fn adapter_id(&self) -> &str {
            &self.adapter_id
        }

        async fn transition(
            &self,
            operation: ResourceAdapterOperation,
            _profile_id: Option<&str>,
            _reason: Option<&str>,
        ) -> oclive_kernel_types::Result<ResourceAdapterControllerOutcome> {
            let in_flight = self.in_flight.fetch_add(1, Ordering::AcqRel) + 1;
            self.max_in_flight.fetch_max(in_flight, Ordering::AcqRel);
            self.events
                .lock()
                .push(format!("{}:{operation:?}", self.adapter_id));
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }
            self.in_flight.fetch_sub(1, Ordering::AcqRel);
            if self.fail_operations.contains(&operation) {
                return Err(AppError::RemoteServiceUnavailable(format!(
                    "{}_failed",
                    self.adapter_id
                )));
            }
            Ok(ResourceAdapterControllerOutcome {
                already_in_state: false,
                recovery_scheduled: operation == ResourceAdapterOperation::Resume,
            })
        }
    }

    #[async_trait]
    impl ResourceAdapterController for SnapshotController {
        fn adapter_id(&self) -> &str {
            &self.adapter_id
        }

        async fn transition(
            &self,
            operation: ResourceAdapterOperation,
            _profile_id: Option<&str>,
            _reason: Option<&str>,
        ) -> oclive_kernel_types::Result<ResourceAdapterControllerOutcome> {
            self.events
                .lock()
                .push(format!("{}:{operation:?}", self.adapter_id));
            let free_mib = match operation {
                ResourceAdapterOperation::Suspend | ResourceAdapterOperation::Unload => {
                    self.released_free_mib
                }
                ResourceAdapterOperation::Resume | ResourceAdapterOperation::Start => {
                    self.occupied_free_mib
                }
                ResourceAdapterOperation::Observe | ResourceAdapterOperation::Release => self
                    .snapshot
                    .read()
                    .gpu_devices
                    .first()
                    .map_or(0, |device| device.free_mib),
            };
            let mut snapshot = self.snapshot.write();
            if let Some(device) = snapshot.gpu_devices.first_mut() {
                device.free_mib = free_mib;
                device.used_mib = device.total_mib.saturating_sub(free_mib);
            }
            Ok(ResourceAdapterControllerOutcome {
                already_in_state: false,
                recovery_scheduled: false,
            })
        }
    }

    #[async_trait]
    impl ResourceSnapshotSource for FixedSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            self.0.clone()
        }
    }

    #[async_trait]
    impl ResourceSnapshotSource for MutableSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            self.0.read().clone()
        }
    }

    #[async_trait]
    impl ResourceSnapshotSource for GatedSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            if self.calls.fetch_add(1, Ordering::AcqRel) == 0 {
                self.first_started.notify_one();
                self.release_first.notified().await;
            }
            self.snapshot.clone()
        }
    }

    fn snapshot(free_mib: u64) -> ResourceSnapshot {
        ResourceSnapshot {
            captured_at_ms: 1,
            source: "test".into(),
            available: true,
            gpu_devices: vec![GpuDeviceSnapshot {
                device_index: 0,
                name: "test-gpu".into(),
                total_mib: 8192,
                free_mib,
                used_mib: 8192 - free_mib,
            }],
            system_memory: None,
            cpu: None,
            reason_codes: Vec::new(),
        }
    }

    fn snapshot_with_host_resources(
        free_mib: u64,
        available_ram_mib: u64,
        logical_cores: u16,
    ) -> ResourceSnapshot {
        let mut snapshot = snapshot(free_mib);
        snapshot.system_memory = Some(SystemMemorySnapshot {
            total_mib: 32_768,
            available_mib: available_ram_mib,
            used_mib: 32_768_u64.saturating_sub(available_ram_mib),
        });
        snapshot.cpu = Some(CpuSnapshot {
            logical_cores,
            physical_cores: Some(logical_cores / 2),
        });
        snapshot
    }

    fn request(adapter: &str, workload: &str, reservation_mib: u64) -> ResourceAdmissionRequest {
        ResourceAdmissionRequest {
            adapter_id: adapter.into(),
            workload_id: workload.into(),
            profile_id: None,
            gpu_device_index: None,
            reservation_mib,
            ram_reservation_mib: 0,
            cpu_thread_reservation: 0,
            priority: ResourcePriority::BackgroundWarmup,
            control_mode: ResourceControlMode::Managed,
            admission_mode: ResourceAdmissionMode::Enforced,
        }
    }

    fn registered_adapter(adapter_id: &str) -> ResourceAdapterDescriptor {
        ResourceAdapterDescriptor {
            adapter_id: adapter_id.into(),
            kind: ResourceAdapterKind::Runtime,
            domain: ResourceAdapterDomain::Voice,
            provider_id: Some("builtin.test".into()),
            control_mode: ResourceControlMode::Managed,
            profiles: vec![ResourceOperatingProfile {
                profile_id: "full".into(),
                quality_rank: 100,
                execution_target: ResourceExecutionTarget::Gpu,
                estimated_reservation_mib: None,
                estimated_ram_mib: None,
                estimated_cpu_threads: None,
                requires_restart: true,
                coordinator_selectable: true,
            }],
            lifecycle_operations: vec![
                ResourceAdapterOperation::Start,
                ResourceAdapterOperation::Resume,
                ResourceAdapterOperation::Suspend,
                ResourceAdapterOperation::Unload,
            ],
            residency_modes: vec![
                ResourceResidencyMode::Resident,
                ResourceResidencyMode::Unloaded,
            ],
            automatic_preemption: Some(ResourceAdapterOperation::Suspend),
        }
    }

    #[tokio::test]
    async fn pending_reservations_are_atomic_and_prevent_double_admission() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy {
                gpu_safety_reserve_mib: 768,
                ..ResourceCoordinatorPolicy::default()
            },
            Arc::new(FixedSnapshot(snapshot(3000))),
        );
        let first = coordinator.admit(request("voice", "a", 1500)).await;
        assert_eq!(first.decision, ResourceAdmissionDecision::Granted);
        let second = coordinator.admit(request("llm", "b", 1500)).await;
        assert_eq!(second.decision, ResourceAdmissionDecision::Denied);
        assert_eq!(second.reason_codes, vec!["insufficient_gpu_headroom"]);
    }

    #[tokio::test]
    async fn critical_pressure_has_a_stable_reason_code() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(500))),
        );
        let diagnostics = coordinator.refresh().await;
        assert_eq!(
            diagnostics.state,
            ResourceCoordinationDiagnosticState::Blocked
        );
        assert_eq!(diagnostics.reason_codes, vec!["gpu_headroom_critical"]);
    }

    #[tokio::test]
    async fn diagnostics_include_default_non_executing_scheduling_intent() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(6000))),
        );
        let diagnostics = coordinator.refresh().await;
        assert_eq!(
            diagnostics.scheduling.state,
            oclive_kernel_types::ResourceSchedulingIntentState::Ready
        );
        assert_eq!(
            diagnostics.scheduling.intent.strategy,
            oclive_kernel_types::ResourceSchedulingStrategy::CompatibilityFirst
        );
        assert!(diagnostics.scheduling.intent.commands.is_empty());
    }

    #[tokio::test]
    async fn activation_reuse_and_release_follow_one_lease_lifecycle() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(6000))),
        );
        let first = coordinator.admit(request("voice", "cosy", 1792)).await;
        let lease = first.lease.expect("lease");
        assert!(coordinator.has_adapter_lease("voice"));
        assert!(!coordinator.has_active_adapter("voice"));
        assert!(coordinator.activate(&lease.lease_id, Some(1400)));
        assert!(coordinator.has_active_adapter("voice"));
        assert_eq!(
            coordinator.record_adapter_reason("voice", "resource_release_unconfirmed"),
            1
        );
        assert_eq!(
            coordinator.record_adapter_reason("voice", "resource_release_unconfirmed"),
            0
        );
        assert!(coordinator.adapter_has_reason("voice", "resource_release_unconfirmed"));
        assert!(!coordinator.adapter_has_reason("voice", "different_reason"));
        let reused = coordinator.admit(request("voice", "cosy", 1792)).await;
        assert_eq!(reused.decision, ResourceAdmissionDecision::Reused);
        assert_eq!(
            reused.lease.as_ref().map(|item| item.state),
            Some(ResourceLeaseState::Active)
        );
        assert_eq!(
            reused.lease.as_ref().and_then(|item| item.expires_at_ms),
            None
        );
        assert_eq!(
            reused.lease.as_ref().expect("reused lease").reason_codes,
            vec!["resource_release_unconfirmed"]
        );
        assert!(coordinator.release(&lease.lease_id));
        assert!(!coordinator.has_adapter_lease("voice"));
        assert!(coordinator.diagnostics_snapshot().leases.is_empty());
    }

    #[tokio::test]
    async fn changed_resource_budget_never_reuses_a_stale_lease() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot_with_host_resources(
                8_000, 16_000, 16,
            ))),
        );
        let mut first_request = request("render", "live2d", 512);
        first_request.ram_reservation_mib = 1_024;
        first_request.cpu_thread_reservation = 2;
        let first = coordinator.admit(first_request).await;
        assert_eq!(first.decision, ResourceAdmissionDecision::Granted);
        let first_lease_id = first.lease.expect("first lease").lease_id;

        let mut larger_request = request("render", "live2d", 768);
        larger_request.ram_reservation_mib = 2_048;
        larger_request.cpu_thread_reservation = 4;
        larger_request.priority = ResourcePriority::ForegroundMedia;
        let second = coordinator.admit(larger_request).await;

        assert_eq!(second.decision, ResourceAdmissionDecision::Granted);
        assert_ne!(
            second.lease.as_ref().expect("second lease").lease_id,
            first_lease_id
        );
        assert_eq!(coordinator.diagnostics_snapshot().leases.len(), 2);
    }

    #[tokio::test]
    async fn unavailable_snapshot_obeys_unverified_policy() {
        let source = Arc::new(FixedSnapshot(ResourceSnapshot::unavailable(
            "test",
            "telemetry_missing",
        )));
        let permissive =
            ResourceCoordinator::new(ResourceCoordinatorPolicy::default(), source.clone());
        let granted = permissive.admit(request("voice", "cosy", 1792)).await;
        assert_eq!(
            granted.decision,
            ResourceAdmissionDecision::GrantedUnverified
        );

        let strict = ResourceCoordinator::new(
            ResourceCoordinatorPolicy {
                allow_unverified_admission: false,
                ..ResourceCoordinatorPolicy::default()
            },
            source,
        );
        let denied = strict.admit(request("voice", "cosy", 1792)).await;
        assert_eq!(denied.decision, ResourceAdmissionDecision::Denied);
    }

    #[tokio::test]
    async fn ram_and_cpu_are_enforced_independently_of_gpu() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot_with_host_resources(6000, 2000, 8))),
        );
        let mut ram_heavy = request("render", "ram", 0);
        ram_heavy.ram_reservation_mib = 1_100;
        let denied_ram = coordinator.admit(ram_heavy).await;
        assert_eq!(denied_ram.decision, ResourceAdmissionDecision::Denied);
        assert_eq!(
            denied_ram.reason_codes,
            vec!["insufficient_system_memory_headroom"]
        );

        let mut cpu_heavy = request("render", "cpu", 0);
        cpu_heavy.cpu_thread_reservation = 8;
        let denied_cpu = coordinator.admit(cpu_heavy).await;
        assert_eq!(denied_cpu.decision, ResourceAdmissionDecision::Denied);
        assert_eq!(
            denied_cpu.reason_codes,
            vec!["insufficient_cpu_thread_headroom"]
        );

        let mut compatible = request("render", "compatible", 0);
        compatible.ram_reservation_mib = 512;
        compatible.cpu_thread_reservation = 2;
        assert_eq!(
            coordinator.admit(compatible).await.decision,
            ResourceAdmissionDecision::Granted
        );
    }

    #[tokio::test]
    async fn fair_queue_prefers_foreground_and_cleans_cancelled_waiters() {
        let first_started = Arc::new(tokio::sync::Notify::new());
        let release_first = Arc::new(tokio::sync::Notify::new());
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(GatedSnapshot {
                snapshot: snapshot(6000),
                calls: AtomicUsize::new(0),
                first_started: Arc::clone(&first_started),
                release_first: Arc::clone(&release_first),
            }),
        ));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let initial = {
            let coordinator = Arc::clone(&coordinator);
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = coordinator.admit(request("initial", "initial", 0)).await;
                tx.send(("initial", result.decision)).unwrap();
            })
        };
        first_started.notified().await;

        let cancelled = {
            let coordinator = Arc::clone(&coordinator);
            tokio::spawn(async move {
                coordinator
                    .admit(request("cancelled", "cancelled", 0))
                    .await
            })
        };
        tokio::task::yield_now().await;
        cancelled.abort();
        let _ = cancelled.await;

        let background = {
            let coordinator = Arc::clone(&coordinator);
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = coordinator
                    .admit(request("background", "background", 0))
                    .await;
                tx.send(("background", result.decision)).unwrap();
            })
        };
        let foreground = {
            let coordinator = Arc::clone(&coordinator);
            tokio::spawn(async move {
                let mut request = request("foreground", "foreground", 0);
                request.priority = ResourcePriority::ForegroundInteractive;
                let result = coordinator.admit(request).await;
                tx.send(("foreground", result.decision)).unwrap();
            })
        };
        for _ in 0..100 {
            if coordinator
                .diagnostics_snapshot()
                .admission_queue
                .queued
                .len()
                == 2
            {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(
            coordinator
                .diagnostics_snapshot()
                .admission_queue
                .queued
                .len(),
            2
        );
        release_first.notify_one();
        let mut order = Vec::new();
        for _ in 0..3 {
            order.push(rx.recv().await.unwrap().0);
        }
        initial.await.unwrap();
        background.await.unwrap();
        foreground.await.unwrap();
        assert_eq!(order, ["initial", "foreground", "background"]);
        assert!(coordinator
            .diagnostics_snapshot()
            .admission_queue
            .queued
            .is_empty());
    }

    #[tokio::test]
    async fn admission_queue_timeout_removes_waiter_without_disturbing_owner() {
        let queue = AdmissionQueue::new(2_000);
        let first_request = request("first", "first", 0);
        let (permit, _) = queue
            .acquire(&first_request, 1_000)
            .await
            .expect("first admission owns the queue");

        let timed_out = queue.acquire(&request("second", "second", 0), 25).await;
        assert!(timed_out.is_none());
        let diagnostics = queue.diagnostics();
        assert_eq!(diagnostics.active_ticket_id, Some(1));
        assert!(diagnostics.queued.is_empty());

        drop(permit);
        assert_eq!(
            queue.diagnostics(),
            ResourceAdmissionQueueDiagnostics::default()
        );
    }

    #[test]
    fn queue_aging_eventually_protects_old_resident_work() {
        let now_ms = now_epoch_ms();
        let queued = [
            (
                1,
                ResourceAdmissionQueueItem {
                    ticket_id: 1,
                    adapter_id: "old".into(),
                    workload_id: "old".into(),
                    priority: ResourcePriority::Resident,
                    enqueued_at_ms: now_ms.saturating_sub(10_000),
                },
            ),
            (
                2,
                ResourceAdmissionQueueItem {
                    ticket_id: 2,
                    adapter_id: "new".into(),
                    workload_id: "new".into(),
                    priority: ResourcePriority::ForegroundInteractive,
                    enqueued_at_ms: now_ms,
                },
            ),
        ]
        .into_iter()
        .collect();
        assert_eq!(select_queue_candidate(&queued, 2_000), Some(1));
    }

    #[tokio::test]
    async fn higher_priority_render_admission_preempts_and_restores_managed_llm() {
        let shared_snapshot =
            Arc::new(RwLock::new(snapshot_with_host_resources(3_000, 16_000, 16)));
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(MutableSnapshot(Arc::clone(&shared_snapshot))),
        );
        coordinator
            .register_adapter(registered_adapter("builtin.llm.test"))
            .unwrap();
        let mut render = registered_adapter("com.example.live2d.render");
        render.domain = ResourceAdapterDomain::Render;
        render.provider_id = Some("com.example.live2d".into());
        coordinator
            .register_third_party_adapter(ResourceAdapterRegistration {
                source: ResourceAdapterRegistrationSource::HostExtension,
                source_id: "com.example.live2d".into(),
                descriptor: render,
            })
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        coordinator
            .register_adapter_controller(Arc::new(SnapshotController {
                adapter_id: "builtin.llm.test".into(),
                snapshot: Arc::clone(&shared_snapshot),
                occupied_free_mib: 1_000,
                released_free_mib: 3_000,
                events: Arc::clone(&events),
            }))
            .unwrap();
        coordinator
            .register_adapter_transition_grant(
                "com.example.live2d.render",
                "builtin.llm.test",
                [
                    ResourceAdapterOperation::Suspend,
                    ResourceAdapterOperation::Resume,
                ],
            )
            .unwrap();

        let mut llm_request = request("builtin.llm.test", "resident", 1_500);
        llm_request.profile_id = Some("full".into());
        llm_request.priority = ResourcePriority::Resident;
        let llm = coordinator.admit(llm_request).await;
        let llm_lease = llm.lease.expect("llm lease");
        assert!(coordinator.activate(&llm_lease.lease_id, Some(1_500)));
        {
            let mut snapshot = shared_snapshot.write();
            snapshot.gpu_devices[0].free_mib = 1_000;
            snapshot.gpu_devices[0].used_mib = 7_192;
        }

        let mut render_request = request("com.example.live2d.render", "foreground-render", 1_000);
        render_request.profile_id = Some("full".into());
        render_request.priority = ResourcePriority::ForegroundInteractive;
        let admitted = coordinator.admit(render_request).await;
        assert_eq!(admitted.decision, ResourceAdmissionDecision::Granted);
        assert_eq!(admitted.preempted_adapters.len(), 1);
        assert_eq!(
            admitted.preempted_adapters[0].adapter_id,
            "builtin.llm.test"
        );
        assert_eq!(*events.lock(), vec!["builtin.llm.test:Suspend"]);
        assert!(!coordinator.has_adapter_lease("builtin.llm.test"));

        coordinator
            .restore_preempted_adapters("com.example.live2d.render", &admitted.preempted_adapters)
            .await
            .unwrap();
        assert_eq!(
            *events.lock(),
            vec!["builtin.llm.test:Suspend", "builtin.llm.test:Resume"]
        );
    }

    #[test]
    fn third_party_controller_binding_requires_the_registered_owner() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(6000))),
        );
        let mut render = registered_adapter("com.example.live2d.render");
        render.domain = ResourceAdapterDomain::Render;
        render.provider_id = Some("com.example.live2d".into());
        coordinator
            .register_third_party_adapter(ResourceAdapterRegistration {
                source: ResourceAdapterRegistrationSource::HostExtension,
                source_id: "com.example.live2d".into(),
                descriptor: render,
            })
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        assert!(coordinator
            .register_third_party_adapter_controller(
                "com.other",
                Arc::new(RecordingController::new(
                    "com.example.live2d.render",
                    Arc::clone(&events),
                )),
            )
            .unwrap_err()
            .contains("is not owned"));
        coordinator
            .register_third_party_adapter_controller(
                "com.example.live2d",
                Arc::new(RecordingController::new(
                    "com.example.live2d.render",
                    events,
                )),
            )
            .unwrap();
    }

    #[tokio::test]
    async fn admission_uses_requested_gpu_instead_of_best_unrelated_device() {
        let multi_gpu = ResourceSnapshot {
            captured_at_ms: 1,
            source: "test".into(),
            available: true,
            gpu_devices: vec![
                GpuDeviceSnapshot {
                    device_index: 0,
                    name: "busy-default".into(),
                    total_mib: 8192,
                    free_mib: 500,
                    used_mib: 7692,
                },
                GpuDeviceSnapshot {
                    device_index: 1,
                    name: "free-secondary".into(),
                    total_mib: 8192,
                    free_mib: 4000,
                    used_mib: 4192,
                },
            ],
            system_memory: None,
            cpu: None,
            reason_codes: Vec::new(),
        };
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(multi_gpu)),
        );
        let default_device = coordinator.admit(request("voice", "gpu0", 768)).await;
        assert_eq!(default_device.decision, ResourceAdmissionDecision::Denied);

        let mut secondary_request = request("voice", "gpu1", 768);
        secondary_request.gpu_device_index = Some(1);
        let secondary = coordinator.admit(secondary_request).await;
        assert_eq!(secondary.decision, ResourceAdmissionDecision::Granted);
    }

    #[tokio::test]
    async fn registered_profile_is_joined_to_lease_and_unknown_profile_is_rejected() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(4096))),
        );
        coordinator
            .register_adapter(registered_adapter("voice"))
            .unwrap();

        let mut unknown = request("voice", "bad", 768);
        unknown.profile_id = Some("missing".into());
        let denied = coordinator.admit(unknown).await;
        assert_eq!(denied.decision, ResourceAdmissionDecision::Denied);
        assert_eq!(denied.reason_codes, vec!["resource_profile_unregistered"]);

        let mut known = request("voice", "good", 768);
        known.profile_id = Some("full".into());
        let granted = coordinator.admit(known).await;
        assert_eq!(granted.decision, ResourceAdmissionDecision::Granted);
        let diagnostics = coordinator.diagnostics_snapshot();
        assert_eq!(diagnostics.adapters.len(), 1);
        assert_eq!(
            diagnostics.adapters[0].current_profile_id.as_deref(),
            Some("full")
        );
        assert_eq!(diagnostics.leases[0].reservation_mib, 768);
    }

    #[tokio::test]
    async fn stale_transition_is_rejected_before_controller_call() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(4096))),
        );
        coordinator
            .register_adapter(registered_adapter("target"))
            .unwrap();
        coordinator
            .register_adapter(registered_adapter("requester"))
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        coordinator
            .register_adapter_controller(Arc::new(RecordingController::new(
                "target",
                Arc::clone(&events),
            )))
            .unwrap();
        coordinator
            .register_adapter_transition_grant(
                "requester",
                "target",
                [ResourceAdapterOperation::Start],
            )
            .unwrap();
        let stale_revision = coordinator.state_revision().saturating_sub(1);
        let error = coordinator
            .transition_adapter(&ResourceAdapterTransitionRequest {
                adapter_id: "target".into(),
                operation: ResourceAdapterOperation::Start,
                requested_by_adapter_id: "requester".into(),
                profile_id: Some("full".into()),
                expected_revision: Some(stale_revision),
                reason: None,
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("resource_plan_stale_revision"));
        assert!(events.lock().is_empty());
    }

    #[tokio::test]
    async fn transitions_for_one_adapter_are_serialized() {
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(4096))),
        ));
        coordinator
            .register_adapter(registered_adapter("target"))
            .unwrap();
        coordinator
            .register_adapter(registered_adapter("requester"))
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let controller = Arc::new(RecordingController::delayed(
            "target",
            Arc::clone(&events),
            25,
        ));
        coordinator
            .register_adapter_controller(controller.clone())
            .unwrap();
        coordinator
            .register_adapter_transition_grant(
                "requester",
                "target",
                [ResourceAdapterOperation::Start],
            )
            .unwrap();
        let request = ResourceAdapterTransitionRequest {
            adapter_id: "target".into(),
            operation: ResourceAdapterOperation::Start,
            requested_by_adapter_id: "requester".into(),
            profile_id: Some("full".into()),
            expected_revision: None,
            reason: None,
        };
        let mut tasks = Vec::new();
        for _ in 0..32 {
            let task_coordinator = Arc::clone(&coordinator);
            let task_request = request.clone();
            tasks.push(tokio::spawn(async move {
                task_coordinator.transition_adapter(&task_request).await
            }));
        }
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        assert_eq!(controller.max_in_flight.load(Ordering::Acquire), 1);
        assert_eq!(events.lock().len(), 32);
    }

    #[tokio::test]
    async fn failed_candidate_step_rolls_back_completed_steps_in_reverse() {
        let scheduling = ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::Custom,
            primary_adapter_id: None,
            commands: ["first", "second"]
                .into_iter()
                .map(|adapter_id| ResourceSchedulingCommand::Residency {
                    adapter_id: adapter_id.into(),
                    mode: ResourceResidencyPreference::Resident,
                })
                .collect(),
        };
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy {
                scheduling,
                ..ResourceCoordinatorPolicy::default()
            },
            Arc::new(FixedSnapshot(snapshot(4096))),
        );
        for adapter_id in ["first", "second"] {
            coordinator
                .register_adapter(registered_adapter(adapter_id))
                .unwrap();
        }
        let events = Arc::new(Mutex::new(Vec::new()));
        coordinator
            .register_adapter_controller(Arc::new(RecordingController::new(
                "first",
                Arc::clone(&events),
            )))
            .unwrap();
        coordinator
            .register_adapter_controller(Arc::new(RecordingController::failing(
                "second",
                Arc::clone(&events),
                ResourceAdapterOperation::Start,
            )))
            .unwrap();
        let plan = coordinator.diagnostics_snapshot().candidate_plan;
        assert!(plan.executable);
        assert_eq!(plan.transitions.len(), 2);
        let error = coordinator.execute_candidate_plan(&plan).await.unwrap_err();
        assert!(error.to_string().contains("rollback_confirmed"));
        assert_eq!(
            *events.lock(),
            vec!["first:Start", "second:Start", "first:Suspend"]
        );
    }

    #[tokio::test]
    async fn transition_validation_reports_stable_control_plane_reasons() {
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(4096))),
        );
        let mut target = registered_adapter("target");
        target.profiles.push(ResourceOperatingProfile {
            profile_id: "manual".into(),
            quality_rank: 10,
            execution_target: ResourceExecutionTarget::Gpu,
            estimated_reservation_mib: Some(512),
            estimated_ram_mib: Some(256),
            estimated_cpu_threads: Some(1),
            requires_restart: true,
            coordinator_selectable: false,
        });
        coordinator.register_adapter(target).unwrap();
        coordinator
            .register_adapter(registered_adapter("requester"))
            .unwrap();
        coordinator
            .register_adapter(registered_adapter("uncontrolled"))
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        coordinator
            .register_adapter_controller(Arc::new(RecordingController::new(
                "target",
                Arc::clone(&events),
            )))
            .unwrap();
        coordinator
            .register_adapter_transition_grant(
                "requester",
                "target",
                [ResourceAdapterOperation::Start],
            )
            .unwrap();

        let base = ResourceAdapterTransitionRequest {
            adapter_id: "target".into(),
            operation: ResourceAdapterOperation::Start,
            requested_by_adapter_id: "requester".into(),
            profile_id: Some("full".into()),
            expected_revision: None,
            reason: None,
        };
        let cases = [
            (
                ResourceAdapterTransitionRequest {
                    requested_by_adapter_id: "missing".into(),
                    ..base.clone()
                },
                "resource_transition_requester_unregistered",
            ),
            (
                ResourceAdapterTransitionRequest {
                    operation: ResourceAdapterOperation::Release,
                    ..base.clone()
                },
                "resource_transition_operation_unsupported",
            ),
            (
                ResourceAdapterTransitionRequest {
                    profile_id: Some("missing".into()),
                    ..base.clone()
                },
                "resource_profile_unregistered",
            ),
            (
                ResourceAdapterTransitionRequest {
                    profile_id: Some("manual".into()),
                    ..base.clone()
                },
                "resource_profile_not_coordinator_selectable",
            ),
            (
                ResourceAdapterTransitionRequest {
                    adapter_id: "uncontrolled".into(),
                    ..base
                },
                "resource_transition_not_authorized",
            ),
        ];
        for (request, reason) in cases {
            let error = coordinator
                .transition_adapter(&request)
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{error}");
        }
        assert!(events.lock().is_empty());
    }

    #[tokio::test]
    async fn transition_soak_keeps_revision_monotonic_without_losing_calls() {
        let cycles = std::env::var("OCLIVE_RESOURCE_SOAK_CYCLES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(512)
            .clamp(1, 1_000_000);
        let coordinator = ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedSnapshot(snapshot(4096))),
        );
        coordinator
            .register_adapter(registered_adapter("target"))
            .unwrap();
        coordinator
            .register_adapter(registered_adapter("requester"))
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        coordinator
            .register_adapter_controller(Arc::new(RecordingController::new(
                "target",
                Arc::clone(&events),
            )))
            .unwrap();
        coordinator
            .register_adapter_transition_grant(
                "requester",
                "target",
                [
                    ResourceAdapterOperation::Start,
                    ResourceAdapterOperation::Suspend,
                ],
            )
            .unwrap();
        let initial_revision = coordinator.state_revision();
        for index in 0..cycles {
            let operation = if index % 2 == 0 {
                ResourceAdapterOperation::Start
            } else {
                ResourceAdapterOperation::Suspend
            };
            let response = coordinator
                .transition_adapter(&ResourceAdapterTransitionRequest {
                    adapter_id: "target".into(),
                    operation,
                    requested_by_adapter_id: "requester".into(),
                    profile_id: (operation == ResourceAdapterOperation::Start)
                        .then(|| "full".into()),
                    expected_revision: None,
                    reason: Some("deterministic in-process soak".into()),
                })
                .await
                .unwrap();
            assert_eq!(response.state_revision, initial_revision + index + 1);
        }
        assert_eq!(events.lock().len(), cycles as usize);
        assert_eq!(coordinator.state_revision(), initial_revision + cycles);
        eprintln!(
            "resource_control_plane_soak cycles={cycles} initial_revision={initial_revision} final_revision={}",
            coordinator.state_revision()
        );
    }
}
