use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::author_pack::AuthorPackFile;
use super::knowledge::KnowledgeIndex;
use super::plugin_backends::PluginBackends;
use super::ui_config::UiConfig;
pub use oclive_validation::{
    IdentityBinding, LifeAvailability, LifeScheduleDisk, LifeScheduleEntryDisk, LifeTrajectoryDisk,
    PersonalitySource,
};
use std::sync::Arc;

/// 角色包内人设默认值（旧七维，与 `PersonalityVector` 字段一致）
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
    /// `vector`：沿用七维增量；`profile`：以核心性格档案 + 运行时「可变性格档案」（**仅由 LLM 根据对话维护**）为准；七维为由正文归纳的**视图**，仅供理解与 UI。
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
    /// 选择该身份时的起始好感度（0～100）；未写则 50。
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

/// 由虚拟时间解析得到的当前生活态（引擎内部）
#[derive(Debug, Clone, PartialEq)]
pub struct LifeState {
    pub label: String,
    pub activity_key: String,
    pub busy_level: f32,
    pub optional_scene_hint: Option<String>,
}

/// 虚拟时间跳转后可选评估的「角色自主换场景」规则（二期，创作者可选）。
/// 写在 `settings.json` → `autonomous_scene`。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutonomousSceneConfig {
    /// 在 `jump_time` 更新虚拟时间后按顺序匹配，首条命中则写入 `role_runtime.current_scene`（需 `storage.is_scene_time_allowed`）。
    #[serde(default)]
    pub on_virtual_time: Vec<AutonomousSceneRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousSceneRule {
    /// 角色当前须在此场景才评估本规则
    pub when_scene: String,
    /// 本地时刻小时 \[start, end\)（0–23）；`end` 可小于 `start` 表示跨午夜
    pub hour_start: u8,
    pub hour_end: u8,
    pub to_scene: String,
}

/// 角色包 `settings.json` 中「异地心声」**模式开关**（可选）。
/// 占位句与轨迹总述见 manifest 的 `life_trajectory`。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemotePresenceConfig {
    /// 安装或首次展示时 UI 是否默认勾选「异地心声」（仅提示；持久化以 DB 为准）
    #[serde(default)]
    pub default_enabled: Option<bool>,
    /// 已迁移至 manifest `life_trajectory.stub_messages`；反序列化仍可读以实现旧包兼容
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stub_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    /// **核心性格档案**：创作者与用户设定的固定人设；运行时 **AI 不得改写**（见 `mutable_profile_llm`），与可变档案共同构成完整人设。
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
    /// 角色包 `manifest` 中的 Ollama 模型名（与 `model` 键互通）；空则回退环境变量与全局默认
    #[serde(default)]
    pub ollama_model: Option<String>,
    /// 身份是否与场景绑定；默认 `per_scene` 与历史行为一致。
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    /// `manifest.json` 中 `life_trajectory`（可选）
    #[serde(default)]
    pub life_trajectory: Option<LifeTrajectoryDisk>,
    /// `manifest.json` 中 `life_schedule`（可选）：虚拟时间下的日常片段，与 `life_trajectory`（异地文案气质）并存
    #[serde(default)]
    pub life_schedule: Option<LifeScheduleDisk>,
    /// `settings.json` 中 `remote_presence`（可选，主要为 `default_enabled`）
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    /// `settings.json` 中 `autonomous_scene`（可选，虚拟时间驱动角色位移）
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    /// `settings.json` 可选：`immersive` | `pure_chat`；运行时持久化见 `role_runtime.interaction_mode`
    #[serde(default)]
    pub interaction_mode: Option<String>,
    /// 角色包 `manifest.min_runtime_version`：要求的最低 oclive 版本；省略表示不检查。
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    /// 为 true 时默认不出现在 `list_roles`（仓库内调试/身份示例包）；`load_role` 仍可按 id 加载。见环境变量 `OCLIVE_LIST_DEV_ROLES`。
    #[serde(default)]
    pub dev_only: bool,
    /// `settings.json` → `plugin_backends`（可选；默认全 builtin）
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// `knowledge/` 加载后的索引（仅内存；由 [`crate::infrastructure::storage::RoleStorage`] 填充）
    #[serde(skip)]
    pub knowledge_index: Option<Arc<KnowledgeIndex>>,
    /// 角色包 `ui.json`（仅内存；由 [`crate::infrastructure::storage::RoleStorage`] 填充）
    #[serde(skip)]
    pub ui_config: UiConfig,
    /// 角色包 `author.json`（可选；仅内存）
    #[serde(skip)]
    pub author_pack: Option<AuthorPackFile>,
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
            plugin_backends: PluginBackends::default(),
            knowledge_index: None,
            ui_config: UiConfig::default(),
            author_pack: None,
        }
    }
}

impl Role {
    /// 插件 `plugin_state` 种子/重置时使用的 UI 基线：`author.suggested_ui`（非空）优先，否则 `ui.json`。
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

    /// 某身份 id 在角色包中配置的初始好感度；未知身份回退 50。
    #[must_use]
    pub fn initial_favorability_for_relation(&self, relation_id: &str) -> f64 {
        self.user_relations
            .iter()
            .find(|r| r.id == relation_id)
            .map(UserRelation::initial_favorability_clamped)
            .unwrap_or(50.0)
    }

    /// 解析本角色应使用的 Ollama 模型：**manifest** → **`OLLAMA_MODEL`** → **全局默认**（`AppState` 启动配置）
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
