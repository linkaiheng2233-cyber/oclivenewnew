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
        let mut max_emotion = Emotion::Neutral;
        let mut max_value = result.neutral;

        if result.joy > max_value {
            max_value = result.joy;
            max_emotion = Emotion::Happy;
        }
        if result.sadness > max_value {
            max_value = result.sadness;
            max_emotion = Emotion::Sad;
        }
        if result.disgust > max_value {
            max_value = result.disgust;
            max_emotion = Emotion::Angry;
        }
        if result.anger > max_value {
            max_value = result.anger;
            max_emotion = Emotion::Angry;
        }
        if result.surprise > max_value {
            max_value = result.surprise;
            max_emotion = Emotion::Excited;
        }
        if result.fear > max_value {
            max_emotion = Emotion::Confused;
        }

        max_emotion
    }

    #[must_use]
    pub fn calculate_intensity(result: &EmotionResult) -> f64 {
        let max_emotion = Self::get_dominant_emotion(result);
        match max_emotion {
            Emotion::Happy => result.joy,
            Emotion::Sad => result.sadness,
            Emotion::Angry => result.anger.max(result.disgust),
            Emotion::Excited => result.surprise,
            Emotion::Confused => result.fear,
            Emotion::Shy => result.fear,
            Emotion::Neutral => result.neutral,
        }
    }

    #[must_use]
    pub fn format_for_prompt(result: &EmotionResult) -> String {
        let dominant = Self::get_dominant_emotion(result);
        let intensity = Self::calculate_intensity(result);
        format!(
            "[emotion dominant=\"{}\" intensity=\"{:.2}\"]",
            dominant, intensity
        )
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
