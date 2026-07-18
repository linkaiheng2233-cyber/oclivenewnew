//! [`DbManager`](super::DbManager) methods for `long_term_memory`.

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::memory_merge::{merge_long_term_memory_line, TxOrPool};
use super::{parse_memory_created_at, DbManager};
use crate::error::{AppError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};

type MemoryRowTuple = (
    i64,
    String,
    String,
    f64,
    f64,
    String,
    Option<String>,
    i32,
    Option<String>,
);

impl DbManager {
    /// Merge one portable long-term memory while preserving its bounded runtime metadata.
    pub async fn import_portable_memory(
        &self,
        role_id: &str,
        entry: &oclive_kernel_types::PortableLongTermMemoryEntry,
    ) -> Result<()> {
        let existing: Option<(i64, f64, f64, i32)> = sqlx::query_as(
            "SELECT id, importance, weight, mention_count FROM long_term_memory
             WHERE role_id = ? AND content = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(role_id)
        .bind(entry.content.trim())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some((id, importance, weight, mention_count)) = existing {
            sqlx::query(
                "UPDATE long_term_memory
                 SET importance = ?, weight = ?, mention_count = ?, accessed_at = COALESCE(?, accessed_at)
                 WHERE id = ? AND role_id = ?",
            )
            .bind(importance.max(entry.importance))
            .bind(weight.max(entry.weight))
            .bind(mention_count.max(entry.mention_count))
            .bind(entry.accessed_at.as_deref())
            .bind(id)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            return Ok(());
        }

        let created_at = entry
            .created_at
            .clone()
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        sqlx::query(
            "INSERT INTO long_term_memory
             (role_id, content, importance, weight, created_at, accessed_at, scene_id, mention_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(entry.content.trim())
        .bind(entry.importance)
        .bind(entry.weight)
        .bind(created_at)
        .bind(entry.accessed_at.as_deref())
        .bind(entry.scene_id.as_deref())
        .bind(entry.mention_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

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
        self.load_memories_paged(role_id, limit, 0).await
    }

    fn memory_rows_to_models(rows: Vec<MemoryRowTuple>) -> Vec<Memory> {
        rows.into_iter()
            .map(
                |(
                    id,
                    role_id,
                    content,
                    importance,
                    weight,
                    created_at,
                    scene_id,
                    mention_count,
                    accessed_at,
                )| {
                    Memory {
                        id: id.to_string(),
                        role_id,
                        content,
                        importance,
                        weight,
                        created_at: parse_memory_created_at(&created_at),
                        scene_id,
                        mention_count: mention_count.max(1),
                        accessed_at: accessed_at.and_then(|s| {
                            DateTime::parse_from_rfc3339(&s)
                                .ok()
                                .map(|dt| dt.with_timezone(&Utc))
                        }),
                    }
                },
            )
            .collect()
    }

    pub async fn update_memory_weight_and_accessed(
        &self,
        memory_id: i64,
        role_id: &str,
        weight: f64,
        accessed_at: DateTime<Utc>,
    ) -> Result<()> {
        let accessed = accessed_at.to_rfc3339();
        sqlx::query(
            "UPDATE long_term_memory SET weight = ?, accessed_at = ? WHERE id = ? AND role_id = ?",
        )
        .bind(weight)
        .bind(accessed)
        .bind(memory_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn persist_memory_decay_batch(
        &self,
        role_id: &str,
        memories: &[Memory],
        touch_accessed: &[Memory],
    ) -> Result<()> {
        if memories.is_empty() {
            return Ok(());
        }
        let touch_ids: std::collections::HashSet<&str> =
            touch_accessed.iter().map(|m| m.id.as_str()).collect();
        let now = Utc::now();
        let now_rfc = now.to_rfc3339();

        let mut ids: Vec<i64> = Vec::with_capacity(memories.len());
        let mut weights: Vec<f64> = Vec::with_capacity(memories.len());
        let mut accessed: Vec<String> = Vec::with_capacity(memories.len());
        for m in memories {
            let Ok(id) = m.id.parse::<i64>() else {
                continue;
            };
            let accessed_at = if touch_ids.contains(m.id.as_str()) {
                now_rfc.clone()
            } else {
                m.accessed_at.unwrap_or(now).to_rfc3339()
            };
            ids.push(id);
            weights.push(m.weight);
            accessed.push(accessed_at);
        }
        if ids.is_empty() {
            return Ok(());
        }

        let mut weight_cases = String::from("CASE id ");
        let mut accessed_cases = String::from("CASE id ");
        let mut placeholders = String::new();
        for (i, id) in ids.iter().enumerate() {
            weight_cases.push_str(&format!("WHEN {id} THEN ? "));
            accessed_cases.push_str(&format!("WHEN {id} THEN ? "));
            if i > 0 {
                placeholders.push_str(", ");
            }
            placeholders.push_str(&id.to_string());
        }
        weight_cases.push_str("ELSE weight END");
        accessed_cases.push_str("ELSE accessed_at END");

        let sql = format!(
            "UPDATE long_term_memory SET weight = {weight_cases}, accessed_at = {accessed_cases} \
             WHERE role_id = ? AND id IN ({placeholders})"
        );
        let mut q = sqlx::query(&sql).bind(role_id);
        for w in &weights {
            q = q.bind(w);
        }
        for a in &accessed {
            q = q.bind(a);
        }
        q.execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
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
        let rows = sqlx::query_as::<_, MemoryRowTuple>(
            "SELECT id, role_id, content, importance, weight, created_at, scene_id, mention_count, accessed_at
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

        Ok(Self::memory_rows_to_models(rows))
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
