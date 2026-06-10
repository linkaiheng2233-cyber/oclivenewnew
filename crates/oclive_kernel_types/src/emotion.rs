//! Emotion-analysis result (seven-dimension distribution).

use crate::models::Emotion;
use crate::SlotExtension;

/// Emotion-analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmotionResult {
    pub joy: f64,
    pub sadness: f64,
    pub anger: f64,
    pub fear: f64,
    pub surprise: f64,
    pub disgust: f64,
    pub neutral: f64,
    /// Optional plugin-specific extension envelope (kernel does not interpret `data`).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<SlotExtension>,
}

impl EmotionResult {
    /// Returns the emotion label corresponding to the largest of the seven dimensions (matching the builtin `EmotionAnalyzer::get_dominant_emotion` rule).
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

    /// Convert to `models::Emotion` (kept for compatibility with the legacy API name).
    #[must_use]
    pub fn to_emotion(&self) -> Emotion {
        self.dominant_emotion()
    }
}
