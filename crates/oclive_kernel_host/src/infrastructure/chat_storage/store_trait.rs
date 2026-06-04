//! [`ConversationStore`] — pluggable chat history backend.

use super::cleanup::AutoCleanupConfig;
use super::types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ImportChatBucket, ImportChatBucketsResult, ReplayProgress, ReplayResult, ReplayTarget,
    RoleStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
use crate::error::{AppError, Result};
use async_trait::async_trait;

/// Chat history persistence (SQLite, file, or hybrid).
#[async_trait]
pub trait ConversationStore: Send + Sync {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<AppendTurnResult>;
    async fn list_sessions(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>>;
    /// List all sessions for a role across scenes (memory replay `role` scope).
    async fn list_sessions_by_role(&self, role_id: &str) -> Result<Vec<SessionMeta>> {
        let _ = role_id;
        Err(AppError::InvalidParameter(
            "chat storage backend does not support list_sessions_by_role".into(),
        ))
    }
    async fn fetch_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>>;

    async fn rebuild_mirror(&self, session_id: &str, max_messages: i64) -> Result<String> {
        let _ = (session_id, max_messages);
        Err(AppError::InvalidParameter(
            "chat storage backend does not support rebuild_mirror".into(),
        ))
    }

    async fn import_chat_buckets(
        &self,
        _buckets: Vec<ImportChatBucket>,
    ) -> Result<ImportChatBucketsResult> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support import_chat_buckets".into(),
        ))
    }

    async fn search_messages(
        &self,
        _query: &str,
        _role_id: Option<&str>,
        _limit: u32,
        _offset: u32,
    ) -> Result<Vec<ChatSearchResult>> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support search_messages".into(),
        ))
    }

    async fn delete_message(&self, _message_id: &str) -> Result<()> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support delete_message".into(),
        ))
    }

    async fn edit_message(&self, _message_id: &str, _new_content: &str) -> Result<()> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support edit_message".into(),
        ))
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let _ = session_id;
        Err(AppError::InvalidParameter(
            "chat storage backend does not support delete_session".into(),
        ))
    }

    async fn export_session(
        &self,
        _session_id: &str,
        _format: &str,
        _max_messages: i64,
        _role_name: Option<&str>,
    ) -> Result<ChatExportResponse> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support export_session".into(),
        ))
    }

    async fn export_role(
        &self,
        _role_id: &str,
        _format: &str,
        _max_messages: i64,
        _role_name: Option<&str>,
    ) -> Result<ChatExportResponse> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support export_role".into(),
        ))
    }

    async fn get_storage_stats(&self) -> Result<Vec<RoleStorageStat>> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support get_storage_stats".into(),
        ))
    }

    async fn apply_auto_cleanup(
        &self,
        _role_id: &str,
        _cfg: &AutoCleanupConfig,
    ) -> Result<AutoCleanupResult> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support apply_auto_cleanup".into(),
        ))
    }

    async fn replay_memory_extraction(
        &self,
        _source: &str,
        _target: &ReplayTarget,
        _task_id: &str,
        _progress: &ReplayProgress,
    ) -> Result<ReplayResult> {
        Err(AppError::InvalidParameter(
            "chat storage backend does not support replay_memory_extraction".into(),
        ))
    }

    /// Backend kind label (`hybrid` / `file` / `sqlite`).
    fn backend_kind(&self) -> &'static str;

    /// Backend supports content search.
    async fn supports_search(&self) -> bool {
        false
    }

    /// Backend supports memory replay from chat history.
    async fn supports_replay(&self) -> bool {
        false
    }

    /// Backend supports automatic session cleanup.
    async fn supports_cleanup(&self) -> bool {
        false
    }
}
