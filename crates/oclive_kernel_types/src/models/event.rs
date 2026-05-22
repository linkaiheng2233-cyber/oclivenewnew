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

/// Detected event with user and bot emotion labels at detection time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub user_emotion: String,
    pub bot_emotion: String,
}
