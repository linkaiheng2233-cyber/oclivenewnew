//! 策略配置与上下文（纯数据结构）。

use crate::models::Event;
use serde::Deserialize;

/// Settings for emotion hold/neutral behavior in the emotion policy port.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EmotionPolicyConfig {
    pub neutral_hold_enabled: bool,
    pub low_confidence_hold_threshold: f64,
}

impl Default for EmotionPolicyConfig {
    fn default() -> Self {
        Self {
            neutral_hold_enabled: true,
            low_confidence_hold_threshold: 0.6,
        }
    }
}

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
