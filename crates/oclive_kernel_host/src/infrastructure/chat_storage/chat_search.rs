//! [`DbManager`] search / context helpers for chat messages.

use super::db::{message_row_from_row, ChatSearchRow, MessageRow};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use sqlx::Row;

impl DbManager {
    /// LIKE search on `chat_messages.content` (FTS5 upgrade path reserved).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn search_chat_messages(
        &self,
        query: &str,
        role_id: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ChatSearchRow>> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let pattern = format!("%{}%", escape_like(q));
        let cap = limit.clamp(1, 100);
        let rows = if let Some(rid) = role_id.filter(|s| !s.trim().is_empty()) {
            sqlx::query(
                "SELECT m.id, m.session_id, m.turn_index, m.sender, m.content, m.metadata, m.created_at,
                        s.role_id, s.scene_id
                 FROM chat_messages m
                 JOIN chat_sessions s ON s.session_id = m.session_id
                 WHERE m.content LIKE ? ESCAPE '\\' AND s.role_id = ?
                 ORDER BY m.created_at DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(&pattern)
            .bind(rid.trim())
            .bind(i64::from(cap))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT m.id, m.session_id, m.turn_index, m.sender, m.content, m.metadata, m.created_at,
                        s.role_id, s.scene_id
                 FROM chat_messages m
                 JOIN chat_sessions s ON s.session_id = m.session_id
                 WHERE m.content LIKE ? ESCAPE '\\'
                 ORDER BY m.created_at DESC
                 LIMIT ? OFFSET ?",
            )
            .bind(&pattern)
            .bind(i64::from(cap))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| ChatSearchRow {
                id: row.get("id"),
                session_id: row.get("session_id"),
                turn_index: row.get::<i32, _>("turn_index"),
                sender: row.get("sender"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                role_id: row.get("role_id"),
                scene_id: row.get("scene_id"),
            })
            .collect())
    }

    /// Messages before / after a turn (search context).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn fetch_message_context(
        &self,
        session_id: &str,
        turn_index: i32,
        before: u32,
        after: u32,
    ) -> Result<(Vec<MessageRow>, Vec<MessageRow>)> {
        let before_rows = sqlx::query(
            "SELECT id, session_id, turn_index, sender, content, metadata, created_at
             FROM chat_messages
             WHERE session_id = ? AND turn_index < ?
             ORDER BY turn_index DESC
             LIMIT ?",
        )
        .bind(session_id)
        .bind(turn_index)
        .bind(i64::from(before.max(1)))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut before_msgs: Vec<MessageRow> = before_rows
            .into_iter()
            .map(|row| message_row_from_row(&row))
            .collect();
        before_msgs.reverse();

        let after_rows = sqlx::query(
            "SELECT id, session_id, turn_index, sender, content, metadata, created_at
             FROM chat_messages m
             WHERE session_id = ? AND turn_index > ?
             ORDER BY turn_index ASC
             LIMIT ?",
        )
        .bind(session_id)
        .bind(turn_index)
        .bind(i64::from(after.max(1)))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let after_msgs = after_rows
            .into_iter()
            .map(|row| message_row_from_row(&row))
            .collect();

        Ok((before_msgs, after_msgs))
    }

    /// Batch before/after context for search hits (one query per distinct `session_id`).
    ///
    /// # Errors
    ///
    /// Database errors propagate as [`AppError::DatabaseError`].
    pub async fn fetch_search_message_contexts_batch(
        &self,
        hits: &[(String, i32)],
        before: u32,
        after: u32,
    ) -> Result<std::collections::HashMap<(String, i32), (Vec<MessageRow>, Vec<MessageRow>)>> {
        use std::collections::{HashMap, HashSet};

        let mut out = HashMap::new();
        if hits.is_empty() {
            return Ok(out);
        }
        let session_ids: Vec<String> = hits
            .iter()
            .map(|(s, _)| s.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let placeholders = session_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT id, session_id, turn_index, sender, content, metadata, created_at
             FROM chat_messages WHERE session_id IN ({placeholders})
             ORDER BY session_id, turn_index"
        );
        let mut query = sqlx::query(&sql);
        for sid in &session_ids {
            query = query.bind(sid);
        }
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut by_session: HashMap<String, Vec<MessageRow>> = HashMap::new();
        for row in rows {
            let msg = message_row_from_row(&row);
            by_session
                .entry(msg.session_id.clone())
                .or_default()
                .push(msg);
        }
        let before_n = before.max(1) as usize;
        let after_n = after.max(1) as usize;
        for (session_id, turn_index) in hits {
            let Some(msgs) = by_session.get(session_id) else {
                out.insert((session_id.clone(), *turn_index), (Vec::new(), Vec::new()));
                continue;
            };
            let mut before_msgs: Vec<MessageRow> = msgs
                .iter()
                .filter(|m| m.turn_index < *turn_index)
                .cloned()
                .collect();
            let take_from = before_msgs.len().saturating_sub(before_n);
            before_msgs = before_msgs.split_off(take_from);
            let after_msgs: Vec<MessageRow> = msgs
                .iter()
                .filter(|m| m.turn_index > *turn_index)
                .take(after_n)
                .cloned()
                .collect();
            out.insert((session_id.clone(), *turn_index), (before_msgs, after_msgs));
        }
        Ok(out)
    }
}

fn escape_like(raw: &str) -> String {
    raw.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
