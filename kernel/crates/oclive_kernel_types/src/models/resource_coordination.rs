//! Public diagnostics and control-plane DTOs for host resource coordination.
//!
//! These values describe ephemeral runtime state. They are never persisted in
//! role packs or blueprints, and they never carry LLM tokens, PCM, or frames.

use serde::{Deserialize, Serialize};

pub const RESOURCE_COORDINATION_SCHEMA_VERSION: u32 = 2;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceExecutionTarget {
    Gpu,
    Cpu,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdapterDiagnostic {
    pub descriptor: ResourceAdapterDescriptor,
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
pub struct ResourceSnapshot {
    pub captured_at_ms: u64,
    pub source: String,
    pub available: bool,
    #[serde(default)]
    pub gpu_devices: Vec<GpuDeviceSnapshot>,
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
            reason_codes: vec![reason.into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCoordinatorPolicy {
    pub gpu_safety_reserve_mib: u64,
    pub pending_lease_ttl_ms: u64,
    pub active_lease_ttl_ms: u64,
    pub allow_unverified_admission: bool,
}

impl Default for ResourceCoordinatorPolicy {
    fn default() -> Self {
        Self {
            gpu_safety_reserve_mib: 768,
            pending_lease_ttl_ms: 120_000,
            active_lease_ttl_ms: 1_800_000,
            allow_unverified_admission: true,
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
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCoordinationDiagnostics {
    pub schema_version: u32,
    pub state: super::execution_plan::ResourceCoordinationDiagnosticState,
    pub pressure: ResourcePressureLevel,
    pub policy: ResourceCoordinatorPolicy,
    pub snapshot: ResourceSnapshot,
    #[serde(default)]
    pub adapters: Vec<ResourceAdapterDiagnostic>,
    #[serde(default)]
    pub leases: Vec<ResourceLeaseDiagnostic>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}
