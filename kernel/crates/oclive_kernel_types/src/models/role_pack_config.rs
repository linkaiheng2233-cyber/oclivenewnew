//! Role pack `config.json` (virtual time, memory decay, relation estrangement).

use serde::{Deserialize, Serialize};

use super::meta_action_templates_config::RolePackMetaActionTemplatesConfig;
use super::portrait_catalog_config::PortraitCatalogToggle;
use super::reply_mode_config::RolePackReplyModeConfig;
use super::reply_post_processor_config::RolePackReplyPostProcessorConfig;
use super::role_time_config::RoleTimeConfig;
use super::visual_presentation_config::RolePackVisualPresentationConfig;

fn default_memory_halflife() -> f64 {
    7.0
}

fn default_relation_halflife() -> f64 {
    30.0
}

fn default_reinforcement() -> f64 {
    0.3
}

fn default_min_strength() -> f64 {
    0.1
}

fn default_similarity_threshold() -> f64 {
    0.6
}

fn default_estrangement_threshold() -> f64 {
    0.3
}

fn default_interaction_recovery() -> f64 {
    0.12
}

fn default_reinforced_mention_threshold() -> i32 {
    3
}

fn default_personality_evolution_interval_hours() -> f64 {
    6.0
}

fn default_replay_similarity_threshold() -> f64 {
    0.6
}

fn default_ephemeral_ttl_turns() -> u32 {
    3
}

fn default_ephemeral_max_chars() -> usize {
    200
}

fn default_true() -> bool {
    true
}

/// One Deep-routing signal in `config.json` → `turn_thinking.deep_when`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "signal", rename_all = "snake_case")]
pub enum TurnThinkingSignalRule {
    LongMessage {
        #[serde(default)]
        min_chars: Option<u32>,
    },
    HighArousal,
    HighSadness,
    HighAnger,
    HighFear,
    ThisTurnEvent {
        events: Vec<String>,
    },
    RecentEvent {
        events: Vec<String>,
    },
    Keyword {
        #[serde(default)]
        keywords: Vec<String>,
    },
    DeepLatchActive,
}

/// AND group: all signals must match for Deep.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnThinkingAndGroup {
    pub all: Vec<TurnThinkingSignalRule>,
}

/// `config.json` → `turn_thinking.deep_when`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnThinkingDeepWhen {
    #[serde(default)]
    pub or: Vec<TurnThinkingSignalRule>,
    #[serde(default)]
    pub and: Vec<TurnThinkingAndGroup>,
}

/// `config.json` → `turn_thinking.latch`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnThinkingLatchConfig {
    #[serde(default)]
    pub enter_on: Vec<String>,
    #[serde(default)]
    pub exit_on: Vec<String>,
}

/// `config.json` → `turn_thinking.ephemeral_archive`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnThinkingEphemeralArchiveConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ephemeral_ttl_turns")]
    pub ttl_turns: u32,
    #[serde(default = "default_ephemeral_max_chars")]
    pub max_chars: usize,
    #[serde(default)]
    pub update_on_events: Vec<String>,
}

/// `config.json` → `turn_thinking` (co-present Fast/Deep routing + ephemeral archive).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RolePackTurnThinkingConfig {
    #[serde(default)]
    pub deep_when: TurnThinkingDeepWhen,
    #[serde(default)]
    pub latch: TurnThinkingLatchConfig,
    #[serde(default)]
    pub ephemeral_archive: Option<TurnThinkingEphemeralArchiveConfig>,
    /// Profile-mode deep archive refresh every N turns; `0` = use host `[turn_thinking]` default.
    #[serde(default)]
    pub deep_profile_update_every_n_turns: u32,
}

/// Default chat log storage location is the global path (backward compatible).
fn default_chat_storage_location() -> String {
    "global".to_string()
}

/// `config.json` → `evolution` (virtual-time-driven staged personality sedimentation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePackEvolutionConfig {
    #[serde(default = "default_personality_evolution_interval_hours")]
    pub personality_evolution_interval_hours: f64,
}

impl Default for RolePackEvolutionConfig {
    fn default() -> Self {
        Self {
            personality_evolution_interval_hours: 6.0,
        }
    }
}

/// `config.json` → `memory`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePackMemoryConfig {
    #[serde(default = "default_memory_halflife")]
    pub decay_halflife_days: f64,
    #[serde(default = "default_reinforcement")]
    pub reinforcement_factor: f64,
    #[serde(default = "default_min_strength")]
    pub min_strength_for_prompt: f64,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    #[serde(default = "default_reinforced_mention_threshold")]
    pub reinforced_mention_threshold: i32,
}

impl Default for RolePackMemoryConfig {
    fn default() -> Self {
        Self {
            decay_halflife_days: 7.0,
            reinforcement_factor: 0.3,
            min_strength_for_prompt: 0.1,
            similarity_threshold: 0.6,
            reinforced_mention_threshold: 3,
        }
    }
}

/// `config.json` → `relation`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolePackRelationConfig {
    #[serde(default = "default_relation_halflife")]
    pub decay_halflife_days: f64,
    #[serde(default = "default_estrangement_threshold")]
    pub estrangement_threshold: f64,
    /// On each real interaction, intimacy recovers by `(1 + recovery)` after estrangement decay.
    #[serde(default = "default_interaction_recovery")]
    pub interaction_recovery: f64,
}

impl Default for RolePackRelationConfig {
    fn default() -> Self {
        Self {
            decay_halflife_days: 30.0,
            estrangement_threshold: 0.3,
            interaction_recovery: 0.12,
        }
    }
}

/// `config.json` → `chat_storage.backend` — pluggable chat log backend.
///
/// Serialized as `hybrid` | `file` | `sqlite` (lowercase). Keep aligned with
/// `oclive-cli` `ChatStorageBackend` when generating role packs.
///
/// | Variant | Storage | Search | Cleanup | Memory replay |
/// |---------|---------|--------|---------|---------------|
/// | `Hybrid` (default) | SQLite + JSON mirror (`mirror: true`) | yes | yes | yes |
/// | `File` (deprecated) | Treated as `hybrid` with `mirror: true` | yes | yes | yes |
/// | `Sqlite` (deprecated) | Treated as `hybrid` with `mirror: false` | yes | yes | yes |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChatStorageBackendKind {
    /// SQLite authoritative + JSON mirror (default).
    #[default]
    Hybrid,
    /// JSON files only under `{app_data}/chats/`.
    File,
    /// SQLite only, no JSON mirror.
    Sqlite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePackChatStorageConfig {
    /// Storage backend; default `hybrid`.
    #[serde(default)]
    pub backend: Option<ChatStorageBackendKind>,
    /// Max messages per session (user + assistant combined); host default 500 when unset.
    #[serde(default)]
    pub max_messages_per_session: Option<u32>,
    /// Keep sessions updated within the last N days; older ones are auto-cleaned. Disabled when unset.
    #[serde(default)]
    pub auto_cleanup_days: Option<u32>,
    /// Max N sessions per role; oldest removed when exceeded. Disabled when unset.
    #[serde(default)]
    pub auto_cleanup_max_sessions: Option<u32>,
    /// Memory replay dedup similarity threshold (0.0–1.0); default 0.6 when unset.
    #[serde(default = "default_replay_similarity_threshold")]
    pub replay_similarity_threshold: f64,
    /// `"role_pack"` stores chat logs under `chats/` in the role pack directory;
    /// `"global"` or unset uses the default `{app_data}/chats/`.
    #[serde(default = "default_chat_storage_location")]
    pub location: String,
    /// JSON mirror under `location` / global chats root. Default follows `backend`:
    /// `hybrid`/`file` → `true`, `sqlite` → `false`. Explicit `mirror` wins.
    #[serde(default)]
    pub mirror: Option<bool>,
}

impl Default for RolePackChatStorageConfig {
    fn default() -> Self {
        Self {
            backend: None,
            max_messages_per_session: None,
            auto_cleanup_days: None,
            auto_cleanup_max_sessions: None,
            replay_similarity_threshold: default_replay_similarity_threshold(),
            location: default_chat_storage_location(),
            mirror: None,
        }
    }
}

/// Optional prompt blocks injected before the reply quality anchor (`PromptInput.extra_sections`).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RolePackPromptExtraSection {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RolePackConfigFile {
    #[serde(default)]
    pub time: RoleTimeConfig,
    #[serde(default)]
    pub memory: RolePackMemoryConfig,
    #[serde(default)]
    pub relation: RolePackRelationConfig,
    #[serde(default)]
    pub evolution: RolePackEvolutionConfig,
    #[serde(default)]
    pub chat_storage: RolePackChatStorageConfig,
    #[serde(default)]
    pub reply_post_processor: RolePackReplyPostProcessorConfig,
    #[serde(default)]
    pub reply_mode: RolePackReplyModeConfig,
    #[serde(default)]
    pub portrait_catalog: PortraitCatalogToggle,
    #[serde(default)]
    pub visual_presentation: RolePackVisualPresentationConfig,
    #[serde(default)]
    pub meta_action_templates: RolePackMetaActionTemplatesConfig,
    /// Co-present Turn Thinking policy (Deep routing, latch, ephemeral archive).
    #[serde(default)]
    pub turn_thinking: Option<RolePackTurnThinkingConfig>,
    /// Host-orchestrated prompt blocks (K-CONTRACT-WIRING-01).
    #[serde(default)]
    pub prompt_extra_sections: Vec<RolePackPromptExtraSection>,
}
