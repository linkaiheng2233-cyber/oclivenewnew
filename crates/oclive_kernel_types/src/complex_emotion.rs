//! 复杂情感解析输入/输出（纯数据结构）。

use serde::{Deserialize, Serialize};

/// 与 JSON-RPC `complex_emotion.resolve_turn` 的 `params` 对齐（snake_case）。
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
    /// 宿主从用户句七维情绪推导；缺省按 0 处理关键词条件。
    #[serde(default)]
    pub user_valence: Option<f64>,
    #[serde(default)]
    pub user_dominance: Option<f64>,
    /// 上一轮用户句（纯文本）；用于「连续两轮用户回复≤2字」判定。
    #[serde(default)]
    pub previous_user_message: Option<String>,
}

/// 与侧车 / Prompt 注入一致。
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
}
