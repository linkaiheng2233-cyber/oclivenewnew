//! [`DbManager`] methods for `chat_sessions`.

/// Cap for manifest-role session list queries (SQL `LIMIT` + hybrid store safety `take`).
pub(crate) const MANIFEST_SESSION_LIST_CAP: i64 = 500;

use super::db::{
    manifest_sess_glob_pattern, session_row_from_tuple, truncate_snippet, SessionRow,
};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

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
        Ok(row.map(session_row_from_tuple))
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn get_chat_message_count(&self, session_id: &str) -> Result<i64> {
        let n: Option<i64> =
            sqlx::query_scalar("SELECT message_count FROM chat_sessions WHERE session_id = ?")
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
        Ok(self
            .list_chat_sessions_with_snippets(role_id, scene_id, limit, offset)
            .await?
            .into_iter()
            .map(|(row, _)| row)
            .collect())
    }

    /// List sessions with last-message snippet in one query (avoids N+1).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_chat_sessions_with_snippets(
        &self,
        role_id: &str,
        scene_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<(SessionRow, String)>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>)>(
            "SELECT s.session_id, s.role_id, s.scene_id, s.created_at, s.updated_at, s.message_count,
                    (SELECT m.content FROM chat_messages m
                     WHERE m.session_id = s.session_id
                     ORDER BY m.created_at DESC, m.id DESC LIMIT 1) AS snippet
             FROM chat_sessions s
             WHERE s.role_id = ? AND s.scene_id = ?
             ORDER BY s.updated_at DESC
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
            .map(|(a, b, c, d, e, f, snippet)| {
                (
                    session_row_from_tuple((a, b, c, d, e, f)),
                    snippet
                        .map(|s| truncate_snippet(&s, 96))
                        .unwrap_or_default(),
                )
            })
            .collect())
    }

    /// Manifest role sessions with snippets (single query per page).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_chat_sessions_for_manifest_role_with_snippets(
        &self,
        manifest_role_id: &str,
    ) -> Result<Vec<(SessionRow, String)>> {
        let mid = manifest_role_id.trim();
        let pattern = manifest_sess_glob_pattern(mid);
        let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>)>(
            "SELECT s.session_id, s.role_id, s.scene_id, s.created_at, s.updated_at, s.message_count,
                    (SELECT m.content FROM chat_messages m
                     WHERE m.session_id = s.session_id
                     ORDER BY m.created_at DESC, m.id DESC LIMIT 1) AS snippet
             FROM chat_sessions s
             WHERE s.role_id = ? OR s.session_id = ? OR s.session_id GLOB ?
             ORDER BY s.updated_at DESC
             LIMIT ?",
        )
        .bind(mid)
        .bind(mid)
        .bind(&pattern)
        .bind(MANIFEST_SESSION_LIST_CAP)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|(a, b, c, d, e, f, snippet)| {
                (
                    session_row_from_tuple((a, b, c, d, e, f)),
                    snippet
                        .map(|s| truncate_snippet(&s, 96))
                        .unwrap_or_default(),
                )
            })
            .collect())
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_session(&self, session_id: &str) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("DELETE FROM chat_messages WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("DELETE FROM chat_sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Count chat sessions for a manifest role (including `__sess__` namespaces).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn count_chat_sessions_for_manifest_role(
        &self,
        manifest_role_id: &str,
    ) -> Result<u32> {
        let mid = manifest_role_id.trim();
        let pattern = manifest_sess_glob_pattern(mid);
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
    pub async fn delete_chat_data_for_manifest_role_in_tx(
        &self,
        manifest_role_id: &str,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<()> {
        let pattern = manifest_sess_glob_pattern(manifest_role_id);
        sqlx::query(
            "DELETE FROM chat_messages WHERE session_id IN (
                SELECT session_id FROM chat_sessions
                WHERE role_id = ? OR session_id = ? OR session_id GLOB ?
             )",
        )
        .bind(manifest_role_id)
        .bind(manifest_role_id)
        .bind(&pattern)
        .execute(tx.as_mut())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM chat_sessions WHERE role_id = ? OR session_id = ? OR session_id GLOB ?",
        )
        .bind(manifest_role_id)
        .bind(manifest_role_id)
        .bind(&pattern)
        .execute(tx.as_mut())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn delete_chat_data_for_manifest_role(&self, manifest_role_id: &str) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        self.delete_chat_data_for_manifest_role_in_tx(manifest_role_id, &mut tx)
            .await?;
        tx.commit()
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM chat_messages WHERE session_id IN (
                SELECT session_id FROM chat_sessions WHERE role_id = ? AND scene_id = ?
             )",
        )
        .bind(role_id)
        .bind(scene)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query("DELETE FROM chat_sessions WHERE role_id = ? AND scene_id = ?")
            .bind(role_id)
            .bind(scene)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
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

    /// Last message snippet for session list UI.
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

    /// All sessions for a manifest role (includes `__sess__` namespaces).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_chat_sessions_for_manifest_role(
        &self,
        manifest_role_id: &str,
    ) -> Result<Vec<SessionRow>> {
        let mid = manifest_role_id.trim();
        let pattern = manifest_sess_glob_pattern(mid);
        let rows = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
            "SELECT session_id, role_id, scene_id, created_at, updated_at, message_count
             FROM chat_sessions
             WHERE role_id = ? OR session_id = ? OR session_id GLOB ?
             ORDER BY updated_at DESC",
        )
        .bind(mid)
        .bind(mid)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().map(session_row_from_tuple).collect())
    }

    /// Distinct manifest `role_id` values with at least one chat session.
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn list_distinct_chat_role_ids(&self) -> Result<Vec<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT role_id FROM chat_sessions ORDER BY role_id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
