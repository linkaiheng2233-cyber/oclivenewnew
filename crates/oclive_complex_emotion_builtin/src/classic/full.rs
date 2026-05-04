//! 从七维情绪分数推导复杂情感模块使用的效价 / 掌控感估计。

use oclive_kernel_core::models::EmotionResult;

#[must_use]
pub fn affect_metrics_from_seven_dim(er: &EmotionResult) -> (f64, f64) {
    let v = er.joy + er.surprise * 0.25
        - er.sadness
        - er.anger * 0.6
        - er.fear * 0.4
        - er.disgust * 0.35;
    let d = er.joy * 0.35 + er.neutral * 0.15 - er.fear * 0.55 - er.sadness * 0.25;
    (v.clamp(-1.0, 1.0), d.clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affect_metrics_range() {
        let er = EmotionResult {
            joy: 0.8,
            sadness: 0.1,
            anger: 0.0,
            fear: 0.05,
            surprise: 0.1,
            disgust: 0.0,
            neutral: 0.2,
        };
        let (v, d) = affect_metrics_from_seven_dim(&er);
        assert!(v > 0.0 && v <= 1.0);
        assert!((-1.0..=1.0).contains(&d));
    }
}
