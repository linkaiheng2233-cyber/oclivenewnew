use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CONTRACT_SCHEMA_VERSION: u32 = 1;
pub const PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDescriptor {
    pub schema_version: u32,
    pub module: ModuleIdentity,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub runtime_requires: Vec<String>,
    #[serde(default)]
    pub resource_claims: Vec<ResourceClaim>,
    #[serde(default)]
    pub declared_affects: Vec<String>,
    #[serde(default)]
    pub validation_profiles: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub extensions: BTreeMap<String, ExtensionEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleIdentity {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceClaim {
    pub resource: String,
    pub mode: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionEnvelope {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactMap {
    pub schema_version: u32,
    #[serde(default)]
    pub supported_extensions: Vec<String>,
    pub module_bindings: Vec<ModuleBinding>,
    #[serde(default)]
    pub policy_affects: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub risk_overrides: Vec<RiskOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleBinding {
    pub module_id: String,
    pub descriptor: String,
    pub selectors: Vec<PathSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSelector {
    pub kind: PathSelectorKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathSelectorKind {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskOverride {
    pub id: String,
    pub selectors: Vec<PathSelector>,
    #[serde(default)]
    pub full: bool,
    #[serde(default)]
    pub force_profiles: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationCatalog {
    pub schema_version: u32,
    pub policies: Vec<ValidationPolicy>,
    pub profiles: Vec<ValidationProfile>,
    pub validators: Vec<Validator>,
    pub commands: Vec<TrustedCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationPolicy {
    pub id: String,
    pub included_tiers: Vec<ValidationTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationProfile {
    pub id: String,
    pub validators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validator {
    pub id: String,
    pub tier: ValidationTier,
    pub gate: GateStrength,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub trust: TrustLevel,
    pub command_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTier {
    Fast,
    Pr,
    Merge,
    Nightly,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStrength {
    Required,
    Advisory,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    UntrustedPr,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedCommand {
    pub id: String,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlanRequest {
    pub base_sha: String,
    pub head_sha: String,
    pub policy: String,
    pub shadow: bool,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiPlan {
    pub schema_version: u32,
    pub base_sha: String,
    pub head_sha: String,
    pub policy: String,
    pub shadow: bool,
    pub changed_files: Vec<String>,
    pub direct_modules: Vec<ReasonedSelection>,
    pub affected_modules: Vec<ReasonedSelection>,
    pub selected_profiles: Vec<ReasonedSelection>,
    pub selected_validators: Vec<PlannedValidator>,
    pub skipped_validators: Vec<SkippedValidator>,
    pub fallback: FallbackDecision,
    pub warnings: Vec<String>,
    pub impact_map_sha256: String,
    pub validation_catalog_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasonedSelection {
    pub id: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedValidator {
    pub id: String,
    pub tier: ValidationTier,
    pub gate: GateStrength,
    pub platforms: Vec<String>,
    pub trust: TrustLevel,
    pub command_id: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedValidator {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallbackDecision {
    pub full: bool,
    pub reasons: Vec<String>,
}
