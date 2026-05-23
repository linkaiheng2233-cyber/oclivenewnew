use serde::{Deserialize, Serialize};

/// Class of in-dialogue event detected by the event policy port.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Quarrel,
    Apology,
    Praise,
    Complaint,
    Confession,
    Joke,
    Ignore,
}

impl AsRef<str> for EventType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Quarrel => "Quarrel",
            Self::Apology => "Apology",
            Self::Praise => "Praise",
            Self::Complaint => "Complaint",
            Self::Confession => "Confession",
            Self::Joke => "Joke",
            Self::Ignore => "Ignore",
        }
    }
}

/// Detected event with user and bot emotion labels at detection time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub user_emotion: String,
    pub bot_emotion: String,
}
