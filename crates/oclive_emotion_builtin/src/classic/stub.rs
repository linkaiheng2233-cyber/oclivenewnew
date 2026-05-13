//! `classic` 关闭：与 `DisabledUserEmotionAnalyzer` 对齐的强中性七维，无关键词表。

use oclive_kernel_core::error::Result;
use oclive_kernel_core::models::{Emotion, EmotionResult};

/// 情绪分析器（桩）
pub struct EmotionAnalyzer;

impl EmotionAnalyzer {
    /// 恒返回归一化强中性（空文本与非空一致语义：无关键词信号）。
    pub fn analyze(_text: &str) -> Result<EmotionResult> {
        Ok(EmotionResult::strong_neutral())
    }

    #[must_use]
    pub fn get_dominant_emotion(result: &EmotionResult) -> Emotion {
        result.dominant_emotion()
    }

    #[must_use]
    pub fn calculate_intensity(result: &EmotionResult) -> f64 {
        result.dominant_intensity()
    }

    #[must_use]
    pub fn format_for_prompt(result: &EmotionResult) -> String {
        result.format_emotion_for_prompt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_neutral_analyze() {
        let r = EmotionAnalyzer::analyze("我很开心").unwrap();
        assert_eq!(r.neutral, 1.0);
        assert_eq!(EmotionAnalyzer::get_dominant_emotion(&r), Emotion::Neutral);
    }
}
