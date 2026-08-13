//! Host-owned resource admission, leases, pressure, and diagnostics.
//!
//! This is a control-plane facility. It does not execute model, voice, or
//! rendering business data paths; concrete adapters remain responsible for
//! starting and stopping their own runtimes.

mod admission;
mod lease;
#[cfg(test)]
mod tests;
mod transition;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use oclive_kernel_contracts::{
    ResourceAdapterController, ResourceAdapterRegistrar, ResourceSnapshotSource,
};
use oclive_kernel_types::{
    AppError, ResourceAdapterOperation, ResourceAdapterRegistration,
    ResourceAdmissionQueueDiagnostics, ResourceAdmissionQueueItem, ResourceAdmissionRequest,
    ResourceAdmissionResult, ResourceCoordinationDiagnosticState, ResourceCoordinationDiagnostics,
    ResourceCoordinatorPolicy, ResourceLeaseDiagnostic, ResourcePressureLevel, ResourcePriority,
    ResourceSnapshot, RESOURCE_COORDINATION_SCHEMA_VERSION,
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
