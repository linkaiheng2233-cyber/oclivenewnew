//! [`DbManager`](super::DbManager) methods for `long_term_memory`.

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::{DbManager, parse_memory_created_at};
use super::memory_merge::{merge_long_term_memory_line, TxOrPool};
use crate::error::{AppError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};

impl DbManager {
    pub async fn save_memory(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
    ) -> Result<String> {
        self.save_memory_merged(role_id, content, importance, 0.6, "default")
            .await
    }

    pub async fn save_memory_merged(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        similarity_threshold: f64,
        scene_id: &str,
    ) -> Result<String> {
        let trimmed = content.trim();
        if importance <= 0.0 || trimmed.is_empty() {
            return Ok(String::new());
        }
        merge_long_term_memory_line(
            TxOrPool::Pool(&self.pool),
            role_id,
            scene_id,
            trimmed,
            importance,
            similarity_threshold,
        )
        .await?;
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM long_term_memory WHERE role_id = ? AND content = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(role_id)
        .bind(trimmed)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(id,)| id.to_string()).unwrap_or_default())
    }

    pub async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>> {
        let rows = sqlx::query_as::<_, (
            i64,
            String,
            String,
            f64,
            f64,
            String,
            Option<String>,
            i32,
        )>(
            "SELECT id, role_id, content, importance, weight, created_at, scene_id, mention_count
             FROM long_term_memory
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(role_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let memories = rows
            .into_iter()
            .map(
                |(id, role_id, content, importance, weight, created_at, scene_id, mention_count)| {
                    Memory {
                        id: id.to_string(),
                        role_id,
                        content,
                        importance,
                        weight,
                        created_at: parse_memory_created_at(&created_at),
                        scene_id,
                        mention_count: mention_count.max(1),
                    }
                },
            )
            .collect();

        Ok(memories)
    }

    pub async fn increment_memory_mention_count(
        &self,
        memory_id: i64,
        role_id: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE long_term_memory SET mention_count = mention_count + 1 WHERE id = ? AND role_id = ?",
        )
        .bind(memory_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn count_memories(&self, role_id: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM long_term_memory WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.0)
    }

    pub async fn load_memories_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Memory>> {
        let rows = sqlx::query_as::<_, (
            i64,
            String,
            String,
            f64,
            f64,
            String,
            Option<String>,
            i32,
        )>(
            "SELECT id, role_id, content, importance, weight, created_at, scene_id, mention_count
             FROM long_term_memory
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
        )
        .bind(role_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let memories = rows
            .into_iter()
            .map(
                |(id, role_id, content, importance, weight, created_at, scene_id, mention_count)| {
                    Memory {
                        id: id.to_string(),
                        role_id,
                        content,
                        importance,
                        weight,
                        created_at: parse_memory_created_at(&created_at),
                        scene_id,
                        mention_count: mention_count.max(1),
                    }
                },
            )
            .collect();

        Ok(memories)
    }

    pub async fn get_latest_memory_created_at(
        &self,
        role_id: &str,
    ) -> Result<Option<chrono::DateTime<Utc>>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT created_at FROM long_term_memory WHERE role_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.and_then(|(s,)| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }))
    }

    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM long_term_memory WHERE id = ?")
            .bind(memory_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_memory_for_role(&self, role_id: &str, memory_id: &str) -> Result<bool> {
        let r = sqlx::query("DELETE FROM long_term_memory WHERE id = ? AND role_id = ?")
            .bind(memory_id)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }
}
