//! 情绪分析结果（七维分布）。

use crate::models::Emotion;

/// 情绪分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// 取七维中最大值对应的情绪标签（与内置 `EmotionAnalyzer::get_dominant_emotion` 规则一致）。
    #[must_use]
    pub fn dominant_emotion(&self) -> Emotion {
        let emotions = [
            (self.joy, Emotion::Happy),
            (self.sadness, Emotion::Sad),
            (self.anger, Emotion::Angry),
            (self.fear, Emotion::Confused),
            (self.surprise, Emotion::Excited),
            (self.disgust, Emotion::Confused),
        ];

        let best = emotions
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, e)| (*s, e.clone()))
            .unwrap_or((0.0, Emotion::Neutral));
        if self.neutral > best.0 {
            Emotion::Neutral
        } else {
            best.1
        }
    }

    /// 转为 `models::Emotion`（兼容旧 API 命名）。
    #[must_use]
    pub fn to_emotion(&self) -> Emotion {
        self.dominant_emotion()
    }
}
