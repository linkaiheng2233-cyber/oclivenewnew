//! Host-owned resource admission, leases, pressure, and diagnostics.
//!
//! This is a control-plane facility. It does not execute model, voice, or
//! rendering business data paths; concrete adapters remain responsible for
//! starting and stopping their own runtimes.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use oclive_kernel_contracts::ResourceSnapshotSource;
use oclive_kernel_types::{
    ResourceAdapterDescriptor, ResourceAdmissionDecision, ResourceAdmissionMode,
    ResourceAdmissionRequest, ResourceAdmissionResult, ResourceCoordinationDiagnosticState,
    ResourceCoordinationDiagnostics, ResourceCoordinatorPolicy, ResourceLeaseDiagnostic,
    ResourceLeaseState, ResourcePressureLevel, ResourcePriority, ResourceSnapshot,
    RESOURCE_COORDINATION_SCHEMA_VERSION,
};
use parking_lot::Mutex;

use super::resource_adapter_registry::ResourceAdapterRegistry;

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

pub struct ResourceCoordinator {
    policy: ResourceCoordinatorPolicy,
    snapshot_source: Arc<dyn ResourceSnapshotSource>,
    state: Mutex<CoordinatorState>,
    adapter_registry: ResourceAdapterRegistry,
    next_lease_id: AtomicU64,
    adapter_operation_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
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
        Self {
            policy,
            snapshot_source,
            state: Mutex::new(CoordinatorState::default()),
            adapter_registry: ResourceAdapterRegistry::new(),
            next_lease_id: AtomicU64::new(1),
            adapter_operation_locks: DashMap::new(),
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
        self.adapter_registry.register(descriptor)
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
        prune_expired(&mut state.leases, now_ms);
        state.last_pressure = pressure_for(
            &snapshot,
            self.policy.gpu_safety_reserve_mib,
            configured_gpu_device_index(),
        );
        state.last_reason_codes = pressure_reason_codes(&snapshot, state.last_pressure);
        state.last_snapshot = snapshot;
        diagnostics_from_state(&self.policy, &self.adapter_registry, &state)
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
            priority,
            control_mode: oclive_kernel_types::ResourceControlMode::ObserveOnly,
            state: ResourceLeaseState::Active,
            granted_at_ms: now_ms,
            expires_at_ms: Some(now_ms.saturating_add(self.policy.active_lease_ttl_ms)),
            reason_codes: vec!["external_activity_observed".into()],
        };
        self.state.lock().leases.insert(lease_id.clone(), lease);
        lease_id
    }

    pub async fn admit(&self, request: ResourceAdmissionRequest) -> ResourceAdmissionResult {
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
                    reason_codes: vec!["resource_profile_unregistered".into()],
                };
            }
        }
        let snapshot = self.snapshot_source.snapshot().await;
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        prune_expired(&mut state.leases, now_ms);
        state.last_pressure = pressure_for(
            &snapshot,
            self.policy.gpu_safety_reserve_mib,
            request.gpu_device_index,
        );
        state.last_snapshot = snapshot.clone();

        let reused_lease = state
            .leases
            .values_mut()
            .find(|lease| {
                lease.adapter_id == request.adapter_id
                    && lease.workload_id == request.workload_id
                    && lease.profile_id == request.profile_id
                    && lease.gpu_device_index == request.gpu_device_index
            })
            .map(|lease| {
                let ttl = match lease.state {
                    ResourceLeaseState::Reserved => self.policy.pending_lease_ttl_ms,
                    ResourceLeaseState::Active => self.policy.active_lease_ttl_ms,
                };
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
            return ResourceAdmissionResult {
                decision: ResourceAdmissionDecision::Reused,
                lease: Some(lease),
                snapshot,
                pressure: state.last_pressure,
                reason_codes: Vec::new(),
            };
        }

        let pending_mib = state
            .leases
            .values()
            .filter(|lease| {
                lease.state == ResourceLeaseState::Reserved
                    && lease.gpu_device_index == request.gpu_device_index
            })
            .map(|lease| lease.reservation_mib)
            .sum::<u64>();
        let selected_free_mib = free_mib_for_device(&snapshot, request.gpu_device_index);
        let required_mib = self
            .policy
            .gpu_safety_reserve_mib
            .saturating_add(pending_mib)
            .saturating_add(request.reservation_mib);

        let (decision, reason_codes) = if !snapshot.available {
            if request.admission_mode == ResourceAdmissionMode::ObserveOnly
                || self.policy.allow_unverified_admission
            {
                (
                    ResourceAdmissionDecision::GrantedUnverified,
                    vec!["gpu_snapshot_unavailable".to_string()],
                )
            } else {
                (
                    ResourceAdmissionDecision::Denied,
                    vec!["gpu_snapshot_unavailable".to_string()],
                )
            }
        } else if selected_free_mib.is_none() {
            if request.admission_mode == ResourceAdmissionMode::ObserveOnly {
                (
                    ResourceAdmissionDecision::GrantedUnverified,
                    vec!["gpu_device_unavailable".to_string()],
                )
            } else {
                (
                    ResourceAdmissionDecision::Denied,
                    vec!["gpu_device_unavailable".to_string()],
                )
            }
        } else if selected_free_mib.is_some_and(|free_mib| free_mib >= required_mib) {
            (ResourceAdmissionDecision::Granted, Vec::new())
        } else if request.admission_mode == ResourceAdmissionMode::ObserveOnly {
            (
                ResourceAdmissionDecision::GrantedUnverified,
                vec!["observe_only_gpu_pressure".to_string()],
            )
        } else {
            (
                ResourceAdmissionDecision::Denied,
                vec!["insufficient_gpu_headroom".to_string()],
            )
        };

        state.last_reason_codes = reason_codes.clone();
        if decision == ResourceAdmissionDecision::Denied {
            return ResourceAdmissionResult {
                decision,
                lease: None,
                snapshot,
                pressure: state.last_pressure,
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
            priority: request.priority,
            control_mode: request.control_mode,
            state: ResourceLeaseState::Reserved,
            granted_at_ms: now_ms,
            expires_at_ms: Some(now_ms.saturating_add(self.policy.pending_lease_ttl_ms)),
            reason_codes: reason_codes.clone(),
        };
        state.leases.insert(lease_id, lease.clone());
        ResourceAdmissionResult {
            decision,
            lease: Some(lease),
            snapshot,
            pressure: state.last_pressure,
            reason_codes,
        }
    }

    pub fn activate(&self, lease_id: &str, actual_mib: Option<u64>) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        let Some(lease) = state.leases.get_mut(lease_id) else {
            return false;
        };
        lease.state = ResourceLeaseState::Active;
        lease.actual_mib = actual_mib.unwrap_or(lease.reservation_mib);
        lease.expires_at_ms =
            if lease.control_mode == oclive_kernel_types::ResourceControlMode::Managed {
                None
            } else {
                Some(now_ms.saturating_add(self.policy.active_lease_ttl_ms))
            };
        true
    }

    pub fn release(&self, lease_id: &str) -> bool {
        self.state.lock().leases.remove(lease_id).is_some()
    }

    pub fn release_workload(&self, adapter_id: &str, workload_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id || lease.workload_id != workload_id);
        before.saturating_sub(state.leases.len())
    }

    pub fn release_adapter(&self, adapter_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id);
        before.saturating_sub(state.leases.len())
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
        updated
    }

    #[must_use]
    pub fn has_active_priority(&self, minimum: ResourcePriority) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        prune_expired(&mut state.leases, now_ms);
        state
            .leases
            .values()
            .any(|lease| lease.state == ResourceLeaseState::Active && lease.priority >= minimum)
    }

    #[must_use]
    pub fn has_active_adapter(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        prune_expired(&mut state.leases, now_ms);
        state.leases.values().any(|lease| {
            lease.state == ResourceLeaseState::Active && lease.adapter_id == adapter_id
        })
    }

    #[must_use]
    /// Whether the adapter has a reserved or active lease after TTL pruning.
    pub fn has_adapter_lease(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        prune_expired(&mut state.leases, now_ms);
        state
            .leases
            .values()
            .any(|lease| lease.adapter_id == adapter_id)
    }

    #[must_use]
    pub fn diagnostics_snapshot(&self) -> ResourceCoordinationDiagnostics {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        prune_expired(&mut state.leases, now_ms);
        diagnostics_from_state(&self.policy, &self.adapter_registry, &state)
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

fn prune_expired(leases: &mut BTreeMap<String, ResourceLeaseDiagnostic>, now_ms: u64) {
    leases.retain(|_, lease| {
        lease
            .expires_at_ms
            .is_none_or(|expires_at| expires_at > now_ms)
    });
}

fn pressure_for(
    snapshot: &ResourceSnapshot,
    safety_reserve_mib: u64,
    gpu_device_index: Option<u32>,
) -> ResourcePressureLevel {
    if !snapshot.available || snapshot.gpu_devices.is_empty() {
        return ResourcePressureLevel::Unknown;
    }
    let selected_free_mib = free_mib_for_device(snapshot, gpu_device_index).unwrap_or(0);
    if selected_free_mib <= safety_reserve_mib {
        ResourcePressureLevel::Critical
    } else if selected_free_mib <= safety_reserve_mib.saturating_mul(2) {
        ResourcePressureLevel::Elevated
    } else {
        ResourcePressureLevel::Normal
    }
}

fn pressure_reason_codes(
    snapshot: &ResourceSnapshot,
    pressure: ResourcePressureLevel,
) -> Vec<String> {
    let mut reasons = snapshot.reason_codes.clone();
    match pressure {
        ResourcePressureLevel::Unknown if reasons.is_empty() => {
            reasons.push("gpu_pressure_unknown".into());
        }
        ResourcePressureLevel::Elevated => reasons.push("gpu_headroom_elevated".into()),
        ResourcePressureLevel::Critical => reasons.push("gpu_headroom_critical".into()),
        ResourcePressureLevel::Unknown | ResourcePressureLevel::Normal => {}
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

fn diagnostics_from_state(
    policy: &ResourceCoordinatorPolicy,
    adapter_registry: &ResourceAdapterRegistry,
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
    let mut reason_codes = state.last_reason_codes.clone();
    for reason in registry_reason_codes {
        if !reason_codes.contains(&reason) {
            reason_codes.push(reason);
        }
    }
    ResourceCoordinationDiagnostics {
        schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
        state: coordination_state,
        pressure: state.last_pressure,
        policy: policy.clone(),
        snapshot: state.last_snapshot.clone(),
        adapters,
        leases: state.leases.values().cloned().collect(),
        reason_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use oclive_kernel_types::{
        GpuDeviceSnapshot, ResourceAdapterDescriptor, ResourceAdapterDomain, ResourceAdapterKind,
        ResourceAdapterOperation, ResourceAdmissionMode, ResourceControlMode,
        ResourceExecutionTarget, ResourceOperatingProfile, ResourceResidencyMode, ResourceSnapshot,
    };

    struct FixedSnapshot(ResourceSnapshot);

    #[async_trait]
    impl ResourceSnapshotSource for FixedSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            self.0.clone()
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
            reason_codes: Vec::new(),
        }
    }

    fn request(adapter: &str, workload: &str, reservation_mib: u64) -> ResourceAdmissionRequest {
        ResourceAdmissionRequest {
            adapter_id: adapter.into(),
            workload_id: workload.into(),
            profile_id: None,
            gpu_device_index: None,
            reservation_mib,
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
                requires_restart: true,
                coordinator_selectable: false,
            }],
            lifecycle_operations: vec![
                ResourceAdapterOperation::Start,
                ResourceAdapterOperation::Unload,
            ],
            residency_modes: vec![
                ResourceResidencyMode::Resident,
                ResourceResidencyMode::Unloaded,
            ],
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
}
