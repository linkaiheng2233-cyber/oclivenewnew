use oclive_validation::SceneContinuityConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DiskSceneConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub welcome_message: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub monologues: Vec<String>,
    #[serde(default)]
    pub time_windows: Vec<DiskSceneTimeWindow>,
    /// Short material when the role is in this scene and the user messages from another scene (may be combined with `away_life.txt`).
    #[serde(default)]
    pub away_life_notes: Vec<String>,
    /// Trajectory material overrides keyed by user conversation-context scene id.
    #[serde(default)]
    pub away_life_by_user_scene: HashMap<String, String>,
    /// Optional narrative-continuity presets and deterministic transitions.
    #[serde(default)]
    pub continuity: Option<SceneContinuityConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DiskSceneTimeWindow {
    pub start: String,
    pub end: String,
}
