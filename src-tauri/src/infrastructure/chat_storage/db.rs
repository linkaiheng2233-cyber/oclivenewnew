//! SQLite read/write for `chat_sessions` / `chat_messages` (authoritative store).

use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
}

#[derive(Debug, Clone)]
pub struct MessageRow {
    pub id: String,
    pub session_id: String,
    pub turn_index: i32,
    pub sender: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct NewTurnMessages {
    pub user_id: String,
    pub assistant_id: String,
    pub turn_index: i32,
    pub user_content: String,
    pub assistant_content: String,
    pub user_metadata: Option<String>,
    pub assistant_metadata: Option<String>,
    pub user_created_at: String,
    pub assistant_created_at: String,
}

impl DbManager {
    /// Upsert session metadata; preserves `created_at` when the row already exists.
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn upsert_chat_session(
        &self,
        session_id: &str,
        role_id: &str,
        scene_id: &str,
    ) -> Result<SessionRow> {
        let now = Utc::now().to_rfc3339();
        let existing = self.get_chat_session(session_id).await?;
        let created_at = existing
            .as_ref()
            .map(|s| s.created_at.clone())
            .unwrap_or_else(|| now.clone());
        let message_count = existing.as_ref().map(|s| s.message_count).unwrap_or(0);
        sqlx::query(
            "INSERT INTO chat_sessions (session_id, role_id, scene_id, created_at, updated_at, message_count)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(session_id) DO UPDATE SET
               role_id = excluded.role_id,
               scene_id = excluded.scene_id,
               updated_at = excluded.updated_at",
        )
        .bind(session_id)
        .bind(role_id)
        .bind(scene_id)
        .bind(&created_at)
        .bind(&now)
        .bind(message_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        self.get_chat_session(session_id)
            .await?
            .ok_or_else(|| AppError::DatabaseError("chat session missing after upsert".into()))
    }

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
    ) -> Result<i64> {
        let cap = max_messages.max(2);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let count: i64 = sqlx::query_scalar(
            "SELECT message_count FROM chat_sessions WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .unwrap_or(0);

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
        .bind(turn.turn_index)
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
        .bind(turn.turn_index)
        .bind(&turn.assistant_content)
        .bind(&turn.assistant_metadata)
        .bind(&turn.assistant_created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let new_count = count.saturating_add(2).min(cap);
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
        Ok(new_count)
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn get_chat_session(&self, session_id: &str) -> Result<Option<SessionRow>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
            "SELECT session_id, role_id, scene_id, created_at, updated_at, message_count
             FROM chat_sessions WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(
            |(session_id, role_id, scene_id, created_at, updated_at, message_count)| SessionRow {
                session_id,
                role_id,
                scene_id,
                created_at,
                updated_at,
                message_count,
            },
        ))
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn get_chat_message_count(&self, session_id: &str) -> Result<i64> {
        let n: Option<i64> = sqlx::query_scalar(
            "SELECT message_count FROM chat_sessions WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(n.unwrap_or(0))
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_chat_sessions(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SessionRow>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
            "SELECT session_id, role_id, scene_id, created_at, updated_at, message_count
             FROM chat_sessions
             WHERE role_id = ? AND scene_id = ?
             ORDER BY updated_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(role_id)
        .bind(scene_id)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(
                |(session_id, role_id, scene_id, created_at, updated_at, message_count)| {
                    SessionRow {
                        session_id,
                        role_id,
                        scene_id,
                        created_at,
                        updated_at,
                        message_count,
                    }
                },
            )
            .collect())
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
            .map(|row| MessageRow {
                id: row.get("id"),
                session_id: row.get("session_id"),
                turn_index: row.get::<i32, _>("turn_index"),
                sender: row.get("sender"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_session(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM chat_messages WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("DELETE FROM chat_sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Remove chat rows for a manifest role and all `role__sess__*` namespaces.
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    /// Count chat sessions for a manifest role (including `__sess__` namespaces).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn count_chat_sessions_for_manifest_role(&self, manifest_role_id: &str) -> Result<u32> {
        let mid = manifest_role_id.trim();
        let pattern = format!("{mid}__sess__*");
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_sessions
             WHERE role_id = ? OR session_id = ? OR session_id GLOB ?",
        )
        .bind(mid)
        .bind(mid)
        .bind(&pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(n.max(0) as u32)
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_data_for_manifest_role(
        &self,
        manifest_role_id: &str,
    ) -> Result<()> {
        let pattern = format!("{manifest_role_id}__sess__*");
        sqlx::query(
            "DELETE FROM chat_messages WHERE session_id IN (
                SELECT session_id FROM chat_sessions
                WHERE role_id = ? OR session_id = ? OR session_id GLOB ?
             )",
        )
        .bind(manifest_role_id)
        .bind(manifest_role_id)
        .bind(&pattern)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM chat_sessions WHERE role_id = ? OR session_id = ? OR session_id GLOB ?",
        )
        .bind(manifest_role_id)
        .bind(manifest_role_id)
        .bind(&pattern)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Remove chat rows for one role + scene (manifest `role_id` column, not session namespace).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_data_for_role_scene(
        &self,
        role_id: &str,
        scene_id: &str,
    ) -> Result<u32> {
        let scene_id = scene_id.trim();
        let scene = if scene_id.is_empty() {
            "default"
        } else {
            scene_id
        };
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_sessions WHERE role_id = ? AND scene_id = ?",
        )
        .bind(role_id)
        .bind(scene)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM chat_messages WHERE session_id IN (
                SELECT session_id FROM chat_sessions WHERE role_id = ? AND scene_id = ?
             )",
        )
        .bind(role_id)
        .bind(scene)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("DELETE FROM chat_sessions WHERE role_id = ? AND scene_id = ?")
            .bind(role_id)
            .bind(scene)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(count.max(0) as u32)
    }

    /// Aggregate session counts per role + scene (storage management UI).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_chat_session_scene_stats(
        &self,
    ) -> Result<Vec<(String, String, u32, Option<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, i64, String)>(
            "SELECT role_id, scene_id, COUNT(*) AS cnt, MAX(updated_at) AS last_active
             FROM chat_sessions
             GROUP BY role_id, scene_id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|(role_id, scene_id, cnt, last_active)| {
                let last = if last_active.trim().is_empty() {
                    None
                } else {
                    Some(last_active)
                };
                (role_id, scene_id, cnt.max(0) as u32, last)
            })
            .collect())
    }

    /// Last message snippet for session list UI (future).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn last_chat_message_snippet(&self, session_id: &str) -> Result<String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT content FROM chat_messages WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(c,)| truncate_snippet(&c, 96)).unwrap_or_default())
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

fn truncate_snippet(s: &str, max_chars: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max_chars {
        return t.to_string();
    }
    t.chars().take(max_chars).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::sqlite_pool;

    async fn mem_db() -> DbManager {
        let pool = sqlite_pool::connect_memory().await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        DbManager::new(pool)
    }

    #[tokio::test]
    async fn insert_and_fetch_turn() {
        use super::config::DEFAULT_MAX_MESSAGES;
        let db = mem_db().await;
        db.upsert_chat_session("mumu", "mumu", "default")
            .await
            .expect("session");
        let count = db
            .insert_chat_turn_messages(
                "mumu",
                NewTurnMessages {
                    user_id: uuid::Uuid::new_v4().to_string(),
                    assistant_id: uuid::Uuid::new_v4().to_string(),
                    turn_index: 0,
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
        assert_eq!(count, 2);
        let msgs = db.fetch_chat_messages("mumu", 10, 0).await.expect("fetch");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, "user");
        assert_eq!(msgs[1].sender, "assistant");
    }
}
