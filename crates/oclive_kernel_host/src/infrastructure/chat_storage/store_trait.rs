//! [`ConversationStore`] — pluggable chat history backend.

use super::cleanup::AutoCleanupConfig;
use super::types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ImportChatBucket, ImportChatBucketsResult, ReplayProgress, ReplayResult, ReplayTarget,
    RoleStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
use crate::error::Result;
use async_trait::async_trait;

/// Chat history persistence (Hybrid-only: SQLite authoritative + optional JSON mirror).
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
    async fn list_sessions_by_role(&self, role_id: &str) -> Result<Vec<SessionMeta>>;
    async fn fetch_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>>;

    async fn rebuild_mirror(&self, session_id: &str, max_messages: i64) -> Result<String>;

    async fn import_chat_buckets(
        &self,
        buckets: Vec<ImportChatBucket>,
    ) -> Result<ImportChatBucketsResult>;

    async fn search_messages(
        &self,
        query: &str,
        role_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChatSearchResult>>;

    async fn delete_message(&self, message_id: &str) -> Result<()>;

    async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<()>;

    async fn delete_session(&self, session_id: &str) -> Result<()>;

    async fn export_session(
        &self,
        session_id: &str,
        format: &str,
        max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<ChatExportResponse>;

    async fn export_role(
        &self,
        role_id: &str,
        format: &str,
        max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<ChatExportResponse>;

    async fn get_storage_stats(&self) -> Result<Vec<RoleStorageStat>>;

    async fn apply_auto_cleanup(
        &self,
        role_id: &str,
        cfg: &AutoCleanupConfig,
    ) -> Result<AutoCleanupResult>;

    async fn replay_memory_extraction(
        &self,
        source: &str,
        target: &ReplayTarget,
        task_id: &str,
        progress: &ReplayProgress,
    ) -> Result<ReplayResult>;

    /// Backend kind label (`hybrid` when JSON mirror is on, else `sqlite`).
    fn backend_kind(&self) -> &'static str;

    /// Backend supports content search.
    fn supports_search(&self) -> bool;

    /// Backend supports memory replay from chat history.
    fn supports_replay(&self) -> bool;

    /// Backend supports automatic session cleanup.
    fn supports_cleanup(&self) -> bool;
}
