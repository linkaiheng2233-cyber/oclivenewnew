use serde::{Deserialize, Serialize};

use super::emotion::Emotion;

/// 情绪分析结果（七维），与 `EmotionAnalyzer` 输出一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionResult {
    pub joy: f64,
    pub sadness: f64,
    pub anger: f64,
    pub fear: f64,
    pub surprise: f64,
    pub disgust: f64,
    pub neutral: f64,
}

impl EmotionResult {
    /// 强中性七维分布（与 stub / 禁用分析器 / `UserEmotionAnalyzer` V2 占位一致）。
    /// 纯 `f64` 字面量，无 FFI；可在 Miri 下覆盖相关单元测试（与 `http_api` 中 `libc::statvfs` 等主机调用不同）。
    #[must_use]
    pub const fn strong_neutral() -> Self {
        Self {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        }
    }

    /// 七维分布上的主导离散标签（与 `oclive_emotion_builtin::classic::EmotionAnalyzer::get_dominant_emotion` 历史行为一致）。
    #[must_use]
    pub fn dominant_emotion(&self) -> Emotion {
        let mut max_emotion = Emotion::Neutral;
        let mut max_value = self.neutral;

        if self.joy > max_value {
            max_value = self.joy;
            max_emotion = Emotion::Happy;
        }
        if self.sadness > max_value {
            max_value = self.sadness;
            max_emotion = Emotion::Sad;
        }
        if self.disgust > max_value {
            max_value = self.disgust;
            max_emotion = Emotion::Angry;
        }
        if self.anger > max_value {
            max_value = self.anger;
            max_emotion = Emotion::Angry;
        }
        if self.surprise > max_value {
            max_value = self.surprise;
            max_emotion = Emotion::Excited;
        }
        if self.fear > max_value {
            max_emotion = Emotion::Confused;
        }

        max_emotion
    }

    /// 与主导标签对应通道上的强度（0~1）。
    #[must_use]
    pub fn dominant_intensity(&self) -> f64 {
        let max_emotion = self.dominant_emotion();
        match max_emotion {
            Emotion::Happy => self.joy,
            Emotion::Sad => self.sadness,
            Emotion::Angry => self.anger.max(self.disgust),
            Emotion::Excited => self.surprise,
            Emotion::Confused => self.fear,
            Emotion::Shy => self.fear,
            Emotion::Neutral => self.neutral,
        }
    }

    /// 注入 Prompt 的情绪属性行（与历史 `format_for_prompt` 输出一致）。
    #[must_use]
    pub fn format_emotion_for_prompt(&self) -> String {
        let dominant = self.dominant_emotion();
        let intensity = self.dominant_intensity();
        format!(
            "[emotion dominant=\"{}\" intensity=\"{:.2}\"]",
            dominant, intensity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::emotion::Emotion;
    use super::EmotionResult;

    #[test]
    fn strong_neutral_is_normalized() {
        let r = EmotionResult::strong_neutral();
        assert_eq!(r.neutral, 1.0);
        assert_eq!(
            r.joy + r.sadness + r.anger + r.fear + r.surprise + r.disgust,
            0.0
        );
    }

    #[test]
    fn dominant_emotion_prefers_joy_when_higher_than_neutral() {
        let r = EmotionResult {
            joy: 0.6,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.5,
        };
        assert_eq!(r.dominant_emotion(), Emotion::Happy);
    }

    #[test]
    fn format_emotion_for_prompt_includes_tag() {
        let r = EmotionResult::strong_neutral();
        let s = r.format_emotion_for_prompt();
        assert!(s.contains("dominant=\"neutral\""));
    }
}
