//! 人格轴与表达驱动：事件影响与立绘纠偏共用，避免两处公式漂移。

use crate::models::PersonalityVector;

/// 轻量三轴：softness / coldness / volatility（与立绘纠偏一致）。
#[inline]
pub fn softness_coldness_volatility(personality: &PersonalityVector) -> (f64, f64, f64) {
    let softness = (personality.warmth * 0.48
        + personality.forgiveness * 0.32
        + personality.clinginess * 0.20)
        .clamp(0.0, 1.0);
    let coldness = (personality.stubbornness * 0.45
        + personality.assertiveness * 0.35
        + (1.0 - personality.warmth) * 0.20)
        .clamp(0.0, 1.0);
    let volatility =
        (personality.sensitivity * 0.6 + personality.talkativeness * 0.4).clamp(0.0, 1.0);
    (softness, coldness, volatility)
}

/// 戒备态驱动（不新增标签，仅细化中间态）。
#[inline]
pub fn guarded_drive(personality: &PersonalityVector) -> f64 {
    (personality.assertiveness * 0.46
        + personality.stubbornness * 0.34
        + (1.0 - personality.warmth) * 0.10
        + (1.0 - personality.forgiveness) * 0.10)
        .clamp(0.0, 1.0)
}

/// 受伤态驱动。
#[inline]
pub fn hurt_drive(personality: &PersonalityVector) -> f64 {
    (personality.sensitivity * 0.48
        + personality.clinginess * 0.26
        + personality.warmth * 0.14
        + (1.0 - personality.assertiveness) * 0.12)
        .clamp(0.0, 1.0)
}

/// 试探态驱动。
#[inline]
pub fn probing_drive(personality: &PersonalityVector) -> f64 {
    (personality.sensitivity * 0.32
        + personality.talkativeness * 0.24
        + personality.clinginess * 0.24
        + (1.0 - personality.assertiveness) * 0.20)
        .clamp(0.0, 1.0)
}
