//! Pure SQLite chat storage (no JSON mirror).

use super::super::cleanup::AutoCleanupConfig;
use super::super::db::{highlight_snippet, NewTurnMessages};
use super::super::export::{export_chat_session, export_role_chats};
use super::super::replay::{run_memory_replay, ReplayTaskRegistry};
use super::super::shared::{cap_limit, normalize_scene_id, rows_to_stored};
use super::super::stats::collect_chat_storage_stats_from_db;
use super::super::store_trait::ConversationStore;
use super::super::types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ReplayProgress, ReplayResult, ReplayTarget,
    RoleStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct SqliteConversationStore {
    db: Arc<DbManager>,
    replay_tasks: Arc<ReplayTaskRegistry>,
}

impl SqliteConversationStore {
    #[must_use]
    pub fn new(db: Arc<DbManager>, replay_tasks: Arc<ReplayTaskRegistry>) -> Self {
        Self { db, replay_tasks }
    }
}

#[async_trait]
impl ConversationStore for SqliteConversationStore {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<AppendTurnResult> {
        let scene_id = normalize_scene_id(&input.scene_id);
        let max = super::super::config::resolve_max_messages_per_session(input.max_messages_per_session);
        let session = self
            .db
            .upsert_chat_session(&input.session_id, &input.role_id, &scene_id)
            .await?;
        let turn_index = (session.message_count / 2) as i32;
        let user_ts = Utc::now().to_rfc3339();
        let assistant_ts = Utc::now().to_rfc3339();
        let user_id = Uuid::new_v4().to_string();
        let assistant_id = Uuid::new_v4().to_string();
        self.db
            .insert_chat_turn_messages(
                &input.session_id,
                NewTurnMessages {
                    user_id: user_id.clone(),
                    assistant_id: assistant_id.clone(),
                    turn_index,
                    user_content: input.user_message,
                    assistant_content: input.assistant_reply,
                    user_metadata: None,
                    assistant_metadata: None,
                    user_created_at: user_ts.clone(),
                    assistant_created_at: assistant_ts.clone(),
                },
                max,
            )
            .await?;
        let cfg = input.auto_cleanup_config.clone();
        let role_id = input.role_id.clone();
        let db = Arc::clone(&self.db);
        if cfg.is_enabled() {
            tokio::spawn(async move {
                let _ = super::super::cleanup::apply_auto_cleanup_sqlite(db.as_ref(), &role_id, &cfg).await;
            });
        }
        Ok(AppendTurnResult {
            user_message_id: user_id,
            assistant_message_id: assistant_id,
            user_message_timestamp: user_ts,
            assistant_message_timestamp: assistant_ts,
        })
    }

    async fn list_sessions(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionMeta>> {
        let scene_id = normalize_scene_id(scene_id);
        let rows = self.db.list_chat_sessions(role_id, &scene_id, limit, offset).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let snippet = self.db.last_chat_message_snippet(&row.session_id).await?;
            out.push(SessionMeta {
                session_id: row.session_id,
                role_id: row.role_id,
                scene_id: row.scene_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                message_count: row.message_count,
                last_message_snippet: snippet,
            });
        }
        Ok(out)
    }

    async fn list_sessions_by_role(&self, role_id: &str) -> Result<Vec<SessionMeta>> {
        let rows = self
            .db
            .list_chat_sessions_for_manifest_role(role_id)
            .await?;
        let mut out = Vec::with_capacity(rows.len().min(500));
        for row in rows.into_iter().take(500) {
            let snippet = self.db.last_chat_message_snippet(&row.session_id).await?;
            out.push(SessionMeta {
                session_id: row.session_id,
                role_id: row.role_id,
                scene_id: row.scene_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                message_count: row.message_count,
                last_message_snippet: snippet,
            });
        }
        Ok(out)
    }

    async fn fetch_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>> {
        let rows = self
            .db
            .fetch_chat_messages(session_id, cap_limit(limit), offset)
            .await?;
        Ok(rows_to_stored(rows))
    }

    async fn search_messages(
        &self,
        query: &str,
        role_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChatSearchResult>> {
        let rows = self
            .db
            .search_chat_messages(query, role_id, limit, offset)
            .await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let (before_rows, after_rows) = self
                .db
                .fetch_message_context(&r.session_id, r.turn_index, 2, 2)
                .await?;
            let to_stored = |row: super::super::db::MessageRow| StoredMessage {
                id: row.id,
                session_id: row.session_id,
                turn_index: row.turn_index,
                sender: row.sender,
                content: row.content,
                metadata: row.metadata,
                created_at: row.created_at,
            };
            out.push(ChatSearchResult {
                session_id: r.session_id.clone(),
                role_id: r.role_id.clone(),
                scene_id: r.scene_id.clone(),
                highlight_snippet: highlight_snippet(&r.content, query, 40),
                message: StoredMessage {
                    id: r.id,
                    session_id: r.session_id,
                    turn_index: r.turn_index,
                    sender: r.sender,
                    content: r.content,
                    metadata: r.metadata,
                    created_at: r.created_at,
                },
                context_before: before_rows.into_iter().map(to_stored).collect(),
                context_after: after_rows.into_iter().map(to_stored).collect(),
            });
        }
        Ok(out)
    }

    async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.db
            .delete_chat_message(message_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::InvalidParameter(format!("message not found: {message_id}"))
            })?;
        Ok(())
    }

    async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<()> {
        self.db
            .edit_chat_message(message_id, new_content)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::InvalidParameter(format!("message not found: {message_id}"))
            })?;
        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        self.db.delete_chat_session(session_id).await
    }

    async fn export_session(
        &self,
        session_id: &str,
        format: &str,
        max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<ChatExportResponse> {
        export_chat_session(
            self.db.as_ref(),
            std::path::Path::new("."),
            session_id,
            format,
            max_messages,
            role_name,
        )
        .await
    }

    async fn export_role(
        &self,
        role_id: &str,
        format: &str,
        max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<ChatExportResponse> {
        export_role_chats(
            self.db.as_ref(),
            std::path::Path::new("."),
            role_id,
            format,
            max_messages,
            role_name,
        )
        .await
    }

    async fn get_storage_stats(&self) -> Result<Vec<RoleStorageStat>> {
        collect_chat_storage_stats_from_db(self.db.as_ref()).await
    }

    async fn apply_auto_cleanup(
        &self,
        role_id: &str,
        cfg: &AutoCleanupConfig,
    ) -> Result<AutoCleanupResult> {
        super::super::cleanup::apply_auto_cleanup_sqlite(self.db.as_ref(), role_id, cfg).await
    }

    async fn replay_memory_extraction(
        &self,
        source: &str,
        target: &ReplayTarget,
        task_id: &str,
        progress: &ReplayProgress,
    ) -> Result<ReplayResult> {
        let _ = progress;
        run_memory_replay(
            Arc::clone(&self.db),
            Arc::new(SqliteConversationStore {
                db: Arc::clone(&self.db),
                replay_tasks: Arc::clone(&self.replay_tasks),
            }),
            source,
            target,
            task_id,
            Arc::clone(&self.replay_tasks),
        )
        .await
    }

    fn backend_kind(&self) -> &'static str {
        "sqlite"
    }

    async fn supports_search(&self) -> bool {
        true
    }

    async fn supports_replay(&self) -> bool {
        true
    }

    async fn supports_cleanup(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::sqlite_pool;

    async fn store() -> SqliteConversationStore {
        let pool = sqlite_pool::connect_memory().await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        SqliteConversationStore::new(
            Arc::new(DbManager::new(pool)),
            Arc::new(ReplayTaskRegistry::new()),
        )
    }

    #[tokio::test]
    async fn append_list_fetch() {
        let store = store().await;
        store
            .append_turn(TurnPersistInput {
                session_id: "sq1".into(),
                role_id: "r".into(),
                scene_id: "default".into(),
                user_message: "u".into(),
                assistant_reply: "a".into(),
                reply_is_fallback: false,
                model_name: None,
                response_ms: 0,
                user_emotion: None,
                bot_emotion: None,
                max_messages_per_session: Some(10),
                auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
            })
            .await
            .expect("append");
        let sessions = store.list_sessions("r", "default", 5, 0).await.expect("list");
        assert_eq!(sessions.len(), 1);
        let msgs = store.fetch_messages("sq1", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    async fn fifo_truncation() {
        let store = store().await;
        for i in 0..4 {
            store
                .append_turn(TurnPersistInput {
                    session_id: "cap".into(),
                    role_id: "r".into(),
                    scene_id: "default".into(),
                    user_message: format!("u{i}"),
                    assistant_reply: format!("a{i}"),
                    reply_is_fallback: false,
                    model_name: None,
                    response_ms: 0,
                    user_emotion: None,
                    bot_emotion: None,
                    max_messages_per_session: Some(4),
                    auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
                })
                .await
                .expect("append");
        }
        let msgs = store.fetch_messages("cap", 20, 0).await.expect("fetch");
        assert!(msgs.len() <= 4);
    }

    #[tokio::test]
    async fn search_finds_content() {
        let store = store().await;
        store
            .append_turn(TurnPersistInput {
                session_id: "find".into(),
                role_id: "r".into(),
                scene_id: "default".into(),
                user_message: "unique-keyword-xyz".into(),
                assistant_reply: "ok".into(),
                reply_is_fallback: false,
                model_name: None,
                response_ms: 0,
                user_emotion: None,
                bot_emotion: None,
                max_messages_per_session: None,
                auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
            })
            .await
            .expect("append");
        let hits = store
            .search_messages("unique-keyword-xyz", Some("r"), 10, 0)
            .await
            .expect("search");
        assert!(!hits.is_empty());
    }
}
