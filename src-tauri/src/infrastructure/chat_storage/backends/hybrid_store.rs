//! Hybrid store: SQLite authoritative + async JSON mirror.

use super::super::config::{
    resolve_max_messages_per_session, resolve_role_chat_storage_root, DEFAULT_MAX_MESSAGES,
};
use super::super::cleanup::AutoCleanupConfig;
use super::super::db::{highlight_snippet, MessageRow, NewTurnMessages};
use super::super::export::{export_chat_session, export_role_chats};
use super::super::mirror;
use super::super::replay::{run_memory_replay, ReplayTaskRegistry};
use super::super::shared::{cap_limit, normalize_scene_id, rows_to_stored, timestamp_ms_to_rfc3339};
use super::super::stats::collect_chat_storage_stats;
use super::super::store_trait::ConversationStore;
use super::super::types::{
    AppendTurnResult, AutoCleanupResult, ChatExportResponse, ChatSearchResult,
    ImportChatBucket, ImportChatBucketsResult, ReplayProgress, ReplayResult, ReplayTarget,
    RoleStorageStat, SessionMeta, StoredMessage, TurnPersistInput,
};
use crate::domain::chat_engine::conversation_state_role_id;
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub struct HybridConversationStore {
    db: Arc<DbManager>,
    app_data_dir: PathBuf,
    roles_dir: PathBuf,
    storage_root: PathBuf,
    replay_tasks: Arc<ReplayTaskRegistry>,
}

impl HybridConversationStore {
    #[must_use]
    pub fn new(
        db: Arc<DbManager>,
        app_data_dir: PathBuf,
        roles_dir: PathBuf,
        storage_root: PathBuf,
        replay_tasks: Arc<ReplayTaskRegistry>,
    ) -> Self {
        Self {
            db,
            app_data_dir,
            roles_dir,
            storage_root,
            replay_tasks,
        }
    }

    fn role_storage_root(&self, role_id: &str, location: Option<&str>) -> PathBuf {
        resolve_role_chat_storage_root(
            &self.app_data_dir,
            &self.roles_dir,
            role_id,
            location,
        )
    }

    async fn session_storage_root(&self, session_id: &str) -> Result<PathBuf> {
        let session = self
            .db
            .get_chat_session(session_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::InvalidParameter(format!(
                    "chat session not found: {session_id}"
                ))
            })?;
        Ok(self.role_storage_root(&session.role_id, None))
    }
    ///
    /// # Errors
    ///
    /// Database / validation errors propagate.
    pub async fn import_chat_buckets(
        &self,
        buckets: Vec<ImportChatBucket>,
    ) -> Result<ImportChatBucketsResult> {
        let mut buckets_imported = 0u32;
        let mut turns_imported = 0u32;
        for bucket in buckets {
            if bucket.messages.is_empty() {
                continue;
            }
            let scene_id = normalize_scene_id(&bucket.scene_id);
            let session_id = bucket
                .session_id
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    conversation_state_role_id(bucket.role_id.as_str(), None)
                });
            let max = DEFAULT_MAX_MESSAGES;
            let mut pending_user: Option<(String, i64, Option<String>)> = None;
            for msg in &bucket.messages {
                let role = msg.role.trim().to_lowercase();
                if role == "system" {
                    continue;
                }
                if role == "user" {
                    pending_user = Some((msg.content.clone(), msg.timestamp, msg.id.clone()));
                    continue;
                }
                if role != "assistant" {
                    continue;
                }
                let Some((user_content, user_ts_ms, user_id)) = pending_user.take() else {
                    continue;
                };
                let user_ts = timestamp_ms_to_rfc3339(user_ts_ms);
                let assistant_ts = timestamp_ms_to_rfc3339(msg.timestamp);
                let session = self
                    .db
                    .upsert_chat_session(&session_id, &bucket.role_id, &scene_id)
                    .await?;
                let turn_index = (session.message_count / 2) as i32;
                let turn = NewTurnMessages {
                    user_id: user_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    assistant_id: msg.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                    turn_index,
                    user_content,
                    assistant_content: msg.content.clone(),
                    user_metadata: None,
                    assistant_metadata: None,
                    user_created_at: user_ts,
                    assistant_created_at: assistant_ts,
                };
                self.db
                    .insert_chat_turn_messages(&session_id, turn, max)
                    .await?;
                turns_imported = turns_imported.saturating_add(1);
            }
            buckets_imported = buckets_imported.saturating_add(1);
            let storage_root = self.role_storage_root(&bucket.role_id, None);
            let _ = mirror::rebuild_mirror(
                self.db.as_ref(),
                &storage_root,
                &session_id,
                max,
            )
            .await;
        }
        Ok(ImportChatBucketsResult {
            buckets_imported,
            turns_imported,
        })
    }
}

#[async_trait]
impl ConversationStore for HybridConversationStore {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<AppendTurnResult> {
        let scene_id = normalize_scene_id(&input.scene_id);
        let max = resolve_max_messages_per_session(input.max_messages_per_session);
        let session = self
            .db
            .upsert_chat_session(&input.session_id, &input.role_id, &scene_id)
            .await?;

        let turn_index = (session.message_count / 2) as i32;
        let user_ts = Utc::now().to_rfc3339();
        let assistant_ts = Utc::now().to_rfc3339();

        let user_meta = serde_json::json!({
            "user_emotion": input.user_emotion,
        });
        let assistant_meta = serde_json::json!({
            "model": input.model_name,
            "response_ms": input.response_ms,
            "reply_is_fallback": input.reply_is_fallback,
            "bot_emotion": input.bot_emotion,
        });
        let user_meta_str = user_meta.to_string();
        let assistant_meta_str = assistant_meta.to_string();

        let user_id = Uuid::new_v4().to_string();
        let assistant_id = Uuid::new_v4().to_string();

        let turn = NewTurnMessages {
            user_id: user_id.clone(),
            assistant_id: assistant_id.clone(),
            turn_index,
            user_content: input.user_message.clone(),
            assistant_content: input.assistant_reply.clone(),
            user_metadata: Some(user_meta_str.clone()),
            assistant_metadata: Some(assistant_meta_str.clone()),
            user_created_at: user_ts.clone(),
            assistant_created_at: assistant_ts.clone(),
        };

        self.db
            .insert_chat_turn_messages(&input.session_id, turn, max)
            .await?;

        let user_row = MessageRow {
            id: user_id.clone(),
            session_id: input.session_id.clone(),
            turn_index,
            sender: "user".into(),
            content: input.user_message,
            metadata: Some(user_meta_str),
            created_at: user_ts.clone(),
        };
        let assistant_row = MessageRow {
            id: assistant_id.clone(),
            session_id: input.session_id.clone(),
            turn_index,
            sender: "assistant".into(),
            content: input.assistant_reply,
            metadata: Some(assistant_meta_str),
            created_at: assistant_ts.clone(),
        };
        let new_rows = [user_row, assistant_row];

        let session_after = self
            .db
            .get_chat_session(&input.session_id)
            .await?
            .unwrap_or(session);

        let storage_root = self.role_storage_root(
            &input.role_id,
            Some(&input.chat_storage_location),
        );
        let max_spawn = max;
        let role_id_spawn = input.role_id.clone();
        let cleanup_cfg = input.auto_cleanup_config.clone();
        let db_spawn = Arc::clone(&self.db);
        tokio::spawn(async move {
            if let Err(e) =
                mirror::sync_mirror_append(&storage_root, &session_after, &new_rows, max_spawn).await
            {
                tracing::warn!(
                    target: "oclive_chat_storage",
                    session_id = %session_after.session_id,
                    error = %e,
                    "sync_mirror_append failed"
                );
            }
            if cleanup_cfg.is_enabled() {
                if let Err(e) = super::super::cleanup::apply_auto_cleanup(
                    db_spawn.as_ref(),
                    &storage_root,
                    &role_id_spawn,
                    &cleanup_cfg,
                )
                .await
                {
                    tracing::warn!(
                        target: "oclive_chat_storage",
                        role_id = %role_id_spawn,
                        error = %e,
                        "apply_auto_cleanup failed"
                    );
                }
            }
        });

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
        let cap = cap_limit(limit);
        let rows = self.db.fetch_chat_messages(session_id, cap, offset).await?;
        Ok(rows_to_stored(rows))
    }

    async fn rebuild_mirror(&self, session_id: &str, max_messages: i64) -> Result<String> {
        let root = self.session_storage_root(session_id).await?;
        let path =
            mirror::rebuild_mirror(self.db.as_ref(), &root, session_id, max_messages).await?;
        Ok(path.to_string_lossy().into_owned())
    }

    async fn import_chat_buckets(
        &self,
        buckets: Vec<ImportChatBucket>,
    ) -> Result<ImportChatBucketsResult> {
        HybridConversationStore::import_chat_buckets(self, buckets).await
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
            let to_stored = |row: MessageRow| StoredMessage {
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
        let session_id = self
            .db
            .delete_chat_message(message_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::InvalidParameter(format!("message not found: {message_id}"))
            })?;
        let max = resolve_max_messages_per_session(None);
        let root = self.session_storage_root(&session_id).await?;
        let _ = mirror::rebuild_mirror(self.db.as_ref(), &root, &session_id, max).await?;
        Ok(())
    }

    async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<()> {
        let session_id = self
            .db
            .edit_chat_message(message_id, new_content)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::InvalidParameter(format!("message not found: {message_id}"))
            })?;
        let max = resolve_max_messages_per_session(None);
        let root = self.session_storage_root(&session_id).await?;
        let _ = mirror::rebuild_mirror(self.db.as_ref(), &root, &session_id, max).await?;
        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.db.get_chat_session(session_id).await? {
            let root = self.role_storage_root(&session.role_id, None);
            let _ = mirror::delete_mirror(
                &root,
                &session.role_id,
                &session.scene_id,
                session_id,
            )
            .await;
        }
        self.db.delete_chat_session(session_id).await
    }

    async fn export_session(
        &self,
        session_id: &str,
        format: &str,
        max_messages: i64,
        role_name: Option<&str>,
    ) -> Result<ChatExportResponse> {
        let root = self.session_storage_root(session_id).await?;
        export_chat_session(
            self.db.as_ref(),
            &root,
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
        let root = self.role_storage_root(role_id, None);
        export_role_chats(
            self.db.as_ref(),
            &root,
            role_id,
            format,
            max_messages,
            role_name,
        )
        .await
    }

    async fn get_storage_stats(&self) -> Result<Vec<RoleStorageStat>> {
        collect_chat_storage_stats(
            &self.app_data_dir,
            &self.roles_dir,
            self.db.as_ref(),
        )
        .await
    }

    async fn apply_auto_cleanup(
        &self,
        role_id: &str,
        cfg: &AutoCleanupConfig,
    ) -> Result<AutoCleanupResult> {
        super::super::cleanup::apply_auto_cleanup(
            self.db.as_ref(),
            &self.role_storage_root(role_id, Some(&cfg.chat_storage_location)),
            role_id,
            cfg,
        )
        .await
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
            self.db.clone(),
            self.clone_store(),
            source,
            target,
            task_id,
            self.replay_tasks.clone(),
        )
        .await
    }

    fn backend_kind(&self) -> &'static str {
        "hybrid"
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

impl HybridConversationStore {
    fn clone_store(&self) -> Arc<dyn ConversationStore> {
        Arc::new(HybridConversationStore {
            db: Arc::clone(&self.db),
            app_data_dir: self.app_data_dir.clone(),
            roles_dir: self.roles_dir.clone(),
            storage_root: self.storage_root.clone(),
            replay_tasks: Arc::clone(&self.replay_tasks),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::sqlite_pool;

    async fn store() -> Arc<dyn ConversationStore> {
        let pool = sqlite_pool::connect_memory().await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        let dir = tempfile::tempdir().expect("dir");
        let app_data = dir.path().to_path_buf();
        let roles_dir = app_data.join("roles");
        let _ = std::fs::create_dir_all(&roles_dir);
        let storage_root = app_data.join("chats");
        Arc::new(HybridConversationStore::new(
            Arc::new(DbManager::new(pool)),
            app_data,
            roles_dir,
            storage_root,
            Arc::new(ReplayTaskRegistry::new()),
        )) as Arc<dyn ConversationStore>
    }

    #[tokio::test]
    async fn append_turn_writes_db_even_if_mirror_lags() {
        let store = store().await;
        store
            .append_turn(TurnPersistInput {
                session_id: "mumu".into(),
                role_id: "mumu".into(),
                scene_id: "default".into(),
                user_message: "你好".into(),
                assistant_reply: "嗯".into(),
                reply_is_fallback: false,
                model_name: Some("test".into()),
                response_ms: 10,
                user_emotion: None,
                bot_emotion: None,
                max_messages_per_session: None,
                auto_cleanup_config: Default::default(),
            chat_storage_location: "global".into(),
            })
            .await
            .expect("append");
        let msgs = store.fetch_messages("mumu", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
    }
}
