//! 复杂情感：输入输出与 Provider trait（实现留在 runtime）。

use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexEmotionInput {
    pub role_id: String,
    pub scene_id: String,
    pub user_message: String,
    pub bot_reply: String,
    #[serde(default)]
    pub recent_dialogue_summary: Option<String>,
    #[serde(default)]
    pub previous_narrative_hint: String,
    #[serde(default)]
    pub user_valence: Option<f64>,
    #[serde(default)]
    pub user_dominance: Option<f64>,
    #[serde(default)]
    pub previous_user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexEmotionOutput {
    pub source: String,
    #[serde(default)]
    pub narrative_hint: Option<String>,
    pub labels: Vec<String>,
    pub pattern: Option<String>,
    pub confidence: f64,
    pub intensity: f64,
    pub dissonance_score: f64,
    pub degraded_to_builtin: bool,
}

pub trait ComplexEmotionProvider: Send + Sync {
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput>;
}
