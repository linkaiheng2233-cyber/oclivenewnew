//! Session slot override, plugin backend, and debug DTOs.

use std::collections::BTreeMap;

use super::super::plugin_backends::{
    PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap,
};
use serde::{Deserialize, Serialize};

/// Per-session `slot_registry` backend override (`set_session_slot_override`).
#[derive(Debug, Clone, Deserialize)]
pub struct SetSessionSlotOverrideRequest {
    pub role_id: String,
    /// `slot_registry` instance key (e.g. `memory`, `llm`).
    pub slot_key: String,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub plugin: Option<String>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClearSessionSlotOverrideRequest {
    pub role_id: String,
    pub slot_key: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClearAllSessionSlotOverridesRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Write full `slot_registry` back to role pack `pipeline.ocblueprint` (architecture diagram R2 disk write).
#[derive(Debug, Clone, Deserialize)]
pub struct SaveRoleSlotRegistryRequest {
    pub role_id: String,
    pub slot_registry: BTreeMap<String, oclive_validation::SlotRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    pub percent: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_total: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetUserRelationRequest {
    pub role_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSceneUserRelationRequest {
    pub role_id: String,
    pub scene_id: String,
    pub relation: String,
}

/// Remove scene identity override so conversation identity falls back to global effective identity (`use_manifest_default` / `user_relation`).
#[derive(Debug, Clone, Deserialize)]
pub struct ClearSceneUserRelationRequest {
    pub role_id: String,
    pub scene_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetEvolutionFactorRequest {
    pub role_id: String,
    pub event_impact_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetRemoteLifeEnabledRequest {
    pub role_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSessionPluginBackendRequest {
    pub role_id: String,
    /// `memory` | `emotion` | `event` | `prompt` | `llm` | `agent`
    pub module: String,
    /// Backend value (snake_case) tri-state:
    /// - field omitted: do not change this module override;
    /// - `null`: remove session override for this module and revert to role pack default;
    /// - `"xxx"`: set to specified backend.
    #[serde(default)]
    pub backend: Option<Option<String>>,
    /// Only when `module = memory`: non-empty after trim sets session `local_memory_provider_id`;
    /// empty string clears session override for this field. Omitted field means no change.
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    /// Optional session id for HTTP trial chat and other multi-session scenarios; omitted means role default session.
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Query runtime snapshot; `session_id` same semantics as the same-named field in `SendMessageRequest` (multi-path trial chat, etc.).
#[derive(Debug, Clone, Deserialize)]
pub struct GetRoleInfoRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// GET-only affect display snapshot (`get_display_metrics` / HTTP `/display_metrics`).
#[derive(Debug, Clone, Deserialize)]
pub struct GetDisplayMetricsRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPluginResolutionDebugRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResolutionDebugInfo {
    pub app_version: String,
    pub api_version: u32,
    pub schema_version: u32,
    pub role_id: String,
    pub session_namespace: String,
    pub plugin_backends_pack_default: PluginBackends,
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    pub plugin_backends_effective: PluginBackends,
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    #[serde(default)]
    pub llm_env_override: Option<String>,
    pub remote_plugin_url_configured: bool,
    pub remote_llm_url_configured: bool,
    #[serde(default)]
    pub local_provider_ids: Vec<String>,
    pub local_provider_count: usize,
}
