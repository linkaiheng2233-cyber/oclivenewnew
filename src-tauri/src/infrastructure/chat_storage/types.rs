//! DTOs for chat storage API and `ConversationStore`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppendTurnResult {
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub user_message_timestamp: String,
    pub assistant_message_timestamp: String,
}

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
    /// Per-role cap; `None` uses [`super::config::DEFAULT_MAX_MESSAGES`].
    #[serde(default)]
    pub max_messages_per_session: Option<u32>,
    /// Auto-cleanup policy snapshot (from role pack); applied async after append.
    #[serde(default)]
    pub auto_cleanup_config: super::cleanup::AutoCleanupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportChatBucket {
    pub role_id: String,
    pub scene_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub messages: Vec<ImportChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportChatBucketsResult {
    pub buckets_imported: u32,
    pub turns_imported: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStorageStat {
    pub scene_id: String,
    pub session_count: u32,
    pub total_size_bytes: u64,
    pub last_active: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleStorageStat {
    pub role_id: String,
    pub total_size_bytes: u64,
    pub scene_count: u32,
    pub last_active: Option<String>,
    pub scenes: Vec<SceneStorageStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteChatsResult {
    pub sessions_deleted: u32,
    pub bytes_freed: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSearchResult {
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub message: StoredMessage,
    pub highlight_snippet: String,
    #[serde(default)]
    pub context_before: Vec<StoredMessage>,
    #[serde(default)]
    pub context_after: Vec<StoredMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoCleanupResult {
    pub sessions_deleted: u32,
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatExportResponse {
    pub content: String,
    pub suggested_filename: String,
    pub mime_type: String,
    #[serde(default)]
    pub content_encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayTarget {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    /// Optional dedupe threshold; unset uses role pack config or default 0.6.
    #[serde(default)]
    pub similarity_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayResult {
    pub total_turns: u32,
    pub new_memories: u32,
    pub updated_memories: u32,
    pub skipped_memories: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayProgress {
    pub task_id: String,
    pub percent: u8,
    pub processed_turns: u32,
    pub total_turns: u32,
    pub new_memories: u32,
    pub updated_memories: u32,
    pub skipped_memories: u32,
    pub done: bool,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStorageCapabilities {
    pub backend_kind: String,
    pub supports_search: bool,
    pub supports_replay: bool,
    pub supports_cleanup: bool,
}
