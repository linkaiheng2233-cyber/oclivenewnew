//! 人格轴：与 runtime `affect_policy::softness_coldness_volatility` 公式一致（事件道歉策略共用）。

use oclive_kernel_models::PersonalityVector;

/// 轻量三轴：softness / coldness / volatility。
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
