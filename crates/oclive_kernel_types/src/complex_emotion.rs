//! Complex-emotion resolution input/output (pure data structures).

use crate::SlotExtension;
use serde::{Deserialize, Serialize};

/// Aligned with the `params` of the JSON-RPC `complex_emotion.resolve_turn` (snake_case).
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
    /// Derived by the host from the user message's seven-dimension emotion; when absent, keyword conditions treat it as 0.
    #[serde(default)]
    pub user_valence: Option<f64>,
    #[serde(default)]
    pub user_dominance: Option<f64>,
    /// Previous-turn user message (plain text); used for the "two consecutive user replies of ≤2 characters" check.
    #[serde(default)]
    pub previous_user_message: Option<String>,
}

/// Consistent with the sidecar / prompt injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexEmotionOutput {
    pub source: String,
    pub narrative_hint: String,
    pub labels: Vec<String>,
    pub pattern: Option<String>,
    pub confidence: f64,
    pub intensity: f64,
    pub dissonance_score: f64,
    pub degraded_to_builtin: bool,
    /// Optional plugin-specific extension envelope (kernel does not interpret `data`).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<SlotExtension>,
}
