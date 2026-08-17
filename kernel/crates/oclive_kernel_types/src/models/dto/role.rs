//! Role data / summary / info DTOs.

use std::collections::BTreeMap;

use super::super::author_pack::AuthorPackFile;
use super::super::plugin_backends::{
    PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap,
};
use super::super::role::{IdentityBinding, LifeState, PersonalitySource};
use super::super::ui_config::UiConfig;
use serde::{Deserialize, Serialize};

use super::{DisplayMetricsDto, UserRelationDto};
use crate::models::ReplyModeInfoDto;

/// Full role runtime snapshot for settings and plugin panels (`get_role_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleData {
    pub role_id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Whether this role pack contains a valid `adult_extension.json`.
    #[serde(default)]
    pub adult_extension_available: bool,
    /// Present when an invalid adult extension was disabled during load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_extension_error: Option<String>,
    #[deprecated(note = "use display_metrics.traits")]
    pub personality_vector: Vec<f64>,
    #[deprecated(note = "use display_metrics.favor")]
    pub current_favorability: f64,
    /// UI-only affect snapshot (favor / traits / relation stage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_metrics: Option<DisplayMetricsDto>,
    /// Placeholder; persisted bot emotion deferred to a later milestone.
    pub current_emotion: String,
    pub memory_count: i32,
    pub event_count: i32,
    /// Relation options defined in the role pack.
    pub user_relations: Vec<UserRelationDto>,
    /// Manifest default relation key.
    pub default_relation: String,
    /// Relation state (favorability-driven stage, e.g. Stranger / Friend).
    #[deprecated(note = "use display_metrics.relation_summary")]
    pub relation_state: String,
    /// Current runtime relation (resolved manifest key).
    pub current_user_relation: String,
    pub use_manifest_default: bool,
    /// When remote-presence: whether to generate life trajectory and inner voice (user toggle; persisted in `role_runtime`).
    pub remote_life_enabled: bool,
    /// Role pack `settings.json` → `remote_presence.default_enabled` (suggested mode default); UI hint only.
    pub remote_life_pack_default: Option<bool>,
    /// Effective event impact factor (DB overrides manifest default).
    pub event_impact_factor: f64,
    /// `evolution.personality_source`：`vector` | `profile`
    #[serde(default)]
    pub personality_source: PersonalitySource,
    /// Ollama model used by this role (manifest → `OLLAMA_MODEL` → global default).
    pub effective_ollama_model: String,
    /// Whether identity is scene-bound (manifest `identity_binding`).
    pub identity_binding: IdentityBinding,
    /// Current interaction mode (`role_runtime`).
    pub interaction_mode: String,
    /// Suggested role pack default from `settings.json` (optional).
    pub interaction_mode_pack_default: Option<String>,
    /// Current schedule inference (`null` when unconfigured or no matching time slot).
    #[serde(default)]
    pub current_life: Option<LifeStateDto>,
    /// `settings.json` → `plugin_backends` (matches runtime `PluginHost` resolution).
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// Session-level override (current session namespace only; `null` when none).
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    /// Effective backends after session override (runtime panel display and toggle echo).
    #[serde(default)]
    pub plugin_backends_effective: PluginBackends,
    /// Effective backend source (pack/session/env).
    #[serde(default)]
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    /// Role pack root `ui.json` (theme, layout, slots, etc.).
    #[serde(default)]
    pub pack_ui_config: UiConfig,
    /// `author.suggested_ui` when non-empty, else same as `pack_ui_config`; plugin UI seed/reset baseline.
    #[serde(default)]
    pub pack_ui_baseline: UiConfig,
    /// Optional full `author.json` (recommended plugins, suggested backends, etc.).
    #[serde(default)]
    pub author_pack: Option<AuthorPackFile>,
    #[serde(default)]
    pub slot_registry_pack: Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
    #[serde(default)]
    pub slot_registry_effective: Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
    #[serde(default)]
    pub slot_session_overridden_keys: Vec<String>,
    /// v2 blueprint `groups` (architecture diagram logical grouping; `null` for legacy).
    #[serde(default)]
    pub blueprint_groups_pack: Option<BTreeMap<String, oclive_validation::SlotGroupEntry>>,
}

/// Lightweight role list entry (`list_roles`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default = "default_preset_order")]
    pub preset_order: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction_mode_suggestion: Option<String>,
    /// Whether this role pack contains a valid `adult_extension.json`.
    #[serde(default)]
    pub adult_extension_available: bool,
    /// Present when an invalid adult extension was disabled during load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_extension_error: Option<String>,
}

fn default_preset_order() -> u32 {
    999
}

/// Scene id + display label for scene switch UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLabelEntry {
    pub id: String,
    pub label: String,
}

/// Current activity inferred from virtual time + manifest `life_schedule` (UI / debug).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeStateDto {
    pub label: String,
    pub activity_key: String,
    pub busy_level: f32,
    pub preferred_scene_id: Option<String>,
}

impl From<&LifeState> for LifeStateDto {
    fn from(s: &LifeState) -> Self {
        Self {
            label: s.label.clone(),
            activity_key: s.activity_key.clone(),
            busy_level: s.busy_level,
            preferred_scene_id: s.optional_scene_hint.clone(),
        }
    }
}

/// Role panel / scene UI snapshot (`get_role_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub role_id: String,
    pub role_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Whether this role pack contains a valid `adult_extension.json`.
    #[serde(default)]
    pub adult_extension_available: bool,
    /// Present when an invalid adult extension was disabled during load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_extension_error: Option<String>,
    #[deprecated(note = "use display_metrics.favor")]
    pub current_favorability: f64,
    pub current_emotion: String,
    #[deprecated(note = "use display_metrics.traits")]
    pub personality_vector: Vec<f64>,
    /// UI-only affect snapshot (favor / traits / relation stage).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_metrics: Option<DisplayMetricsDto>,
    /// `evolution.personality_source`：`vector` | `profile`
    #[serde(default)]
    pub personality_source: PersonalitySource,
    pub last_interaction: Option<String>,
    /// Available scene ids (manifest + `scenes/` directory).
    pub scenes: Vec<String>,
    /// Same order as `scenes`; `label` from `scenes/{id}/scene.json` `name` or built-in mapping.
    pub scene_labels: Vec<SceneLabelEntry>,
    pub current_scene: Option<String>,
    /// User narrative / send-message context scene (persisted); may differ from `current_scene`.
    pub user_presence_scene: Option<String>,
    /// Virtual world time (UTC ms); 0 if not yet initialized via `get_time_state`.
    pub virtual_time_ms: i64,
    pub user_relations: Vec<UserRelationDto>,
    pub default_relation: String,
    pub current_user_relation: String,
    /// Whether the user selected "default identity" (follows manifest `default_relation`); when true the dropdown should show `OCLIVE_DEFAULT_RELATION_SENTINEL`.
    pub use_manifest_default: bool,
    /// Relation state (`role_runtime.relation_state`).
    #[deprecated(note = "use display_metrics.relation_summary")]
    pub relation_state: String,
    /// Remote inner voice toggle.
    pub remote_life_enabled: bool,
    /// Role pack suggested default for remote inner voice (`settings.json` → `remote_presence.default_enabled`).
    pub remote_life_pack_default: Option<bool>,
    pub event_impact_factor: f64,
    /// Ollama model actually used by this role (manifest → `OLLAMA_MODEL` → global default).
    pub effective_ollama_model: String,
    /// Whether identity is scene-bound (manifest `identity_binding`).
    pub identity_binding: IdentityBinding,
    /// Current interaction mode (`role_runtime`).
    pub interaction_mode: String,
    /// Suggested role pack default from `settings.json` (optional).
    pub interaction_mode_pack_default: Option<String>,
    /// Current schedule inference (`null` when unconfigured or no time window matches).
    #[serde(default)]
    pub current_life: Option<LifeStateDto>,
    /// `settings.json` → `plugin_backends` (consistent with `load_role` / orchestration layer).
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// Session-level override (current session namespace only; `null` when none).
    #[serde(default)]
    pub plugin_backends_session_override: Option<PluginBackendsOverride>,
    /// Effective backends after session override (runtime panel display and toggle echo).
    #[serde(default)]
    pub plugin_backends_effective: PluginBackends,
    /// Effective backend source (pack/session/env).
    #[serde(default)]
    pub plugin_backends_effective_sources: PluginBackendsSourceMap,
    /// Whether the currently loaded role has a worldview knowledge index built (`knowledge_index`).
    #[serde(default)]
    pub knowledge_enabled: bool,
    /// `knowledge_index.chunks` count; 0 when index not loaded.
    #[serde(default)]
    pub knowledge_chunk_count: i32,
    /// Role pack root `ui.json` (theme, layout, slots, etc.).
    #[serde(default)]
    pub pack_ui_config: UiConfig,
    /// `author.suggested_ui` when non-empty, else same as `pack_ui_config`; plugin UI seed/reset baseline.
    #[serde(default)]
    pub pack_ui_baseline: UiConfig,
    /// Optional `author.json`.
    #[serde(default)]
    pub author_pack: Option<AuthorPackFile>,
    /// Role pack `slot_registry` (v2 blueprint; `null` for legacy packs).
    #[serde(default)]
    pub slot_registry_pack: Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
    /// Effective `slot_registry` after session overrides.
    #[serde(default)]
    pub slot_registry_effective: Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
    /// Instance keys with overrides in the current session.
    #[serde(default)]
    pub slot_session_overridden_keys: Vec<String>,
    /// v2 blueprint `groups` (architecture diagram logical grouping; `null` for legacy).
    #[serde(default)]
    pub blueprint_groups_pack: Option<BTreeMap<String, oclive_validation::SlotGroupEntry>>,
    /// `runtime_config.dual_core.enabled` and `pipeline.experimental` is non-empty.
    #[serde(default)]
    pub dual_core_enabled: bool,
    /// `pipeline.experimental` action list (architecture diagram / debug read-only).
    #[serde(default)]
    pub pipeline_experimental_actions: Vec<String>,
    /// Effective reply post-processor enabled (role pack + distro merge).
    #[serde(default)]
    pub reply_post_processor_enabled: bool,
    /// `builtin` | `remote` | `directory` | `off` when disabled.
    #[serde(default)]
    pub reply_post_processor_backend: String,
    /// Effective builtin profile (`standard` | `minimal`) when enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_post_processor_profile: Option<String>,
    /// Effective reply presentation mode; present when the role pack enables segmented replies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_reply_mode: Option<ReplyModeInfoDto>,
}
