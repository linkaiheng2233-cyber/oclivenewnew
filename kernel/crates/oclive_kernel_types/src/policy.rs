//! Policy configuration and context (pure data structures).

use crate::models::Event;
use serde::Deserialize;

/// Emotion policy configuration.
///
/// B M1 slice 2: hold fields removed — the main LLM is the sole arbiter of
/// complex emotion (v1.5 §11.1). Kept as an empty config type so the public
/// `PolicyConfig.emotion` contract stays stable.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EmotionPolicyConfig {}

/// Settings for memory filtering, importance, and FIFO cap in memory policy.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryPolicyConfig {
    pub ignore_single_char_filter: bool,
    pub default_importance: f64,
    pub fifo_limit: i32,
}

impl Default for MemoryPolicyConfig {
    fn default() -> Self {
        Self {
            ignore_single_char_filter: true,
            default_importance: 0.5,
            fifo_limit: 500,
        }
    }
}

/// Bundled emotion and memory policy configuration from role settings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PolicyConfig {
    pub emotion: EmotionPolicyConfig,
    pub memory: MemoryPolicyConfig,
}

/// Per-turn inputs passed into memory policy decisions.
pub struct PolicyContext<'a> {
    pub role_id: &'a str,
    pub user_message: &'a str,
    pub reply: &'a str,
    pub event: &'a Event,
    pub event_confidence: f32,
}
