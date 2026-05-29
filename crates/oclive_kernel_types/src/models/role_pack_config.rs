//! 角色包 `config.json`（虚拟时间、记忆遗忘、关系疏远）。

use serde::{Deserialize, Serialize};

use super::role_time_config::RoleTimeConfig;

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

/// `config.json` → `evolution`（虚拟时间驱动的阶段性性格沉淀）
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
    /// 每次实际互动时，在疏远衰减后按 `(1 + recovery)` 回升亲密值。
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

/// `config.json` → `chat_storage`（聊天记录单会话条数上限等）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RolePackChatStorageConfig {
    /// 单会话最多保留消息条数（user+assistant 合计）；未设则用宿主默认 500。
    #[serde(default)]
    pub max_messages_per_session: Option<u32>,
    /// 保留最近 N 天内更新的会话；超出自动清理。未设则不启用。
    #[serde(default)]
    pub auto_cleanup_days: Option<u32>,
    /// 每个角色最多保留 N 个会话；超出删除最旧。未设则不启用。
    #[serde(default)]
    pub auto_cleanup_max_sessions: Option<u32>,
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
}
