//! 对话事件类型与记录（规则检测 / 事件估计共用）。

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub user_emotion: String,
    pub bot_emotion: String,
}
