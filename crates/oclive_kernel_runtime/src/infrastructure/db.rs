use crate::error::{AppError, Result};
use crate::models::{Event, EventType, Memory};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Minimal kernel-side DbManager.
///
/// Migration note: this is a subset of `src-tauri/src/infrastructure/db.rs` focused on
/// what `KernelAppState` and runtime repositories currently need.
pub struct DbManager {
    pool: SqlitePool,
}

impl DbManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ===== Long-term memory =====
    pub async fn save_memory(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
    ) -> Result<String> {
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(content)
        .bind(importance)
        .bind(1.0)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid().to_string())
    }

    pub async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>> {
        let rows = sqlx::query_as::<_, (i64, String, String, f64, f64, String, Option<String>)>(
            "SELECT id, role_id, content, importance, weight, created_at, scene_id
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

        Ok(rows
            .into_iter()
            .map(
                |(id, role_id, content, importance, weight, created_at, scene_id)| Memory {
                    id: id.to_string(),
                    role_id,
                    content,
                    importance,
                    weight,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    scene_id,
                },
            )
            .collect())
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
        let rows = sqlx::query_as::<_, (i64, String, String, f64, f64, String, Option<String>)>(
            "SELECT id, role_id, content, importance, weight, created_at, scene_id
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

        Ok(rows
            .into_iter()
            .map(
                |(id, role_id, content, importance, weight, created_at, scene_id)| Memory {
                    id: id.to_string(),
                    role_id,
                    content,
                    importance,
                    weight,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    scene_id,
                },
            )
            .collect())
    }

    // ===== Role runtime / favorability / personality =====
    pub async fn ensure_role_runtime(&self, role_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR IGNORE INTO role_runtime (role_id, current_favorability, updated_at) VALUES (?, 0.0, ?)",
        )
        .bind(role_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_favorability(&self, role_id: &str) -> Result<Option<f64>> {
        let row = sqlx::query_as::<_, (f64,)>(
            "SELECT current_favorability FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(value,)| value))
    }

    pub async fn apply_favorability_delta(&self, role_id: &str, delta: f64) -> Result<()> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        let res = sqlx::query(
            "UPDATE role_runtime SET current_favorability = current_favorability + ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(delta)
        .bind(&now_str)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if res.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO role_runtime (role_id, current_favorability, updated_at) VALUES (?, ?, ?)",
            )
            .bind(role_id)
            .bind(delta)
            .bind(&now_str)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        sqlx::query(
            "INSERT INTO favorability_history (role_id, delta, reason, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(delta)
        .bind("chat")
        .bind(now_str)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_core_delta_personality_json(
        &self,
        role_id: &str,
    ) -> Result<(Option<String>, Option<String>)> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT core_personality, delta_personality FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.unwrap_or((None, None)))
    }

    pub async fn get_mutable_personality(&self, role_id: &str) -> Result<String> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT mutable_personality FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(c,)| c).unwrap_or_default())
    }

    // ===== Short-term memory / events (needed by chat orchestration) =====

    /// 最近 N 轮对话（旧→新），仅 user/bot 文本，供立绘情绪等上下文
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

    /// 获取角色事件历史
    pub async fn get_events(&self, role_id: &str, limit: i32) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT event_type, user_emotion, bot_emotion, resolution
             FROM events
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(role_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(event_type, user_emotion, bot_emotion, _resolution)| {
                let event_type = match event_type.as_str() {
                    "Quarrel" => EventType::Quarrel,
                    "Apology" => EventType::Apology,
                    "Praise" => EventType::Praise,
                    "Complaint" => EventType::Complaint,
                    "Confession" => EventType::Confession,
                    "Joke" => EventType::Joke,
                    "Ignore" => EventType::Ignore,
                    _ => EventType::Ignore,
                };
                Event {
                    event_type,
                    user_emotion,
                    bot_emotion,
                }
            })
            .collect())
    }
}
