//! Hybrid store: SQLite authoritative + async JSON mirror.

use super::db::{MessageRow, NewTurnMessages};
use super::mirror;
use super::types::{SessionMeta, StoredMessage, TurnPersistInput};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Chat history persistence (SQLite + JSON mirror).
#[async_trait]
pub trait ConversationStore: Send + Sync {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<()>;
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
    async fn rebuild_mirror(&self, session_id: &str) -> Result<String>;
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
}

#[async_trait]
impl ConversationStore for HybridConversationStore {
    async fn append_turn(&self, input: TurnPersistInput) -> Result<()> {
        let scene_id = normalize_scene_id(&input.scene_id);
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
            .insert_chat_turn_messages(&input.session_id, turn)
            .await?;

        let new_rows = vec![
            MessageRow {
                id: user_id,
                session_id: input.session_id.clone(),
                turn_index,
                sender: "user".into(),
                content: input.user_message,
                metadata: Some(user_meta_str),
                created_at: user_ts,
            },
            MessageRow {
                id: assistant_id,
                session_id: input.session_id.clone(),
                turn_index,
                sender: "assistant".into(),
                content: input.assistant_reply,
                metadata: Some(assistant_meta_str),
                created_at: assistant_ts,
            },
        ];

        let session_after = self
            .db
            .get_chat_session(&input.session_id)
            .await?
            .unwrap_or(session);

        let app_data = self.app_data_dir.clone();
        tokio::spawn(async move {
            if let Err(e) = mirror::sync_mirror_append(&app_data, &session_after, &new_rows).await {
                tracing::warn!(
                    target: "oclive_chat_storage",
                    session_id = %session_after.session_id,
                    error = %e,
                    "sync_mirror_append failed"
                );
            }
        });

        Ok(())
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

    async fn rebuild_mirror(&self, session_id: &str) -> Result<String> {
        let path = mirror::rebuild_mirror(self.db.as_ref(), &self.app_data_dir, session_id).await?;
        Ok(path.to_string_lossy().into_owned())
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
            })
            .await
            .expect("append");
        let msgs = store.fetch_messages("mumu", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
    }
}
