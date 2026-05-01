use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
