use serde::{Deserialize, Serialize};
use std::fmt;

/// Discrete role-facing emotion label used in prompts and UI.
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

impl std::str::FromStr for Emotion {
    type Err = ();

    /// Parses the lowercase display form produced by [`Display`](fmt::Display).
    ///
    /// Unknown or mixed-case tokens return `Err(())` so callers can fall back
    /// (e.g. degraded keep keeps the previous emotion or defaults to neutral).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "happy" => Ok(Self::Happy),
            "sad" => Ok(Self::Sad),
            "angry" => Ok(Self::Angry),
            "neutral" => Ok(Self::Neutral),
            "excited" => Ok(Self::Excited),
            "confused" => Ok(Self::Confused),
            "shy" => Ok(Self::Shy),
            _ => Err(()),
        }
    }
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
