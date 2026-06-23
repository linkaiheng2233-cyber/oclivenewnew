//! HTTP / Tauri invoke request and response DTOs (field names are the API contract).
//!
//! Types below `SendMessageResponse` map 1:1 to host commands documented in `creator-docs`.

use std::collections::BTreeMap;

use super::author_pack::AuthorPackFile;
use super::plugin_backends::PluginBackends;
use super::plugin_backends::PluginBackendsOverride;
use super::plugin_backends::PluginBackendsSourceMap;
use super::role::IdentityBinding;
use super::role::LifeState;
use super::role::PersonalitySource;
use super::ui_config::UiConfig;
use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 15;

/// Primary chat invoke payload (`send_message`).
#[derive(Debug, Default, Deserialize)]
pub struct SendMessageRequest {
    pub role_id: String,
    pub user_message: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    /// Optional: distinguishes multiple sessions for the same role (e.g. HTTP trial chat "new session"); combined with `role_id` as the internal DB namespace.
    #[serde(default)]
    pub session_id: Option<String>,
    /// When `true`, response may include `raw_reply` if post-processor changed the LLM text.
    #[serde(default)]
    pub include_raw_reply: Option<bool>,
}

/// Seven-dimensional emotion snapshot returned to the UI.
#[derive(Debug, Serialize, Deserialize)]
pub struct EmotionDto {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub disgust: f32,
    pub neutral: f32,
}

/// Serialized detected event for the chat response payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct DetectedEventDto {
    pub event_type: String,
    pub confidence: f32,
}

/// `send_message` co-present / remote-presence stub / remote inner-voice modes (for UI styling and debugging).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMode {
    CoPresent,
    RemoteStub,
    RemoteLife,
}

/// Primary chat invoke result (`send_message`); field `reply` is the assistant text.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub api_version: u32,
    pub schema: u32,
    /// Co-present / remote-presence stub / remote inner voice.
    pub presence_mode: PresenceMode,
    /// Relation state after this turn (`role_runtime.relation_state`).
    pub relation_state: String,
    pub reply: String,
    /// User-input emotion analysis (seven dimensions); for debugging or advanced UI display.
    pub emotion: EmotionDto,
    /// Bot emotion label parsed this turn (lowercase English; matches `Emotion::Display`).
    pub bot_emotion: String,
    /// Portrait expression (LLM + persona + events combined; matches `role_runtime.current_emotion`).
    pub portrait_emotion: String,
    /// Closed-set catalog asset id when `portrait_catalog.enabled`; legacy packs omit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_state_id: Option<String>,
    /// Render directive from visual presentation facility (#4); omitted when disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance_directive: Option<super::visual_presentation_config::PerformanceDirective>,
    pub favorability_delta: f32,
    pub favorability_current: f32,
    pub events: Vec<DetectedEventDto>,
    pub scene_id: String,
    /// Set by the backend when the user expresses travel/move intent; actual scene switch only via `switch_scene`.
    pub offer_destination_picker: bool,
    /// Set when rules/model detect the user inviting the role to travel together; confirm via `switch_scene` (`together: true`).
    #[serde(default)]
    pub offer_together_travel: bool,
    /// Whether a fallback short reply was used after main dialogue LLM failure (co-present or remote inner voice).
    #[serde(default)]
    pub reply_is_fallback: bool,
    /// When `reply_is_fallback = true`, optional main LLM failure reason (for UI hint; no full prompt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_fallback_reason: Option<String>,
    /// Knowledge chunks retrieved and injected into main/remote prompt this turn (0 = none injected or no hit).
    #[serde(default)]
    pub knowledge_chunks_in_prompt: u32,
    pub timestamp: i64,
    /// User row id after CoPresent writes to `chat_messages`; `None` if not written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_message_timestamp: Option<String>,
    /// `true` when CoPresent chat row persistence failed (SQLite authoritative store).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_persist_failed: Option<bool>,
    /// Human-readable chat persistence error when `chat_persist_failed` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_persist_error: Option<String>,
    /// `true` when role blueprint enables dual-core but host fell back to co-present path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dual_core_degraded: Option<bool>,
    /// Pre–post-processor LLM text; only when `include_raw_reply` was requested and text changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_reply: Option<String>,
}

// ----- WEEK3-004: role / memory / event queries -----

/// Sentinel submitted for the identity dropdown option "follow creator manifest default identity" (not a manifest key).
pub const OCLIVE_DEFAULT_RELATION_SENTINEL: &str = "__oclive_default__";

/// Sentinel for User Identity Prompt Template picker "follow pack default" (same value as relation sentinel).
pub const OCLIVE_DEFAULT_IDENTITY_SENTINEL: &str = "__oclive_default__";

/// One entry from `user_identities/index.json` (`get_user_identity_state`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentityDto {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maps_to_relation_id: Option<String>,
}

/// Switch active User Identity Prompt Template (`set_user_identity`).
#[derive(Debug, Clone, Deserialize)]
pub struct SetUserIdentityRequest {
    pub role_id: String,
    pub identity_id: String,
}

/// Per-scene identity override when `identity_binding = per_scene`.
#[derive(Debug, Clone, Deserialize)]
pub struct SetSceneUserIdentityRequest {
    pub role_id: String,
    pub scene_id: String,
    pub identity_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserIdentityStateRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentityStateResponse {
    pub role_id: String,
    pub identities: Vec<UserIdentityDto>,
    pub default_identity_id: String,
    pub current_identity_id: String,
    pub use_manifest_default: bool,
    pub effective_relation_key: String,
}

/// Manifest-defined user identity option (`get_role_info` / relation pickers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelationDto {
    pub id: String,
    pub name: String,
    pub prompt_hint: String,
    pub favor_multiplier: f32,
    /// Initial favorability configured for this identity in the role pack (0–100); synced to current favorability on identity switch.
    pub initial_favorability: f64,
}

/// Full role runtime snapshot for settings and plugin panels (`get_role_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleData {
    pub role_id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub personality_vector: Vec<f64>,
    pub current_favorability: f64,
    /// Placeholder; persisted bot emotion deferred to a later milestone.
    pub current_emotion: String,
    pub memory_count: i32,
    pub event_count: i32,
    /// Relation options defined in the role pack.
    pub user_relations: Vec<UserRelationDto>,
    /// Manifest default relation key.
    pub default_relation: String,
    /// Relation state (favorability-driven stage, e.g. Stranger / Friend).
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
    pub current_favorability: f64,
    pub current_emotion: String,
    pub personality_vector: Vec<f64>,
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
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStateResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRoleInteractionModeRequest {
    pub role_id: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTimeRequest {
    pub role_id: String,
    #[serde(default)]
    pub timestamp_ms: Option<i64>,
    #[serde(default)]
    pub preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTimeResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
    /// Monologues generated after time jump (typically 2; for frontend chat insertion).
    pub monologues: Vec<String>,
    pub favorability_delta: f32,
    pub favorability_current: f32,
    /// When `autonomous_scene` rule switches role `current_scene` from `from` to `to`.
    #[serde(default)]
    pub autonomous_scene_from: Option<String>,
    #[serde(default)]
    pub autonomous_scene_to: Option<String>,
}

fn default_switch_together() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSceneRequest {
    pub role_id: String,
    pub scene_id: String,
    /// `true`: write `current_scene` and treat as co-present with role; `false`: only update `user_presence_scene` (narrative solitude).
    #[serde(default = "default_switch_together")]
    pub together: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUserPresenceSceneRequest {
    pub role_id: String,
    pub scene_id: String,
}

/// Pre-import preview for `.ocpak` (manifest).
#[derive(Debug, Clone, Serialize)]
pub struct RolePackPeekResponse {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// `switch_scene` response: role info and scene welcome message (for frontend chat insertion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSceneResponse {
    #[serde(flatten)]
    pub role: RoleInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_welcome: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateMonologueRequest {
    pub role_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateMonologueResponse {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportChatLogsRequest {
    pub role_id: Option<String>,
    #[serde(default)]
    pub all_roles: bool,
    pub format: String,
    /// Optional: attach plugin backend resolution diagnostics to export (default off; ignored when `all_roles=true`).
    #[serde(default)]
    pub include_plugin_resolution_debug: bool,
    /// Optional session id for diagnostic namespace (only when `include_plugin_resolution_debug=true` and single-role export).
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportChatLogsResponse {
    pub content: String,
    pub suggested_filename: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryMemoriesRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub role_id: String,
    pub content: String,
    /// Current store is long-term memory table only; fixed as `long_term`.
    pub memory_type: String,
    pub timestamp: String,
    pub importance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryEventsRequest {
    pub role_id: String,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventItem {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
    pub timestamp: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub role_id: String,
    pub event_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventResponse {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub description: Option<String>,
}

/// Theater cast member reference (`generate_theater_scene`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterCastRef {
    pub role_id: String,
    pub name: String,
}

/// One scripted beat in a theater scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TheaterScriptLine {
    pub id: String,
    pub cast: String,
    pub name: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
}

/// Poke chip brief for `cast_rewrite` (chip_id + drama intent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterPokeChipDef {
    pub chip_id: String,
    pub drama_seed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Fork patch template for cast adaptation (`mode = cast_adapt`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterForkTemplate {
    pub chip_id: String,
    pub insert_after_beat_id: String,
    pub patch_lines: Vec<TheaterScriptLine>,
}

/// User-applied poke / custom tweak metadata for scene director rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterTweak {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip_label: Option<String>,
    pub drama_seed: String,
    pub insert_after_beat_id: String,
    pub lead_cast: String,
}

/// `generate_theater_scene` request — full-scene structured rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterSceneRequest {
    pub cast_a: TheaterCastRef,
    pub cast_b: TheaterCastRef,
    pub scene_id: String,
    pub base_beats: Vec<TheaterScriptLine>,
    pub applied_tweaks: Vec<TheaterTweak>,
    pub fallback_beats: Vec<TheaterScriptLine>,
    #[serde(default)]
    pub max_beats: Option<u32>,
    /// `cast_adapt` | `cast_rewrite` | `ripple` (JSON ripple rewrite) | `patch` (local prose micro-scene).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Patch mode only: `0` = first variant, `1` = alternate plot branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_variant: Option<u8>,
    /// Fork patch templates (name-bound baseline) for `cast_adapt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_templates: Option<Vec<TheaterForkTemplate>>,
    /// Cast-adapt pass: `voice` | `depth` | `polish` (multi-round persona rewrite).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapt_pass: Option<String>,
    /// Poke chip definitions for `cast_rewrite`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poke_chips: Option<Vec<TheaterPokeChipDef>>,
    /// Pair-relation preset id (`family` | `friend` | `stranger` | `lover`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_relation_id: Option<String>,
    /// Human-readable pair-relation tone for cast_rewrite / ripple prompts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_relation_hint: Option<String>,
    /// Theater scene preset id (`breakfast` | `supermarket` | …); orthogonal to `scene_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theater_scene: Option<String>,
    /// Short scene description for cast_rewrite / ripple prompts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_brief: Option<String>,
    /// Scene constraints (location, time, forbidden elements).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_setting_hint: Option<String>,
}

/// `generate_theater_scene` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterSceneResponse {
    pub beats: Vec<TheaterScriptLine>,
    /// `local` | `cloud` | `fallback`
    pub source: String,
    pub model: String,
    /// Adapted fork patches when `mode = cast_adapt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapted_forks: Option<Vec<TheaterForkTemplate>>,
    /// Machine-readable hint when `source = "fallback"` (e.g. `rewrite_llm_timeout`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Partial success note (e.g. `rewrite_forks_template` when beats OK but forks reused).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_note: Option<String>,
}
