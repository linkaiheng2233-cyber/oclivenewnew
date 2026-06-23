//! [`DbManager`] methods for `chat_messages`.

use super::db::{message_row_from_row, InsertTurnResult, MessageRow, NewTurnMessages};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use sqlx::Row;

impl DbManager {
    /// Insert one user + one assistant message; enforces per-session FIFO cap.
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn insert_chat_turn_messages(
        &self,
        session_id: &str,
        turn: NewTurnMessages,
        max_messages: i64,
    ) -> Result<InsertTurnResult> {
        let cap = max_messages.max(2);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let turn_index: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(turn_index), -1) + 1 FROM chat_messages WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let overflow = count.saturating_add(2).saturating_sub(cap);
        if overflow > 0 {
            trim_oldest_chat_messages(&mut tx, session_id, overflow).await?;
        }

        sqlx::query(
            "INSERT INTO chat_messages (id, session_id, turn_index, sender, content, metadata, created_at)
             VALUES (?, ?, ?, 'user', ?, ?, ?)",
        )
        .bind(&turn.user_id)
        .bind(session_id)
        .bind(turn_index)
        .bind(&turn.user_content)
        .bind(&turn.user_metadata)
        .bind(&turn.user_created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO chat_messages (id, session_id, turn_index, sender, content, metadata, created_at)
             VALUES (?, ?, ?, 'assistant', ?, ?, ?)",
        )
        .bind(&turn.assistant_id)
        .bind(session_id)
        .bind(turn_index)
        .bind(&turn.assistant_content)
        .bind(&turn.assistant_metadata)
        .bind(&turn.assistant_created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let new_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let updated_at = turn.assistant_created_at.clone();
        sqlx::query(
            "UPDATE chat_sessions SET message_count = ?, updated_at = ? WHERE session_id = ?",
        )
        .bind(new_count)
        .bind(&updated_at)
        .bind(session_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(InsertTurnResult {
            message_count: new_count,
            turn_index,
        })
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn fetch_chat_messages(
        &self,
        session_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<MessageRow>> {
        let rows = sqlx::query(
            "SELECT id, session_id, turn_index, sender, content, metadata, created_at
             FROM chat_messages
             WHERE session_id = ?
             ORDER BY created_at ASC, id ASC
             LIMIT ? OFFSET ?",
        )
        .bind(session_id)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| message_row_from_row(&row))
            .collect())
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn get_chat_message(&self, message_id: &str) -> Result<Option<MessageRow>> {
        let row = sqlx::query(
            "SELECT id, session_id, turn_index, sender, content, metadata, created_at
             FROM chat_messages WHERE id = ?",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|row| message_row_from_row(&row)))
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_message(&self, message_id: &str) -> Result<Option<String>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let row = sqlx::query("SELECT session_id FROM chat_messages WHERE id = ?")
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let Some(row) = row else {
            tx.rollback().await.ok();
            return Ok(None);
        };
        let session_id: String = row.get("session_id");
        sqlx::query("DELETE FROM chat_messages WHERE id = ?")
            .bind(message_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE session_id = ?")
                .bind(&session_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE chat_sessions SET message_count = ?, updated_at = ? WHERE session_id = ?",
        )
        .bind(count)
        .bind(&now)
        .bind(&session_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(Some(session_id))
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn edit_chat_message(
        &self,
        message_id: &str,
        new_content: &str,
    ) -> Result<Option<String>> {
        let trimmed = new_content.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidParameter(
                "message content cannot be empty".into(),
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let row =
            sqlx::query("SELECT session_id, sender, metadata FROM chat_messages WHERE id = ?")
                .bind(message_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let Some(row) = row else {
            tx.rollback().await.ok();
            return Ok(None);
        };
        let session_id: String = row.get("session_id");
        let sender: String = row.get("sender");
        if sender != "user" {
            tx.rollback().await.ok();
            return Err(AppError::InvalidParameter(
                "only user messages can be edited".into(),
            ));
        }
        let metadata: Option<String> = row.get("metadata");
        let mut meta: serde_json::Value = metadata
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "edited_at".into(),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            );
        }
        let meta_str = meta.to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE chat_messages SET content = ?, metadata = ? WHERE id = ?")
            .bind(trimmed)
            .bind(&meta_str)
            .bind(message_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("UPDATE chat_sessions SET updated_at = ? WHERE session_id = ?")
            .bind(&now)
            .bind(&session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(Some(session_id))
    }
}

async fn trim_oldest_chat_messages(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    session_id: &str,
    remove_count: i64,
) -> Result<()> {
    if remove_count <= 0 {
        return Ok(());
    }
    sqlx::query(
        "DELETE FROM chat_messages WHERE id IN (
            SELECT id FROM chat_messages WHERE session_id = ?
            ORDER BY created_at ASC, id ASC LIMIT ?
         )",
    )
    .bind(session_id)
    .bind(remove_count)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::chat_storage::config::DEFAULT_MAX_MESSAGES;
    use crate::infrastructure::chat_storage::db::{MessageRow, NewTurnMessages};
    use crate::infrastructure::db::DbManager;
    use crate::infrastructure::test_db;
    use chrono::Utc;

    async fn mem_db() -> DbManager {
        test_db::mem_db_manager().await
    }

    #[tokio::test]
    async fn insert_and_fetch_turn() {
        let db = mem_db().await;
        db.upsert_chat_session("mumu", "mumu", "default")
            .await
            .expect("session");
        let inserted = db
            .insert_chat_turn_messages(
                "mumu",
                NewTurnMessages {
                    user_id: uuid::Uuid::new_v4().to_string(),
                    assistant_id: uuid::Uuid::new_v4().to_string(),
                    user_content: "hi".into(),
                    assistant_content: "hello".into(),
                    user_metadata: None,
                    assistant_metadata: Some(r#"{"reply_is_fallback":false}"#.into()),
                    user_created_at: Utc::now().to_rfc3339(),
                    assistant_created_at: Utc::now().to_rfc3339(),
                },
                DEFAULT_MAX_MESSAGES,
            )
            .await
            .expect("insert");
        assert_eq!(inserted.message_count, 2);
        assert_eq!(inserted.turn_index, 0);
        let msgs = db.fetch_chat_messages("mumu", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, "user");
        assert_eq!(msgs[1].sender, "assistant");
    }

    fn assert_messages_paired(msgs: &[MessageRow]) {
        use std::collections::HashMap;
        let mut by_turn: HashMap<i32, (bool, bool)> = HashMap::new();
        for m in msgs {
            let entry = by_turn.entry(m.turn_index).or_insert((false, false));
            match m.sender.as_str() {
                "user" => entry.0 = true,
                "assistant" => entry.1 = true,
                other => panic!("unexpected sender: {other}"),
            }
        }
        for (turn, (has_user, has_assistant)) in by_turn {
            assert!(
                has_user && has_assistant,
                "turn {turn} missing user or assistant pair"
            );
        }
    }

    #[tokio::test]
    async fn odd_configured_cap_keeps_user_assistant_pairs() {
        use crate::infrastructure::chat_storage::config::load_max_messages_per_session;

        let cap = load_max_messages_per_session(Some(5));
        assert_eq!(cap, 4);

        let db = mem_db().await;
        db.upsert_chat_session("pair_test", "pair_test", "default")
            .await
            .expect("session");

        for i in 0..3 {
            db.insert_chat_turn_messages(
                "pair_test",
                NewTurnMessages {
                    user_id: uuid::Uuid::new_v4().to_string(),
                    assistant_id: uuid::Uuid::new_v4().to_string(),
                    user_content: format!("user turn {i}"),
                    assistant_content: format!("assistant turn {i}"),
                    user_metadata: None,
                    assistant_metadata: None,
                    user_created_at: Utc::now().to_rfc3339(),
                    assistant_created_at: Utc::now().to_rfc3339(),
                },
                cap,
            )
            .await
            .expect("insert");
        }

        let msgs = db
            .fetch_chat_messages("pair_test", 20, 0)
            .await
            .expect("fetch");
        assert!(!msgs.is_empty());
        assert_messages_paired(&msgs);
    }
}
