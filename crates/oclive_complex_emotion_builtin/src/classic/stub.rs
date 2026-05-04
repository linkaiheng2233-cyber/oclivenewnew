//! `classic` 关闭：不向复杂情感注入基于七维的估计。

use oclive_kernel_core::models::EmotionResult;

#[must_use]
pub fn affect_metrics_from_seven_dim(_er: &EmotionResult) -> (f64, f64) {
    (0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_zero() {
        let er = EmotionResult {
            joy: 0.8,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.2,
        };
        assert_eq!(affect_metrics_from_seven_dim(&er), (0.0, 0.0));
    }
}
