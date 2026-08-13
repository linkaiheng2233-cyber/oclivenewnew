
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

use super::{now_epoch_ms, select_queue_candidate, AdmissionQueue, ResourceCoordinator};
use oclive_kernel_contracts::ResourceSnapshotSource;
use oclive_kernel_types::{
    AppError, ResourceAdapterTransitionRequest, ResourceAdmissionDecision,
    ResourceAdmissionQueueDiagnostics, ResourceAdmissionQueueItem, ResourceAdmissionRequest,
    ResourceCoordinationDiagnosticState, ResourceCoordinatorPolicy, ResourceLeaseState,
    ResourcePriority,
};
use parking_lot::{Mutex, RwLock};
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;

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
    let permissive = ResourceCoordinator::new(ResourceCoordinatorPolicy::default(), source.clone());
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
    let shared_snapshot = Arc::new(RwLock::new(snapshot_with_host_resources(3_000, 16_000, 16)));
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
        .register_adapter_transition_grant("requester", "target", [ResourceAdapterOperation::Start])
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
        .register_adapter_transition_grant("requester", "target", [ResourceAdapterOperation::Start])
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
        .register_adapter_transition_grant("requester", "target", [ResourceAdapterOperation::Start])
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
                profile_id: (operation == ResourceAdapterOperation::Start).then(|| "full".into()),
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
