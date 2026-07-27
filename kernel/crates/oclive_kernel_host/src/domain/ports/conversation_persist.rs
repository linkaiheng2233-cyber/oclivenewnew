//! Chat history append port (decouples domain from `TurnPersistInput`).

use crate::error::Result;
use async_trait::async_trait;

/// Result of appending one user/assistant turn to chat storage.
#[derive(Debug, Clone)]
pub struct TurnAppendResult {
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub user_message_timestamp: String,
    pub assistant_message_timestamp: String,
}

/// Auto-cleanup policy snapshot for async post-append maintenance.
#[derive(Debug, Clone, Default)]
pub struct TurnAutoCleanupConfig {
    pub auto_cleanup_days: Option<u32>,
    pub auto_cleanup_max_sessions: Option<u32>,
    pub chat_storage_location: String,
}

impl TurnAutoCleanupConfig {
    #[must_use]
    pub fn from_role_config(cfg: &crate::models::RolePackChatStorageConfig) -> Self {
        Self {
            auto_cleanup_days: cfg.auto_cleanup_days,
            auto_cleanup_max_sessions: cfg.auto_cleanup_max_sessions,
            chat_storage_location: cfg.location.clone(),
        }
    }
}

/// Domain request to append one turn to chat storage.
#[derive(Debug, Clone)]
pub struct TurnPersistRequest {
    /// Stable key for retry-safe appends. Omitted for ordinary foreground turns.
    pub idempotency_key: Option<String>,
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub user_message: String,
    pub user_message_hidden: bool,
    pub assistant_reply: String,
    pub reply_is_fallback: bool,
    pub model_name: Option<String>,
    pub response_ms: u64,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
    pub max_messages_per_session: Option<u32>,
    pub auto_cleanup_config: TurnAutoCleanupConfig,
    pub chat_storage_location: String,
}

/// Appends chat turns to the configured conversation store.
#[async_trait]
pub trait ConversationPersistPort: Send + Sync {
    /// # Errors
    ///
    /// Storage backend failures.
    async fn append_turn(&self, input: TurnPersistRequest) -> Result<TurnAppendResult>;
}
