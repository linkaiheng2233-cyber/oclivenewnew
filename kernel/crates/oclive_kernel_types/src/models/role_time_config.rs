//! Role pack `config.json` → `time` section (virtual clock and forgetting gradients).

use serde::{Deserialize, Serialize};

/// Real-to-virtual time flow ratio: 1 real minute → `speed` virtual minutes (default **5**).
pub const DEFAULT_REAL_TO_VIRTUAL_RATIO: f64 = 5.0;

fn default_speed() -> f64 {
    DEFAULT_REAL_TO_VIRTUAL_RATIO
}

fn default_decay_per_day() -> f64 {
    1.0
}

fn default_memory_decay_per_day() -> f64 {
    1.0
}

/// The `time` object in `roles/<id>/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleTimeConfig {
    /// Real:virtual minute ratio (1 real minute = `speed` virtual minutes).
    #[serde(default = "default_speed")]
    pub speed: f64,
    /// When true, large virtual time jumps also apply forgetting (personality delta decays toward 0, memory weight decays).
    #[serde(default)]
    pub decay_on_jump: bool,
    /// Personality delta decay strength per virtual day (multiplies the `0.95^days` exponent; higher = faster forgetting).
    #[serde(default = "default_decay_per_day")]
    pub decay_per_day: f64,
    /// Memory weight decay strength per virtual day (same as above; used by `MemoryEngine::decay_weight`).
    #[serde(default = "default_memory_decay_per_day")]
    pub memory_decay_per_day: f64,
}

impl Default for RoleTimeConfig {
    fn default() -> Self {
        Self {
            speed: DEFAULT_REAL_TO_VIRTUAL_RATIO,
            decay_on_jump: false,
            decay_per_day: 1.0,
            memory_decay_per_day: 1.0,
        }
    }
}

impl RoleTimeConfig {
    #[must_use]
    pub fn effective_ratio(&self) -> f64 {
        if self.speed.is_finite() && self.speed > 0.0 {
            self.speed
        } else {
            DEFAULT_REAL_TO_VIRTUAL_RATIO
        }
    }
}
