use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::knowledge::KnowledgeIndex;
use super::plugin_backends::PluginBackends;
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
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            event_impact_factor: 1.0,
            ai_analysis_interval: 15,
            max_change_per_event: 0.05,
            max_total_change: 0.5,
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

/// 用户身份与场景的关系：由角色包 manifest `identity_binding` 决定。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdentityBinding {
    /// 全剧单一身份：忽略 `role_scene_identity`，仅全局 `user_relation` / manifest 默认。
    Global,
    /// 每个场景可单独覆盖身份（`role_scene_identity` 优先于全局）。
    #[default]
    PerScene,
}

impl UserRelation {
    #[must_use]
    pub fn initial_favorability_clamped(&self) -> f64 {
        self.initial_favorability.clamp(0.0, 100.0)
    }
}

/// 角色包 `manifest.json` 中「异地生活轨迹」创作者设定（可选）。
/// 模式开关（`default_enabled`）放在 `settings.json` 的 `remote_presence`。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifeTrajectoryDisk {
    /// 异地时角色日常节奏、碎碎念风格、与用户「心有灵犀」的总体说明（注入异地心声 LLM，模型依人设延伸现编；总述为写法与气质约定，非固定台词）
    #[serde(default)]
    pub summary: Option<String>,
    /// 与 `summary` 二选一：`summary_lines` 非空时优先按段拼接（manifest 里用 JSON 数组多行书写，便于阅读）
    #[serde(default)]
    pub summary_lines: Vec<String>,
    /// 可选：固定 OOC 括注；与 `stub_messages` 同时配置时由引擎拼接为「stub_ooc + ， + 旁白」（不需要此结构时勿填）
    #[serde(default)]
    pub stub_ooc: Option<String>,
    /// 未配置 `stub_ooc`：每元素为**整段占位**（创作者自定）。配置了 `stub_ooc`：每元素为**仅旁白句**（轮换）。
    #[serde(default)]
    pub stub_messages: Vec<String>,
}

impl LifeTrajectoryDisk {
    /// 合并 `summary_lines` 与单字段 `summary`（`summary_lines` 非空时优先）。
    #[must_use]
    pub fn effective_summary(&self) -> Option<String> {
        if !self.summary_lines.is_empty() {
            let joined = self
                .summary_lines
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            if !joined.is_empty() {
                return Some(joined);
            }
        }
        self.summary
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

/// 日程片段：忙碌程度（供 prompt / UI；不直接改好感数值）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LifeAvailability {
    #[default]
    Free,
    Distracted,
    Busy,
}

/// 单条周内重复日程（本地时刻由 `LifeScheduleDisk::timezone_offset_minutes` 换算）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeScheduleEntryDisk {
    /// 1 = 周一 … 7 = 周日（与 `chrono::Weekday::number_from_monday()` 一致）
    pub weekday: u8,
    /// 开始时刻 `HH:MM`（24 小时制）
    pub time_start: String,
    /// 结束时刻 `HH:MM`。若 **小于** `time_start`，表示窗口跨午夜（与 `hour_start`/`hour_end` 自主换场景规则一致）
    pub time_end: String,
    /// 机器可读活动键，如 `work` / `school`
    pub activity_id: String,
    /// 人类可读短标签，注入提示与 UI
    pub label: String,
    #[serde(default)]
    pub preferred_scene_id: Option<String>,
    #[serde(default)]
    pub availability: Option<LifeAvailability>,
}

/// 创作者配置的「日常时间安排」（虚拟时间 → 当前活动）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifeScheduleDisk {
    /// 相对 UTC 的分钟偏移，用于把 `virtual_time_ms` 换算成角色本地星期与时刻；省略则按 UTC
    #[serde(default)]
    pub timezone_offset_minutes: Option<i32>,
    #[serde(default)]
    pub entries: Vec<LifeScheduleEntryDisk>,
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
    /// 为 true 时默认不出现在 `list_roles`（仓库内调试/身份示例包）；`load_role` 仍可按 id 加载。见环境变量 `OCLIVE_LIST_DEV_ROLES`。
    #[serde(default)]
    pub dev_only: bool,
    /// `settings.json` → `plugin_backends`（可选；默认全 builtin）
    #[serde(default)]
    pub plugin_backends: PluginBackends,
    /// `knowledge/` 加载后的索引（仅内存；由 [`crate::infrastructure::storage::RoleStorage`] 填充）
    #[serde(skip)]
    pub knowledge_index: Option<Arc<KnowledgeIndex>>,
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
            dev_only: false,
            plugin_backends: PluginBackends::default(),
            knowledge_index: None,
        }
    }
}

impl Role {
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
