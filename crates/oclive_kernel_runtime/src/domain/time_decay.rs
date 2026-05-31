//! Virtual-time forgetting: personality delta shrinkage and memory weight decay.

use oclive_kernel_types::{Memory, PersonalityVector};

use super::memory_engine::MemoryEngine;
use super::virtual_time::virtual_days_between_ms;

/// Per-virtual-day decay base (`0.95`), with the exponent scaled by the configured `decay_per_day`.
fn decay_factor_for_virtual_days(virtual_days: f64, decay_strength_per_day: f64) -> f64 {
    if virtual_days <= 0.0 {
        return 1.0;
    }
    let strength = decay_strength_per_day.max(0.0);
    let exponent = virtual_days * strength;
    0.95_f64.powf(exponent).clamp(0.05, 1.0)
}

/// Shrink the personality **delta** toward 0 (time-based forgetting, without changing the pack's core).
#[must_use]
pub fn decay_personality_delta(
    mut delta: PersonalityVector,
    virtual_days: f64,
    decay_per_day: f64,
) -> PersonalityVector {
    let factor = decay_factor_for_virtual_days(virtual_days, decay_per_day);
    delta.scale_components(factor);
    delta
}

/// Decay `weight` by the interval between the memory's creation time and the current virtual time.
#[must_use]
pub fn decay_memory_for_virtual_age(
    memory: Memory,
    memory_created_ms: i64,
    virtual_now_ms: i64,
    memory_decay_per_day: f64,
) -> Memory {
    let days = virtual_days_between_ms(memory_created_ms, virtual_now_ms);
    if days <= 0.0 {
        return memory;
    }
    let strength = memory_decay_per_day.max(0.0);
    let effective_days = days * strength;
    MemoryEngine::decay_weight(memory, effective_days)
}
