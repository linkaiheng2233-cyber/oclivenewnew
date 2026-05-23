#![allow(clippy::missing_errors_doc)]

use crate::error::{AppError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use sqlx::{Row, SqlitePool};
use std::time::Instant;

/// 短期对话 FIFO 上限（与长期记忆 500 条策略对齐）
pub const SHORT_TERM_FIFO_LIMIT: i64 = 500;

const TX_WARN_MS: u128 = 300;
const TX_ERROR_MS: u128 = 800;

pub(crate) fn parse_memory_created_at(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|e| {
            tracing::warn!(
                created_at = raw,
                error = %e,
                "invalid long_term_memory.created_at, using now"
            );
            Utc::now()
        })
}

/// 数据库操作管理
pub struct DbManager {
    pub(crate) pool: SqlitePool,
}

/// `events` 表分页行（API `query_events`）
#[derive(Debug, Clone)]
pub struct EventListRow {
    pub id: i64,
    pub role_id: String,
    pub event_type: String,
    pub user_emotion: Option<String>,
    pub bot_emotion: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
}

pub struct ChatTurnTxInput<'a> {
    pub role_id: &'a str,
    pub personality: &'a PersonalityVector,
    pub current_emotion: &'a str,
    pub relation_state: &'a str,
    /// 本回合好感/关系阶段写入所归属的 manifest 用户身份键（与 `role_identity_stats` 一致）。
    pub user_relation_key: &'a str,
    pub favor_delta: f64,
    pub memory_content: &'a str,
    pub memory_importance: f64,
    pub memory_fifo_limit: i32,
    pub event: &'a Event,
    pub user_message: &'a str,
    pub bot_reply: &'a str,
    pub scene_id: &'a str,
}

pub(crate) fn log_txn_finish(tx_name: &str, role_id: &str, elapsed_ms: u128) {
    if elapsed_ms >= TX_ERROR_MS {
        tracing::error!(
            "tx slow code=TXN_SLOW_CRITICAL tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else if elapsed_ms >= TX_WARN_MS {
        tracing::warn!(
            "tx slow code=TXN_SLOW_WARN tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else {
        tracing::info!(
            "tx finish tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    }
}


impl DbManager {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn health_ping(&self) -> Result<()> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("health_ping: {e}")))?;
        Ok(())
    }

    pub async fn save_memory_and_event_atomic(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        event: &Event,
    ) -> Result<(String, String)> {
        let started = Instant::now();
        tracing::info!("tx save_memory_and_event_atomic start role_id={}", role_id);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_BEGIN_FAILED",
                message: e.to_string(),
            })?;

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(content)
        .bind(importance)
        .bind(1.0)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::TransactionError {
            code: "TXN_MEMORY_INSERT_FAILED",
            message: e.to_string(),
        })?;

        let memory_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_MEMORY_ID_FETCH_FAILED",
                message: e.to_string(),
            })?
            .to_string();

        sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(format!("{:?}", event.event_type))
        .bind(&event.user_emotion)
        .bind(&event.bot_emotion)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::TransactionError {
            code: "TXN_EVENT_INSERT_FAILED",
            message: e.to_string(),
        })?;

        let event_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_EVENT_ID_FETCH_FAILED",
                message: e.to_string(),
            })?
            .to_string();

        tx.commit().await.map_err(|e| AppError::TransactionError {
            code: "TXN_COMMIT_FAILED",
            message: e.to_string(),
        })?;
        let elapsed_ms = started.elapsed().as_millis();
        tracing::info!(
            "tx save_memory_and_event_atomic committed role_id={} memory_id={} event_id={} elapsed_ms={}",
            role_id,
            memory_id,
            event_id,
            elapsed_ms
        );
        log_txn_finish("save_memory_and_event_atomic", role_id, elapsed_ms);

        Ok((memory_id, event_id))
    }

    pub async fn apply_chat_turn_atomic(&self, input: ChatTurnTxInput<'_>) -> Result<f64> {
        let role_id = input.role_id;
        let personality = input.personality;
        let current_emotion = input.current_emotion;
        let relation_state = input.relation_state;
        let favor_delta = input.favor_delta;
        let memory_content = input.memory_content;
        let memory_importance = input.memory_importance;
        let memory_fifo_limit = input.memory_fifo_limit;
        let event = input.event;
        let user_message = input.user_message;
        let bot_reply = input.bot_reply;
        let scene_id = input.scene_id;
        let started = Instant::now();
        tracing::info!("tx apply_chat_turn_atomic start role_id={}", role_id);
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_BEGIN_FAILED",
                message: e.to_string(),
            })?;
        let now = Utc::now().to_rfc3339();

        macro_rules! txn_step {
            ($code:literal, $step_name:literal, $future:expr) => {
                let _step_started = Instant::now();
                if let Err(e) = $future.await {
                    let msg = e.to_string();
                    tracing::error!(
                        "tx step failed code={} step={} role_id={} err={} elapsed_ms={}",
                        $code,
                        $step_name,
                        role_id,
                        msg,
                        started.elapsed().as_millis()
                    );
                    if let Err(rb_err) = tx.rollback().await {
                        tracing::error!(
                            "tx rollback failed code=TXN_ROLLBACK_FAILED role_id={} err={} elapsed_ms={}",
                            role_id,
                            rb_err,
                            started.elapsed().as_millis()
                        );
                    }
                    return Err(AppError::TransactionError {
                        code: $code,
                        message: msg,
                    });
                }
                tracing::debug!(
                    "tx step ok step={} role_id={} step_elapsed_ms={} tx_elapsed_ms={}",
                    $step_name,
                    role_id,
                    _step_started.elapsed().as_millis(),
                    started.elapsed().as_millis()
                );
            };
        }

        let urk = input.user_relation_key;

        txn_step!(
            "TXN_PERSONALITY_INSERT_FAILED",
            "insert_personality_vector",
            sqlx::query(
                "INSERT INTO personality_vector
             (role_id, effective_personality, reason, created_at)
             VALUES (?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(personality.to_json_vec())
            .bind("chat_turn")
            .bind(&now)
            .execute(&mut *tx)
        );

        txn_step!(
            "TXN_IDENTITY_ENSURE_FAILED",
            "ensure_identity_stats_row_tx",
            sqlx::query(
                "INSERT OR IGNORE INTO role_identity_stats (role_id, user_relation_key, favorability, relation_state, updated_at)
                 VALUES (?, ?,
                    COALESCE((SELECT current_favorability FROM role_runtime WHERE role_id = ?), 0),
                    COALESCE((SELECT relation_state FROM role_runtime WHERE role_id = ?), 'Stranger'),
                    ?)",
            )
            .bind(role_id)
            .bind(urk)
            .bind(role_id)
            .bind(role_id)
            .bind(&now)
            .execute(&mut *tx)
        );

        let (favor_after, relation_after): (f64, String) = match sqlx::query_as(
            "UPDATE role_identity_stats
             SET favorability = favorability + ?,
                 relation_state = ?,
                 updated_at = ?
             WHERE role_id = ? AND user_relation_key = ?
             RETURNING favorability, relation_state",
        )
        .bind(favor_delta)
        .bind(relation_state)
        .bind(&now)
        .bind(role_id)
        .bind(urk)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                let msg = e.to_string();
                tracing::error!(
                    "tx step failed code=TXN_IDENTITY_FAVOR_UPDATE_FAILED role_id={} err={} elapsed_ms={}",
                    role_id,
                    msg,
                    started.elapsed().as_millis()
                );
                if let Err(rb_err) = tx.rollback().await {
                    tracing::error!(
                        "tx rollback failed code=TXN_ROLLBACK_FAILED role_id={} err={} elapsed_ms={}",
                        role_id,
                        rb_err,
                        started.elapsed().as_millis()
                    );
                }
                return Err(AppError::TransactionError {
                    code: "TXN_IDENTITY_FAVOR_UPDATE_FAILED",
                    message: msg,
                });
            }
        };

        let favor_current: f64 = match sqlx::query_scalar(
            "INSERT INTO role_runtime (role_id, current_favorability, current_emotion, relation_state, emotion_updated_at, relation_updated_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(role_id) DO UPDATE SET
                 current_favorability = excluded.current_favorability,
                 relation_state = excluded.relation_state,
                 current_emotion = excluded.current_emotion,
                 emotion_updated_at = excluded.emotion_updated_at,
                 relation_updated_at = excluded.relation_updated_at,
                 updated_at = excluded.updated_at
             RETURNING current_favorability",
        )
        .bind(role_id)
        .bind(favor_after)
        .bind(current_emotion)
        .bind(relation_after.as_str())
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let msg = e.to_string();
                tracing::error!(
                    "tx step failed code=TXN_RUNTIME_UPSERT_FAILED role_id={} err={} elapsed_ms={}",
                    role_id,
                    msg,
                    started.elapsed().as_millis()
                );
                if let Err(rb_err) = tx.rollback().await {
                    tracing::error!(
                        "tx rollback failed code=TXN_ROLLBACK_FAILED role_id={} err={} elapsed_ms={}",
                        role_id,
                        rb_err,
                        started.elapsed().as_millis()
                    );
                }
                return Err(AppError::TransactionError {
                    code: "TXN_RUNTIME_UPSERT_FAILED",
                    message: msg,
                });
            }
        };

        txn_step!(
            "TXN_FAVORABILITY_HISTORY_INSERT_FAILED",
            "insert_favorability_history",
            sqlx::query(
            "INSERT INTO favorability_history (role_id, delta, reason, created_at) VALUES (?, ?, ?, ?)",
        )
            .bind(role_id)
            .bind(favor_delta)
            .bind("chat")
            .bind(&now)
            .execute(&mut *tx)
        );

        if memory_importance > 0.0 && !memory_content.trim().is_empty() {
            txn_step!(
                "TXN_MEMORY_INSERT_FAILED",
                "insert_long_term_memory",
                sqlx::query(
                    "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at, scene_id)
                 VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(role_id)
                .bind(memory_content)
                .bind(memory_importance)
                .bind(1.0)
                .bind(&now)
                .bind(scene_id)
                .execute(&mut *tx)
            );
        } else {
            tracing::info!("tx memory skipped role_id={} reason=low_value", role_id);
        }

        // 每个角色长期记忆上限 500，超出后按 created_at FIFO 删除旧记录。
        txn_step!(
            "TXN_MEMORY_FIFO_TRIM_FAILED",
            "trim_memory_fifo",
            sqlx::query(
                "DELETE FROM long_term_memory
                 WHERE id IN (
                    SELECT id FROM long_term_memory
                    WHERE role_id = ?
                    ORDER BY created_at DESC
                    LIMIT -1 OFFSET ?
                 )",
            )
            .bind(role_id)
            .bind(memory_fifo_limit)
            .execute(&mut *tx)
        );

        txn_step!(
            "TXN_EVENT_INSERT_FAILED",
            "insert_event",
            sqlx::query(
                "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(format!("{:?}", event.event_type))
            .bind(&event.user_emotion)
            .bind(&event.bot_emotion)
            .bind(&now)
            .execute(&mut *tx)
        );

        txn_step!(
            "TXN_SHORT_TERM_INSERT_FAILED",
            "insert_short_term_memory",
            sqlx::query(
                "INSERT INTO short_term_memory (role_id, user_input, bot_reply, emotion, scene, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(user_message)
            .bind(bot_reply)
            .bind(current_emotion)
            .bind(scene_id)
            .bind(&now)
            .execute(&mut *tx)
        );

        txn_step!(
            "TXN_SHORT_TERM_TRIM_FAILED",
            "trim_short_term_fifo",
            sqlx::query(
                "DELETE FROM short_term_memory
                 WHERE role_id = ? AND id NOT IN (
                    SELECT id FROM short_term_memory
                    WHERE role_id = ?
                    ORDER BY id DESC
                    LIMIT ?
                 )",
            )
            .bind(role_id)
            .bind(role_id)
            .bind(SHORT_TERM_FIFO_LIMIT)
            .execute(&mut *tx)
        );

        tx.commit().await.map_err(|e| {
            tracing::error!(
                "tx commit failed code=TXN_COMMIT_FAILED role_id={} err={} elapsed_ms={}",
                role_id,
                e,
                started.elapsed().as_millis()
            );
            AppError::TransactionError {
                code: "TXN_COMMIT_FAILED",
                message: e.to_string(),
            }
        })?;
        tracing::info!(
            "tx apply_chat_turn_atomic committed role_id={} favor_current={} elapsed_ms={}",
            role_id,
            favor_current,
            started.elapsed().as_millis()
        );
        log_txn_finish(
            "apply_chat_turn_atomic",
            role_id,
            started.elapsed().as_millis(),
        );
        Ok(favor_current)
    }

    pub async fn role_runtime_exists(&self, role_id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM role_runtime WHERE role_id = ? LIMIT 1")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.is_some())
    }

    pub async fn delete_all_data_for_manifest_role(
        &self,
        manifest_role_id: &str,
    ) -> Result<Vec<String>> {
        let mid = manifest_role_id.trim();
        if mid.is_empty() {
            return Err(AppError::InvalidParameter(
                "manifest role_id empty".to_string(),
            ));
        }
        let pattern = format!("{mid}__sess__*");
        let ids: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT role_id FROM role_runtime WHERE role_id = ? OR role_id GLOB ?",
        )
        .bind(mid)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for id in &ids {
            for sql in [
                "DELETE FROM short_term_memory WHERE role_id = ?",
                "DELETE FROM long_term_memory WHERE role_id = ?",
                "DELETE FROM events WHERE role_id = ?",
                "DELETE FROM favorability_history WHERE role_id = ?",
                "DELETE FROM personality_vector WHERE role_id = ?",
                "DELETE FROM operation_logs WHERE role_id = ?",
                "DELETE FROM role_scene_identity WHERE role_id = ?",
                "DELETE FROM role_identity_stats WHERE role_id = ?",
            ] {
                sqlx::query(sql)
                    .bind(id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }
            sqlx::query("DELETE FROM role_runtime WHERE role_id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(ids)
    }
}

mod long_term_memory;
mod plugin_state;
mod relation_state;
mod role_runtime;
mod role_runtime_repo;
mod session_state;

pub use role_runtime_repo::RoleRuntimeRepo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::sqlite_pool;

    async fn setup_test_db() -> Result<SqlitePool> {
        let pool = sqlite_pool::connect_memory()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // 创建表
        sqlx::query(include_str!("../../../migrations/001_init.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/002_add_current_emotion.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/004_add_relation_state.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/005_add_virtual_time.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/006_role_pack_runtime.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/007_role_scene_identity.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/008_role_identity_stats.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/009_remote_life_enabled.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/010_user_presence_scene.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/011_app_settings.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!(
            "../../../migrations/012_role_runtime_interaction_mode.sql"
        ))
        .execute(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../../migrations/013_mutable_personality.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // 为测试创建角色运行时记录
        sqlx::query("INSERT INTO role_runtime (role_id, current_favorability) VALUES (?, ?)")
            .bind("test_role")
            .bind(0.0)
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_save_and_load_memory() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        let memory_id = db
            .save_memory("test_role", "test content", 0.8)
            .await
            .unwrap();
        assert!(!memory_id.is_empty());

        // 简化查询，只获取基本字段
        let rows =
            sqlx::query("SELECT id, content, importance FROM long_term_memory WHERE role_id = ?")
                .bind("test_role")
                .fetch_all(&db.pool)
                .await
                .unwrap();

        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        let _id: i64 = row.get(0);
        let content: String = row.get(1);
        let importance: f64 = row.get(2); // SQLite REAL is f64

        assert_eq!(content, "test content");
        assert!((importance - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        let memory_id = db
            .save_memory("test_role", "test content", 0.8)
            .await
            .unwrap();
        db.delete_memory(&memory_id).await.unwrap();

        let memories = db.load_memories("test_role", 10).await.unwrap();
        assert_eq!(memories.len(), 0);
    }

    #[tokio::test]
    async fn test_save_and_get_personality_vector() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        let personality = PersonalityVector {
            stubbornness: 0.3,
            clinginess: 0.6,
            sensitivity: 0.7,
            assertiveness: 0.4,
            forgiveness: 0.7,
            talkativeness: 0.6,
            warmth: 0.8,
        };

        db.save_personality_vector("test_role", &personality, "test")
            .await
            .unwrap();

        let loaded = db
            .get_latest_personality_vector("test_role")
            .await
            .unwrap()
            .unwrap();
        assert!((loaded.warmth - 0.8).abs() < 1e-9);
        assert!((loaded.stubbornness - 0.3).abs() < 1e-9);
    }

    #[tokio::test]
    async fn test_save_and_get_favorability() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        db.save_favorability("test_role", 50.0).await.unwrap();
        let favorability = db.get_favorability("test_role").await.unwrap().unwrap();
        assert_eq!(favorability, 50.0);
    }

    #[tokio::test]
    async fn test_save_and_get_events() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        let event = Event {
            event_type: EventType::Praise,
            user_emotion: "happy".to_string(),
            bot_emotion: "joyful".to_string(),
        };

        db.save_event("test_role", &event).await.unwrap();
        let events = db.get_events("test_role", 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Praise);
    }

    #[tokio::test]
    async fn test_save_memory_and_event_atomic() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        let event = Event {
            event_type: EventType::Joke,
            user_emotion: "happy".to_string(),
            bot_emotion: "neutral".to_string(),
        };
        let (memory_id, event_id) = db
            .save_memory_and_event_atomic("test_role", "hello", 0.5, &event)
            .await
            .unwrap();
        assert!(!memory_id.is_empty());
        assert!(!event_id.is_empty());

        let memories = db.load_memories("test_role", 10).await.unwrap();
        let events = db.get_events("test_role", 10).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Joke);
    }

    #[tokio::test]
    async fn test_apply_chat_turn_atomic_updates_all() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);
        let personality = PersonalityVector {
            stubbornness: 0.4,
            clinginess: 0.5,
            sensitivity: 0.6,
            assertiveness: 0.4,
            forgiveness: 0.5,
            talkativeness: 0.5,
            warmth: 0.7,
        };
        let event = Event {
            event_type: EventType::Praise,
            user_emotion: "happy".to_string(),
            bot_emotion: "neutral".to_string(),
        };

        let favor = db
            .apply_chat_turn_atomic(ChatTurnTxInput {
                role_id: "test_role",
                personality: &personality,
                current_emotion: "Happy",
                relation_state: "Friend",
                user_relation_key: "friend",
                favor_delta: 0.2,
                memory_content: "chat line",
                memory_importance: 0.5,
                memory_fifo_limit: 500,
                event: &event,
                user_message: "hi",
                bot_reply: "hello",
                scene_id: "default",
            })
            .await
            .unwrap();
        assert!((favor - 0.2).abs() < 1e-6);

        let latest = db
            .get_latest_personality_vector("test_role")
            .await
            .unwrap()
            .unwrap();
        assert!((latest.warmth - 0.7).abs() < 1e-6);
        assert!((latest.stubbornness - 0.4).abs() < 1e-6);

        let memories = db.load_memories("test_role", 10).await.unwrap();
        let events = db.get_events("test_role", 10).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::Praise);
        let emotion = db.get_current_emotion("test_role").await.unwrap().unwrap();
        assert_eq!(emotion, "Happy");
    }

    #[tokio::test]
    async fn test_memory_fifo_trim_to_500_per_role() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);
        let personality = PersonalityVector {
            stubbornness: 0.5,
            clinginess: 0.5,
            sensitivity: 0.5,
            assertiveness: 0.5,
            forgiveness: 0.5,
            talkativeness: 0.5,
            warmth: 0.5,
        };
        let event = Event {
            event_type: EventType::Joke,
            user_emotion: "happy".to_string(),
            bot_emotion: "happy".to_string(),
        };

        for i in 0..510 {
            db.apply_chat_turn_atomic(ChatTurnTxInput {
                role_id: "test_role",
                personality: &personality,
                current_emotion: "Happy",
                relation_state: "Friend",
                user_relation_key: "friend",
                favor_delta: 0.0,
                memory_content: &format!("m{}", i),
                memory_importance: 0.5,
                memory_fifo_limit: 500,
                event: &event,
                user_message: "u",
                bot_reply: "b",
                scene_id: "default",
            })
            .await
            .unwrap();
        }

        let count = db.count_memories("test_role").await.unwrap();
        assert_eq!(count, 500);
    }

    #[tokio::test]
    async fn test_set_and_get_user_relation_for_scene() {
        let pool = setup_test_db().await.unwrap();
        let db = DbManager::new(pool);

        db.set_user_relation_for_scene("test_role", "school", "classmate")
            .await
            .unwrap();
        let relation = db
            .get_user_relation_for_scene("test_role", "school")
            .await
            .unwrap();
        assert_eq!(relation.as_deref(), Some("classmate"));

        db.set_user_relation_for_scene("test_role", "school", "stranger")
            .await
            .unwrap();
        let relation = db
            .get_user_relation_for_scene("test_role", "school")
            .await
            .unwrap();
        assert_eq!(relation.as_deref(), Some("stranger"));
    }
}
