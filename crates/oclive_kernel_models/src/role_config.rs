//! 角色包 `settings.json` / manifest 中与性格演化、记忆话题、用户身份相关的纯配置。

use oclive_validation::PersonalitySource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
