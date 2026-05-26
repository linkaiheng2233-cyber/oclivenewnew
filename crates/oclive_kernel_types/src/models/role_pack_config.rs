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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RolePackConfigFile {
    #[serde(default)]
    pub time: RoleTimeConfig,
    #[serde(default)]
    pub memory: RolePackMemoryConfig,
    #[serde(default)]
    pub relation: RolePackRelationConfig,
}
