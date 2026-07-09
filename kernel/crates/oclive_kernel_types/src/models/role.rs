use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use super::author_pack::AuthorPackFile;
use super::knowledge::KnowledgeIndex;
use super::plugin_backends::PluginBackends;
use super::portrait_catalog_config::{PortraitCatalogFile, PortraitCatalogToggle};
use super::reply_post_processor_config::RolePackReplyPostProcessorConfig;
use super::role_pack_config::{
    RolePackChatStorageConfig, RolePackEvolutionConfig, RolePackMemoryConfig,
    RolePackPromptExtraSection, RolePackRelationConfig, RolePackTurnThinkingConfig,
};
use super::role_time_config::RoleTimeConfig;
use super::scene_disk::DiskSceneConfig;
use super::ui_config::UiConfig;
use super::user_identity::UserIdentityCatalog;
use super::visual_presentation_config::RolePackVisualPresentationConfig;
pub use oclive_validation::{
    AutonomousSceneConfig, AutonomousSceneRule, IdentityBinding, LifeAvailability,
    LifeScheduleDisk, LifeScheduleEntryDisk, LifeTrajectoryDisk, PersonalitySource, PipelineStep,
    RemotePresenceConfig, RuntimeConfig,
};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

/// Default persona values inside a role pack (legacy seven dimensions, matching the `PersonalityVector` fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityDefaults {
    pub stubbornness: f32,
    pub clinginess: f32,
    pub sensitivity: f32,
    pub assertiveness: f32,
    pub forgiveness: f32,
    pub talkativeness: f32,
    pub warmth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionBounds {
    pub stubbornness: (f64, f64),
    pub clinginess: (f64, f64),
    pub sensitivity: (f64, f64),
    pub assertiveness: (f64, f64),
    pub forgiveness: (f64, f64),
    pub talkativeness: (f64, f64),
    pub warmth: (f64, f64),
}

impl EvolutionBounds {
    #[must_use]
    pub fn full_01() -> Self {
        let r = (0.0, 1.0);
        Self {
            stubbornness: r,
            clinginess: r,
            sensitivity: r,
            assertiveness: r,
            forgiveness: r,
            talkativeness: r,
            warmth: r,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub event_impact_factor: f64,
    pub ai_analysis_interval: i32,
    pub max_change_per_event: f64,
    pub max_total_change: f64,
    /// `vector`: keep using the seven-dimension deltas; `profile`: use the core personality profile + the runtime "mutable personality profile" (**maintained only by the LLM from the dialogue**); the seven dimensions are a **view** induced from the text, for understanding and UI only.
    #[serde(default)]
    pub personality_source: PersonalitySource,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            event_impact_factor: 1.0,
            ai_analysis_interval: 15,
            max_change_per_event: 0.05,
            max_total_change: 0.5,
            personality_source: PersonalitySource::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_scene_wm")]
    pub scene_weight_multiplier: f64,
    #[serde(default)]
    pub topic_weights: HashMap<String, HashMap<String, f64>>,
}

fn default_scene_wm() -> f64 {
    1.2
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            scene_weight_multiplier: default_scene_wm(),
            topic_weights: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelation {
    pub id: String,
    pub name: String,
    pub prompt_hint: String,
    #[serde(default = "default_favor_mult")]
    pub favor_multiplier: f32,
    /// Starting favorability when this identity is selected (0–100); defaults to 50 if omitted.
    #[serde(default = "default_initial_favorability")]
    pub initial_favorability: f64,
}

fn default_favor_mult() -> f32 {
    1.0
}

fn default_initial_favorability() -> f64 {
    50.0
}

impl UserRelation {
    #[must_use]
    pub fn initial_favorability_clamped(&self) -> f64 {
        self.initial_favorability.clamp(0.0, 100.0)
    }
}

/// Current life state derived from virtual time (engine-internal)
#[derive(Debug, Clone, PartialEq)]
pub struct LifeState {
    pub label: String,
    pub activity_key: String,
    pub busy_level: f32,
    pub optional_scene_hint: Option<String>,
}

/// Loaded role pack aggregate (manifest + settings + derived runtime fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    /// **Core personality profile**: the fixed persona set by the creator and user; the **AI must not rewrite** it at runtime (see `mutable_profile_llm`), and together with the mutable profile it forms the complete persona.
    pub core_personality: String,
    pub default_personality: PersonalityDefaults,
    pub evolution_bounds: EvolutionBounds,
    pub user_relations: Vec<UserRelation>,
    #[serde(default)]
    pub evolution_config: EvolutionConfig,
    #[serde(default)]
    pub memory_config: Option<MemoryConfig>,
    #[serde(default)]
    pub default_relation: String,
    /// Ollama model name from the role pack `manifest` (interchangeable with the `model` key); if empty, falls back to the environment variable and global default
    #[serde(default)]
    pub ollama_model: Option<String>,
    /// Whether identity is bound to the scene; the default `per_scene` matches historical behavior.
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    /// `life_trajectory` in `manifest.json` (optional)
    #[serde(default)]
    pub life_trajectory: Option<LifeTrajectoryDisk>,
    /// `life_schedule` in `manifest.json` (optional): daily fragments under virtual time, coexisting with `life_trajectory` (the remote-presence tone)
    #[serde(default)]
    pub life_schedule: Option<LifeScheduleDisk>,
    /// `remote_presence` in `settings.json` (optional, mainly `default_enabled`)
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    /// `autonomous_scene` in `settings.json` (optional, virtual-time-driven character movement)
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    /// Optional in `settings.json`: `immersive` | `pure_chat`; runtime persistence is in `role_runtime.interaction_mode`
    #[serde(default)]
    pub interaction_mode: Option<String>,
    /// Role pack `manifest.min_runtime_version`: the minimum required oclive version; omitting it means no check.
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    /// When true, by default this does not appear in `list_roles` (in-repo debug/identity example packs); `load_role` can still load it by id. See the `OCLIVE_LIST_DEV_ROLES` environment variable.
    #[serde(default)]
    pub dev_only: bool,
    /// Blueprint `meta.featured`: show in first-run preset gallery.
    #[serde(default)]
    pub featured: bool,
    /// Blueprint `meta.deep_capsule_enabled`: use `prompts/deep_capsule.txt` on Small+Deep when true.
    #[serde(default)]
    pub deep_capsule_enabled: bool,
    /// Wave D: offline-distilled Deep persona (`prompts/deep_capsule.txt`; in-memory only).
    #[serde(skip)]
    pub deep_capsule: Option<String>,
    /// Blueprint `meta.preset_order`: gallery sort (lower first).
    #[serde(default = "default_preset_order")]
    pub preset_order: u32,
    /// `settings.json` → `plugin_backends` (optional; defaults to all builtin)
    #[serde(
        default = "default_plugin_backends",
        with = "serde_arc_plugin_backends"
    )]
    pub plugin_backends: Arc<PluginBackends>,
    /// `pipeline.ocblueprint` v2 → `slot_registry` (multi-instance; used by P2+ orchestration; serialized for debug/export)
    #[serde(default, skip_serializing_if = "slot_registry_is_empty")]
    pub slot_registry: Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
    /// `pipeline.ocblueprint` v2 → `groups` (architecture-diagram grouping; optional)
    #[serde(default, skip_serializing_if = "slot_groups_is_empty")]
    pub slot_groups: Option<BTreeMap<String, oclive_validation::SlotGroupEntry>>,
    /// Index after loading `knowledge/` (in-memory only; populated by [`crate::infrastructure::storage::RoleStorage`])
    #[serde(skip)]
    pub knowledge_index: Option<Arc<KnowledgeIndex>>,
    /// Role pack `ui.json` (in-memory only; populated by [`crate::infrastructure::storage::RoleStorage`])
    #[serde(skip)]
    pub ui_config: UiConfig,
    /// Role pack `author.json` (optional; in-memory only)
    #[serde(skip)]
    pub author_pack: Option<AuthorPackFile>,
    /// Optional in `settings.json`: the full text of the main-dialogue "quality anchor"; if non-empty it replaces the engine default (see `prompt_builder::DEFAULT_REPLY_QUALITY_ANCHOR`).
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
    /// `config.json` → `time` (virtual-clock flow rate); if not provided, uses the default 1:5.
    #[serde(default)]
    pub time_config: RoleTimeConfig,
    /// `config.json` → `memory` (Ebbinghaus decay and reinforcement).
    #[serde(default)]
    pub pack_memory_config: RolePackMemoryConfig,
    /// `config.json` → `relation` (estrangement decay).
    #[serde(default)]
    pub pack_relation_config: RolePackRelationConfig,
    /// `config.json` → `evolution` (virtual-time phased personality-evolution interval).
    #[serde(default)]
    pub pack_evolution_config: RolePackEvolutionConfig,
    /// `config.json` → `chat_storage` (per-session chat-history cap, etc.).
    #[serde(default)]
    pub pack_chat_storage_config: RolePackChatStorageConfig,
    /// `config.json` → `reply_post_processor` (builtin post-LLM text polish; default disabled).
    #[serde(default)]
    pub pack_reply_post_processor_config: RolePackReplyPostProcessorConfig,
    /// `config.json` → `portrait_catalog.enabled` (assets in `portrait_catalog.json`).
    #[serde(default)]
    pub pack_portrait_catalog: PortraitCatalogToggle,
    /// `portrait_catalog.json` (in-memory; populated when `pack_portrait_catalog.enabled`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait_catalog: Option<PortraitCatalogFile>,
    /// `config.json` → `visual_presentation` (facility #4; default disabled).
    #[serde(default)]
    pub pack_visual_presentation_config: RolePackVisualPresentationConfig,
    /// `config.json` → `turn_thinking` (Fast/Deep routing, latch, ephemeral archive).
    #[serde(default)]
    pub pack_turn_thinking_config: Option<RolePackTurnThinkingConfig>,
    /// `config.json` → `prompt_extra_sections` (generic prompt blocks before quality anchor).
    #[serde(default)]
    pub pack_prompt_extra_sections: Vec<RolePackPromptExtraSection>,
    /// `user_identities/` catalog (in-memory only; populated by [`RoleStorage::finish_role_pack_load`]).
    #[serde(skip)]
    pub user_identity_catalog: Option<Arc<UserIdentityCatalog>>,
    /// v3 blueprint `runtime_config` (loaded by the host; creator packs usually omit it or leave dual-core off).
    #[serde(default)]
    pub runtime_config: Option<RuntimeConfig>,
    /// v3 blueprint `pipeline.experimental` (`pipeline.stable` does not participate in runtime execution).
    #[serde(default)]
    pub pipeline_experimental: Option<Vec<PipelineStep>>,
    /// Scene id list (manifest `scenes` + the `scenes/` subdirectory); populated by [`RoleStorage::finish_role_pack_load`].
    #[serde(skip)]
    pub scene_ids: Arc<[String]>,
    /// `scene.json` parse results cached by scene id; populated by [`RoleStorage::get_scene_config`].
    #[serde(skip)]
    pub scene_config_cache: Arc<RwLock<HashMap<String, Arc<DiskSceneConfig>>>>,
    /// Scene text-material cache (`desc:{scene}` / `away:{char}:{user}`); populated by [`RoleStorage`].
    #[serde(skip)]
    pub scene_text_cache: Arc<RwLock<HashMap<String, Arc<str>>>>,
    /// Directory the role pack was loaded from (runtime only).
    #[serde(skip)]
    pub source_dir: Option<PathBuf>,
}

fn default_preset_order() -> u32 {
    999
}

fn default_plugin_backends() -> Arc<PluginBackends> {
    Arc::new(PluginBackends::default())
}

mod serde_arc_plugin_backends {
    use super::{Arc, PluginBackends};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Arc<PluginBackends>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<PluginBackends>, D::Error>
    where
        D: Deserializer<'de>,
    {
        PluginBackends::deserialize(deserializer).map(Arc::new)
    }
}

impl Default for Role {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            version: String::new(),
            author: String::new(),
            core_personality: String::new(),
            default_personality: PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![],
            evolution_config: EvolutionConfig::default(),
            memory_config: None,
            default_relation: "friend".to_string(),
            ollama_model: None,
            identity_binding: IdentityBinding::default(),
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            featured: false,
            deep_capsule_enabled: false,
            deep_capsule: None,
            preset_order: default_preset_order(),
            plugin_backends: default_plugin_backends(),
            slot_registry: None,
            slot_groups: None,
            knowledge_index: None,
            ui_config: UiConfig::default(),
            author_pack: None,
            reply_quality_anchor: None,
            time_config: RoleTimeConfig::default(),
            pack_memory_config: RolePackMemoryConfig::default(),
            pack_relation_config: RolePackRelationConfig::default(),
            pack_evolution_config: RolePackEvolutionConfig::default(),
            pack_chat_storage_config: RolePackChatStorageConfig::default(),
            pack_reply_post_processor_config: RolePackReplyPostProcessorConfig::default(),
            pack_portrait_catalog: PortraitCatalogToggle::default(),
            portrait_catalog: None,
            pack_visual_presentation_config: RolePackVisualPresentationConfig::default(),
            pack_turn_thinking_config: None,
            pack_prompt_extra_sections: Vec::new(),
            user_identity_catalog: None,
            runtime_config: None,
            pipeline_experimental: None,
            scene_ids: Arc::from(Vec::<String>::new()),
            scene_config_cache: Arc::new(RwLock::new(HashMap::new())),
            scene_text_cache: Arc::new(RwLock::new(HashMap::new())),
            source_dir: None,
        }
    }
}

impl Role {
    /// Dual-core gate (pure predicate): `runtime_config.dual_core.enabled` and `pipeline.experimental` is non-empty.
    ///
    /// The host uses this to choose dual-core scheduling or the single-path `co_present`; it does **not** execute experimental steps.
    #[must_use]
    pub fn dual_core_gated(&self) -> bool {
        self.runtime_config
            .as_ref()
            .and_then(|r| r.dual_core.as_ref())
            .is_some_and(|d| d.enabled)
            && self
                .pipeline_experimental
                .as_ref()
                .is_some_and(|steps| !steps.is_empty())
    }
}

fn slot_registry_is_empty(
    m: &Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>>,
) -> bool {
    m.as_ref().is_none_or(|map| map.is_empty())
}

fn slot_groups_is_empty(m: &Option<BTreeMap<String, oclive_validation::SlotGroupEntry>>) -> bool {
    m.as_ref().is_none_or(|map| map.is_empty())
}

impl Role {
    /// UI baseline used when seeding/resetting a plugin's `plugin_state`: `author.suggested_ui` (if non-empty) takes precedence, otherwise `ui.json`.
    #[must_use]
    pub fn plugin_state_ui_baseline(&self) -> &UiConfig {
        if let Some(ref ap) = self.author_pack {
            if let Some(ref sug) = ap.suggested_ui {
                if !sug.is_effectively_empty() {
                    return sug;
                }
            }
        }
        &self.ui_config
    }

    /// Initial favorability configured in the role pack for an identity id; unknown identities fall back to 50.
    #[must_use]
    pub fn initial_favorability_for_relation(&self, relation_id: &str) -> f64 {
        self.user_relations
            .iter()
            .find(|r| r.id == relation_id)
            .map(UserRelation::initial_favorability_clamped)
            .unwrap_or(50.0)
    }

    /// Resolve the Ollama model name for this role (manifest → env → global fallback policy chain).
    #[must_use]
    pub fn resolve_ollama_model(&self, global_fallback: &str) -> String {
        if let Some(ref m) = self.ollama_model {
            let t = m.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
        if let Ok(v) = std::env::var("OLLAMA_MODEL") {
            let t = v.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
        global_fallback.to_string()
    }
}
