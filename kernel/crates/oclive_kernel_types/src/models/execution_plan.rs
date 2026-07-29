//! Read-only diagnostics for the host-owned capability registry and execution plan.
//!
//! These DTOs describe an in-memory snapshot. They are not role-pack fields and
//! must never be persisted back into `pipeline.ocblueprint`.

use serde::{Deserialize, Serialize};

use super::plugin_backends::PluginBackends;

pub const EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
pub struct GetExecutionPlanDiagnosticsRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPlanFlowTemplate {
    CoPresentStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCoordinationDiagnosticState {
    NotEvaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityConsumerKind {
    SixSlot,
    Facility,
    SideChannel,
    Host,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConsumerDiagnostic {
    pub capability: String,
    pub kind: CapabilityConsumerKind,
    pub consumer_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderSource {
    Builtin,
    Directory,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderAvailability {
    Ready,
    Disabled,
    ManifestIncompatible,
    NotExecutable,
    DependencyUnavailable,
    PermissionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPermissionDiagnostic {
    pub permission: String,
    pub granted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProviderDiagnostic {
    pub provider_id: String,
    pub version: String,
    pub manifest_schema_version: u32,
    pub source: CapabilityProviderSource,
    pub provides: Vec<String>,
    pub availability: CapabilityProviderAvailability,
    #[serde(default)]
    pub permissions: Vec<CapabilityPermissionDiagnostic>,
    #[serde(default)]
    pub dependency_issues: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRegistryDiagnostic {
    pub schema_version: u32,
    pub distro_id: String,
    pub consumers: Vec<CapabilityConsumerDiagnostic>,
    pub providers: Vec<CapabilityProviderDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlanCoreNode {
    pub node_id: String,
    pub backend: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionPlanStatus {
    Ready,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlanExtension {
    pub instance_id: String,
    pub capability: String,
    pub required: bool,
    pub config_schema_version: u32,
    pub config_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_provider_version: Option<String>,
    pub status: ExtensionPlanStatus,
    pub active: bool,
    #[serde(default)]
    pub provider_candidates: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPlanDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlanDiagnostic {
    pub code: String,
    pub severity: ExecutionPlanDiagnosticSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_provider_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub schema_version: u32,
    pub role_id: String,
    pub distro_id: String,
    pub flow_template: ExecutionPlanFlowTemplate,
    pub core_nodes: Vec<ExecutionPlanCoreNode>,
    pub core_backends: PluginBackends,
    pub extensions: Vec<ExecutionPlanExtension>,
    pub activatable: bool,
    pub resource_coordination: ResourceCoordinationDiagnosticState,
    #[serde(default)]
    pub diagnostics: Vec<ExecutionPlanDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlanDiagnostics {
    pub schema_version: u32,
    pub plan: ExecutionPlan,
    pub capability_registry: CapabilityRegistryDiagnostic,
}
