use serde::{Deserialize, Serialize};
use std::fmt;

/// UI 向离散情绪标签（与七维 `EmotionResult` 映射见 `oclive_emotion_builtin::classic::EmotionAnalyzer`）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Emotion {
    Happy,
    Sad,
    Angry,
    Neutral,
    Excited,
    Confused,
    Shy,
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Emotion::Happy => "happy",
            Emotion::Sad => "sad",
            Emotion::Angry => "angry",
            Emotion::Neutral => "neutral",
            Emotion::Excited => "excited",
            Emotion::Confused => "confused",
            Emotion::Shy => "shy",
        };
        write!(f, "{}", s)
    }
}
