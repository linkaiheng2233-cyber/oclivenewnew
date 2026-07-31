//! Public diagnostics and control-plane DTOs for host resource coordination.
//!
//! These values describe ephemeral runtime state. They are never persisted in
//! role packs or blueprints, and they never carry LLM tokens, PCM, or frames.

use serde::{Deserialize, Serialize};

pub const RESOURCE_COORDINATION_SCHEMA_VERSION: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceControlMode {
    Managed,
    ObserveOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdmissionMode {
    Enforced,
    ObserveOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePriority {
    Resident,
    BackgroundWarmup,
    ForegroundMedia,
    ForegroundInteractive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLeaseState {
    Reserved,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdmissionDecision {
    Granted,
    GrantedUnverified,
    Reused,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePressureLevel {
    Unknown,
    Normal,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdapterKind {
    Runtime,
    ActivityObserver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdapterDomain {
    Llm,
    Voice,
    Render,
    Compute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceExecutionTarget {
    Gpu,
    Cpu,
    Hybrid,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdapterOperation {
    Observe,
    Start,
    Resume,
    Suspend,
    Unload,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceResidencyMode {
    Resident,
    OnDemand,
    Suspended,
    Unloaded,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdapterRuntimeState {
    Unknown,
    Inactive,
    Reserved,
    Active,
}

/// Distro/user objective used when the coordinator compiles adapter facts into
/// an ephemeral runtime plan. It never overrides physical safety or adapter
/// control boundaries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSchedulingStrategy {
    #[default]
    CompatibilityFirst,
    PrimaryFirst,
    LatencyFirst,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceResidencyPreference {
    Resident,
    OnDemand,
}

/// Small declarative vocabulary accepted by the scheduling-intent validator.
///
/// These are constraints and preferences, not executable lifecycle steps.
/// The coordinator remains responsible for producing and validating concrete
/// adapter transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResourceSchedulingCommand {
    Require {
        adapter_id: String,
    },
    Residency {
        adapter_id: String,
        mode: ResourceResidencyPreference,
    },
    Coexist {
        adapter_ids: Vec<String>,
    },
    Exclusive {
        adapter_ids: Vec<String>,
    },
    YieldThenRun {
        yielding_adapter_id: String,
        target_adapter_id: String,
    },
    Fallback {
        adapter_id: String,
        profile_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSchedulingIntent {
    #[serde(default)]
    pub strategy: ResourceSchedulingStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_adapter_id: Option<String>,
    #[serde(default)]
    pub commands: Vec<ResourceSchedulingCommand>,
}

impl Default for ResourceSchedulingIntent {
    fn default() -> Self {
        Self {
            strategy: ResourceSchedulingStrategy::CompatibilityFirst,
            primary_adapter_id: None,
            commands: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSchedulingIntentState {
    Ready,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSchedulingIntentDiagnostics {
    pub state: ResourceSchedulingIntentState,
    pub intent: ResourceSchedulingIntent,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl Default for ResourceSchedulingIntentDiagnostics {
    fn default() -> Self {
        Self {
            state: ResourceSchedulingIntentState::Ready,
            intent: ResourceSchedulingIntent::default(),
            reason_codes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCandidatePlanState {
    #[default]
    NotEvaluated,
    Ready,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceProfileSelectionSource {
    Current,
    Strategy,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceProfileSelection {
    pub adapter_id: String,
    pub profile_id: String,
    pub source: ResourceProfileSelectionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_reservation_mib: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_ram_mib: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cpu_threads: Option<u16>,
}

/// One lifecycle operation proposed by the read-only candidate-plan compiler.
///
/// A proposed transition is not proof that the operation ran. `executable`
/// on the containing plan additionally requires complete profile selection,
/// a registered controller for every target, and a rollback operation for
/// every proposed transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCandidateTransition {
    pub adapter_id: String,
    pub operation: ResourceAdapterOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_operation: Option<ResourceAdapterOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_by_adapter_id: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

/// Ephemeral, read-only result of compiling scheduling intent against current
/// adapter facts, leases, controller availability, and the latest snapshot.
///
/// It is never persisted to a role pack or blueprint. Execution must re-check
/// `compiled_from_revision` before applying any transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCandidatePlan {
    pub plan_id: String,
    pub compiled_from_revision: u64,
    pub state: ResourceCandidatePlanState,
    pub executable: bool,
    #[serde(default)]
    pub selections: Vec<ResourceProfileSelection>,
    #[serde(default)]
    pub transitions: Vec<ResourceCandidateTransition>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl Default for ResourceCandidatePlan {
    fn default() -> Self {
        Self {
            plan_id: "resource-plan-not-evaluated".into(),
            compiled_from_revision: 0,
            state: ResourceCandidatePlanState::NotEvaluated,
            executable: false,
            selections: Vec::new(),
            transitions: Vec::new(),
            reason_codes: vec!["resource_plan_not_evaluated".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceOperatingProfile {
    pub profile_id: String,
    /// Adapter-local quality order. Higher values mean the adapter considers
    /// this profile closer to its preferred full-capability mode.
    pub quality_rank: u16,
    pub execution_target: ResourceExecutionTarget,
    /// Adapter estimate available at registration. `None` means the host cannot
    /// statically estimate an external or dynamically configured runtime; the
    /// active lease remains the runtime truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_reservation_mib: Option<u64>,
    /// Estimated host RAM reservation for this profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_ram_mib: Option<u64>,
    /// Estimated logical CPU threads reserved while this profile is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cpu_threads: Option<u16>,
    pub requires_restart: bool,
    /// Whether the coordinator can currently request this profile directly.
    /// Registration alone must not claim that automatic profile transitions exist.
    pub coordinator_selectable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterDescriptor {
    pub adapter_id: String,
    pub kind: ResourceAdapterKind,
    pub domain: ResourceAdapterDomain,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    pub control_mode: ResourceControlMode,
    #[serde(default)]
    pub profiles: Vec<ResourceOperatingProfile>,
    #[serde(default)]
    pub lifecycle_operations: Vec<ResourceAdapterOperation>,
    #[serde(default)]
    pub residency_modes: Vec<ResourceResidencyMode>,
    /// Explicit lifecycle operation the coordinator may use for automatic
    /// preemption. `None` means this adapter is never an automatic victim even
    /// when it exposes manual suspend/unload operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_preemption: Option<ResourceAdapterOperation>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAdapterRegistrationSource {
    #[default]
    Builtin,
    HostExtension,
    DirectoryPlugin,
}

/// Owner-scoped registration envelope for resource-sensitive extensions.
///
/// Registration describes facts only. A managed descriptor becomes
/// controllable only after the host separately registers its single-writer
/// `ResourceAdapterController` implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterRegistration {
    pub source: ResourceAdapterRegistrationSource,
    pub source_id: String,
    pub descriptor: ResourceAdapterDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterDiagnostic {
    pub descriptor: ResourceAdapterDescriptor,
    #[serde(default)]
    pub registration_source: ResourceAdapterRegistrationSource,
    #[serde(default)]
    pub registration_source_id: String,
    pub runtime_state: ResourceAdapterRuntimeState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_profile_id: Option<String>,
    #[serde(default)]
    pub lease_ids: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuDeviceSnapshot {
    pub device_index: u32,
    pub name: String,
    pub total_mib: u64,
    pub free_mib: u64,
    pub used_mib: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemMemorySnapshot {
    pub total_mib: u64,
    pub available_mib: u64,
    pub used_mib: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuSnapshot {
    pub logical_cores: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_cores: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub captured_at_ms: u64,
    pub source: String,
    pub available: bool,
    #[serde(default)]
    pub gpu_devices: Vec<GpuDeviceSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_memory: Option<SystemMemorySnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<CpuSnapshot>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

impl ResourceSnapshot {
    #[must_use]
    pub fn unavailable(source: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            captured_at_ms: 0,
            source: source.into(),
            available: false,
            gpu_devices: Vec::new(),
            system_memory: None,
            cpu: None,
            reason_codes: vec![reason.into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCoordinatorPolicy {
    pub gpu_safety_reserve_mib: u64,
    #[serde(default = "default_system_memory_safety_reserve_mib")]
    pub system_memory_safety_reserve_mib: u64,
    #[serde(default = "default_cpu_safety_reserve_threads")]
    pub cpu_safety_reserve_threads: u16,
    pub pending_lease_ttl_ms: u64,
    pub active_lease_ttl_ms: u64,
    pub allow_unverified_admission: bool,
    #[serde(default = "default_admission_queue_timeout_ms")]
    pub admission_queue_timeout_ms: u64,
    #[serde(default = "default_queue_aging_quantum_ms")]
    pub queue_aging_quantum_ms: u64,
    #[serde(default = "default_true")]
    pub automatic_preemption: bool,
    #[serde(default)]
    pub scheduling: ResourceSchedulingIntent,
}

const fn default_system_memory_safety_reserve_mib() -> u64 {
    1_024
}

const fn default_cpu_safety_reserve_threads() -> u16 {
    1
}

const fn default_admission_queue_timeout_ms() -> u64 {
    30_000
}

const fn default_queue_aging_quantum_ms() -> u64 {
    2_000
}

const fn default_true() -> bool {
    true
}

impl Default for ResourceCoordinatorPolicy {
    fn default() -> Self {
        Self {
            gpu_safety_reserve_mib: 768,
            system_memory_safety_reserve_mib: default_system_memory_safety_reserve_mib(),
            cpu_safety_reserve_threads: default_cpu_safety_reserve_threads(),
            pending_lease_ttl_ms: 120_000,
            active_lease_ttl_ms: 1_800_000,
            allow_unverified_admission: true,
            admission_queue_timeout_ms: default_admission_queue_timeout_ms(),
            queue_aging_quantum_ms: default_queue_aging_quantum_ms(),
            automatic_preemption: true,
            scheduling: ResourceSchedulingIntent::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionRequest {
    pub adapter_id: String,
    pub workload_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_device_index: Option<u32>,
    pub reservation_mib: u64,
    #[serde(default)]
    pub ram_reservation_mib: u64,
    #[serde(default)]
    pub cpu_thread_reservation: u16,
    pub priority: ResourcePriority,
    pub control_mode: ResourceControlMode,
    pub admission_mode: ResourceAdmissionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLeaseDiagnostic {
    pub lease_id: String,
    pub adapter_id: String,
    pub workload_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_device_index: Option<u32>,
    pub reservation_mib: u64,
    pub actual_mib: u64,
    #[serde(default)]
    pub ram_reservation_mib: u64,
    #[serde(default)]
    pub actual_ram_mib: u64,
    #[serde(default)]
    pub cpu_thread_reservation: u16,
    #[serde(default)]
    pub actual_cpu_threads: u16,
    pub priority: ResourcePriority,
    pub control_mode: ResourceControlMode,
    pub state: ResourceLeaseState,
    pub granted_at_ms: u64,
    /// `None` means a host-managed resident runtime that must be released explicitly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionResult {
    pub decision: ResourceAdmissionDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease: Option<ResourceLeaseDiagnostic>,
    pub snapshot: ResourceSnapshot,
    pub pressure: ResourcePressureLevel,
    #[serde(default)]
    pub queue_wait_ms: u64,
    #[serde(default)]
    pub preempted_adapters: Vec<ResourcePreemptionRecord>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePreemptionRecord {
    pub adapter_id: String,
    pub operation: ResourceAdapterOperation,
    pub restore_operation: ResourceAdapterOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionQueueItem {
    pub ticket_id: u64,
    pub adapter_id: String,
    pub workload_id: String,
    pub priority: ResourcePriority,
    pub enqueued_at_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionQueueDiagnostics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_ticket_id: Option<u64>,
    #[serde(default)]
    pub queued: Vec<ResourceAdmissionQueueItem>,
}

/// Cross-process request for the authoritative kernel to transition one
/// registered runtime on behalf of another adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterTransitionRequest {
    pub adapter_id: String,
    pub operation: ResourceAdapterOperation,
    pub requested_by_adapter_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterTransitionResponse {
    pub schema_version: u32,
    pub adapter_id: String,
    pub operation: ResourceAdapterOperation,
    pub requested_by_adapter_id: String,
    pub already_in_state: bool,
    pub recovery_scheduled: bool,
    pub state_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCoordinationDiagnostics {
    pub schema_version: u32,
    #[serde(default)]
    pub state_revision: u64,
    pub state: super::execution_plan::ResourceCoordinationDiagnosticState,
    pub pressure: ResourcePressureLevel,
    pub policy: ResourceCoordinatorPolicy,
    pub snapshot: ResourceSnapshot,
    #[serde(default)]
    pub adapters: Vec<ResourceAdapterDiagnostic>,
    #[serde(default)]
    pub leases: Vec<ResourceLeaseDiagnostic>,
    #[serde(default)]
    pub scheduling: ResourceSchedulingIntentDiagnostics,
    #[serde(default)]
    pub candidate_plan: ResourceCandidatePlan,
    #[serde(default)]
    pub admission_queue: ResourceAdmissionQueueDiagnostics,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_transition_wire_shape_is_stable() {
        let request = ResourceAdapterTransitionRequest {
            adapter_id: "builtin.llm.llama_server".into(),
            operation: ResourceAdapterOperation::Suspend,
            requested_by_adapter_id: "builtin.voice.cosyvoice2".into(),
            profile_id: None,
            expected_revision: Some(7),
            reason: Some("foreground speech".into()),
        };
        let value = serde_json::to_value(&request).expect("serialize transition");
        assert_eq!(value["operation"], "suspend");
        assert_eq!(value["expected_revision"], 7);
        assert_eq!(
            serde_json::from_value::<ResourceAdapterTransitionRequest>(value)
                .expect("deserialize transition"),
            request
        );
    }

    #[test]
    fn legacy_policy_defaults_to_compatibility_strategy() {
        let policy: ResourceCoordinatorPolicy = serde_json::from_value(serde_json::json!({
            "gpu_safety_reserve_mib": 768,
            "pending_lease_ttl_ms": 120000,
            "active_lease_ttl_ms": 1800000,
            "allow_unverified_admission": true
        }))
        .expect("deserialize legacy policy");
        assert_eq!(
            policy.scheduling.strategy,
            ResourceSchedulingStrategy::CompatibilityFirst
        );
        assert!(policy.scheduling.commands.is_empty());
        assert_eq!(policy.system_memory_safety_reserve_mib, 1_024);
        assert_eq!(policy.cpu_safety_reserve_threads, 1);
        assert_eq!(policy.admission_queue_timeout_ms, 30_000);
        assert_eq!(policy.queue_aging_quantum_ms, 2_000);
        assert!(policy.automatic_preemption);
    }

    #[test]
    fn legacy_diagnostics_without_scheduling_remain_readable() {
        let diagnostics: ResourceCoordinationDiagnostics =
            serde_json::from_value(serde_json::json!({
                "schema_version": 2,
                "state": "not_evaluated",
                "pressure": "unknown",
                "policy": {
                    "gpu_safety_reserve_mib": 768,
                    "pending_lease_ttl_ms": 120000,
                    "active_lease_ttl_ms": 1800000,
                    "allow_unverified_admission": true
                },
                "snapshot": {
                    "captured_at_ms": 0,
                    "source": "not_evaluated",
                    "available": false,
                    "gpu_devices": [],
                    "reason_codes": []
                },
                "adapters": [],
                "leases": [],
                "reason_codes": []
            }))
            .expect("deserialize v2 diagnostics");
        assert_eq!(
            diagnostics.scheduling.intent.strategy,
            ResourceSchedulingStrategy::CompatibilityFirst
        );
        assert_eq!(
            diagnostics.scheduling.state,
            ResourceSchedulingIntentState::Ready
        );
        assert_eq!(diagnostics.state_revision, 0);
        assert_eq!(
            diagnostics.candidate_plan.state,
            ResourceCandidatePlanState::NotEvaluated
        );
    }

    #[test]
    fn scheduling_command_wire_shape_is_tagged_and_stable() {
        let command = ResourceSchedulingCommand::YieldThenRun {
            yielding_adapter_id: "builtin.llm.llama_server".into(),
            target_adapter_id: "builtin.voice.cosyvoice2".into(),
        };
        let value = serde_json::to_value(&command).expect("serialize command");
        assert_eq!(value["kind"], "yield_then_run");
        assert_eq!(
            serde_json::from_value::<ResourceSchedulingCommand>(value)
                .expect("deserialize command"),
            command
        );
    }

    #[test]
    fn v5_registration_and_host_resource_fields_round_trip() {
        let registration = ResourceAdapterRegistration {
            source: ResourceAdapterRegistrationSource::HostExtension,
            source_id: "com.example.live2d".into(),
            descriptor: ResourceAdapterDescriptor {
                adapter_id: "com.example.live2d.render".into(),
                kind: ResourceAdapterKind::Runtime,
                domain: ResourceAdapterDomain::Render,
                provider_id: Some("com.example.live2d".into()),
                control_mode: ResourceControlMode::Managed,
                profiles: vec![ResourceOperatingProfile {
                    profile_id: "gpu_full".into(),
                    quality_rank: 100,
                    execution_target: ResourceExecutionTarget::Hybrid,
                    estimated_reservation_mib: Some(512),
                    estimated_ram_mib: Some(256),
                    estimated_cpu_threads: Some(1),
                    requires_restart: false,
                    coordinator_selectable: true,
                }],
                lifecycle_operations: vec![
                    ResourceAdapterOperation::Start,
                    ResourceAdapterOperation::Suspend,
                    ResourceAdapterOperation::Resume,
                ],
                residency_modes: vec![
                    ResourceResidencyMode::Resident,
                    ResourceResidencyMode::Suspended,
                ],
                automatic_preemption: Some(ResourceAdapterOperation::Suspend),
            },
        };
        let value = serde_json::to_value(&registration).expect("serialize registration");
        assert_eq!(value["source"], "host_extension");
        assert_eq!(value["descriptor"]["domain"], "render");
        assert_eq!(
            serde_json::from_value::<ResourceAdapterRegistration>(value)
                .expect("deserialize registration"),
            registration
        );
    }
}
