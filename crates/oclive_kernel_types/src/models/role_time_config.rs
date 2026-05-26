//! 角色包 `config.json` → `time` 段（虚拟时钟与遗忘梯度）。

use serde::{Deserialize, Serialize};

/// 现实时间与虚拟时间的流速比：现实 1 分钟 → 虚拟 `speed` 分钟（默认 **5**）。
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

/// `roles/<id>/config.json` 中的 `time` 对象。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleTimeConfig {
    /// 现实:虚拟 分钟比（1 现实分钟 = `speed` 虚拟分钟）。
    #[serde(default = "default_speed")]
    pub speed: f64,
    /// 虚拟时间大跳转时是否叠加遗忘（性格 delta 向 0 衰减、记忆权重衰减）。
    #[serde(default)]
    pub decay_on_jump: bool,
    /// 每虚拟日的性格 delta 衰减强度（乘在 `0.95^days` 的指数上，越大遗忘越快）。
    #[serde(default = "default_decay_per_day")]
    pub decay_per_day: f64,
    /// 每虚拟日的记忆权重衰减强度（同上，用于 `MemoryEngine::decay_weight`）。
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

