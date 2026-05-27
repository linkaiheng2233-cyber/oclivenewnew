//! DTOs for chat storage API and `ConversationStore`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnPersistInput {
    /// SQLite / session namespace (`srid`: manifest id or `role__sess__*`).
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub user_message: String,
    pub assistant_reply: String,
    pub reply_is_fallback: bool,
    pub model_name: Option<String>,
    pub response_ms: u64,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
    pub last_message_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub session_id: String,
    pub turn_index: i32,
    pub sender: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}
