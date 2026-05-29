//! Hybrid store: SQLite authoritative + async JSON mirror.

use super::config::{resolve_max_messages_per_session, DEFAULT_MAX_MESSAGES};
use super::db::{MessageRow, NewTurnMessages};
use super::mirror;
use super::types::{
    AppendTurnResult, ImportChatBucket, ImportChatBucketsResult, SessionMeta, StoredMessage,
    TurnPersistInput,
};
use crate::domain::chat_engine::conversation_state_role_id;
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Chat history persistence (SQLite + JSON mirror).
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
}

pub struct HybridConversationStore {
    db: Arc<DbManager>,
    app_data_dir: PathBuf,
}

impl HybridConversationStore {
    #[must_use]
    pub fn new(db: Arc<DbManager>, app_data_dir: PathBuf) -> Self {
        Self { db, app_data_dir }
    }

    /// Import IndexedDB-exported buckets (paired user/assistant turns).
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
            let _ = mirror::rebuild_mirror(
                self.db.as_ref(),
                &self.app_data_dir,
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

        let app_data = self.app_data_dir.clone();
        let max_spawn = max;
        let role_id_spawn = input.role_id.clone();
        let cleanup_cfg = input.auto_cleanup_config.clone();
        let db_spawn = Arc::clone(&self.db);
        tokio::spawn(async move {
            if let Err(e) =
                mirror::sync_mirror_append(&app_data, &session_after, &new_rows, max_spawn).await
            {
                tracing::warn!(
                    target: "oclive_chat_storage",
                    session_id = %session_after.session_id,
                    error = %e,
                    "sync_mirror_append failed"
                );
            }
            if cleanup_cfg.is_enabled() {
                if let Err(e) = super::cleanup::apply_auto_cleanup(
                    db_spawn.as_ref(),
                    &app_data,
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

    async fn fetch_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StoredMessage>> {
        let cap = if limit == 0 {
            u32::MAX
        } else {
            limit.min(10_000)
        };
        let rows = self.db.fetch_chat_messages(session_id, cap, offset).await?;
        Ok(rows
            .into_iter()
            .map(|r| StoredMessage {
                id: r.id,
                session_id: r.session_id,
                turn_index: r.turn_index,
                sender: r.sender,
                content: r.content,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn rebuild_mirror(&self, session_id: &str, max_messages: i64) -> Result<String> {
        let path =
            mirror::rebuild_mirror(self.db.as_ref(), &self.app_data_dir, session_id, max_messages)
                .await?;
        Ok(path.to_string_lossy().into_owned())
    }

    async fn import_chat_buckets(
        &self,
        buckets: Vec<ImportChatBucket>,
    ) -> Result<ImportChatBucketsResult> {
        HybridConversationStore::import_chat_buckets(self, buckets).await
    }
}

fn normalize_scene_id(scene_id: &str) -> String {
    let t = scene_id.trim();
    if t.is_empty() {
        "default".to_string()
    } else {
        t.to_string()
    }
}

fn timestamp_ms_to_rfc3339(ms: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp_millis(ms) {
        return dt.to_rfc3339();
    }
    Utc::now().to_rfc3339()
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
        Arc::new(HybridConversationStore::new(
            Arc::new(DbManager::new(pool)),
            dir.path().to_path_buf(),
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
            })
            .await
            .expect("append");
        let msgs = store.fetch_messages("mumu", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
    }
}
