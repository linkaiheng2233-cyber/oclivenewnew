//! `session state` 相关 [`DbManager`](super::DbManager) 方法。

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::DbManager;
use crate::error::{AppError, Result};

impl DbManager {
    pub async fn list_short_term_recent_turns(
        &self,
        role_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT user_input, bot_reply FROM short_term_memory
             WHERE role_id = ?
             ORDER BY datetime(created_at) DESC
             LIMIT ?",
        )
        .bind(role_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().rev().collect())
    }

    pub async fn list_short_term_turns(
        &self,
        role_id: &str,
    ) -> Result<Vec<(String, String, String, Option<String>, String)>> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
            "SELECT user_input, bot_reply, emotion, scene, created_at
             FROM short_term_memory WHERE role_id = ?
             ORDER BY datetime(created_at) ASC",
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    pub async fn list_conversation_sessions(&self) -> Result<Vec<(String, i64, Option<String>)>> {
        let rows = sqlx::query_as::<_, (String, i64, Option<String>)>(
            "SELECT role_id, COUNT(*), MAX(created_at)
             FROM short_term_memory
             GROUP BY role_id
             ORDER BY MAX(created_at) DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }
}
