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

/// 数据库操作管理
pub struct DbManager {
    pool: SqlitePool,
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

fn log_txn_finish(tx_name: &str, role_id: &str, elapsed_ms: u128) {
    if elapsed_ms >= TX_ERROR_MS {
        log::error!(
            "tx slow code=TXN_SLOW_CRITICAL tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else if elapsed_ms >= TX_WARN_MS {
        log::warn!(
            "tx slow code=TXN_SLOW_WARN tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    } else {
        log::info!(
            "tx finish tx_name={} role_id={} elapsed_ms={}",
            tx_name,
            role_id,
            elapsed_ms
        );
    }
}

impl DbManager {
    /// 创建新的数据库管理器
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ===== 记忆操作 =====

    /// 保存长期记忆
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

    /// 原子写入：长期记忆 + 事件
    ///
    /// 用于 `send_message` 成功路径，避免出现「记忆已写入但事件未写入」的不一致。
    pub async fn save_memory_and_event_atomic(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        event: &Event,
    ) -> Result<(String, String)> {
        let started = Instant::now();
        log::info!("tx save_memory_and_event_atomic start role_id={}", role_id);
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
        log::info!(
            "tx save_memory_and_event_atomic committed role_id={} memory_id={} event_id={} elapsed_ms={}",
            role_id,
            memory_id,
            event_id,
            elapsed_ms
        );
        log_txn_finish("save_memory_and_event_atomic", role_id, elapsed_ms);

        Ok((memory_id, event_id))
    }

    /// 原子写入一个 chat turn 的关键状态：
    /// personality + favorability + memory + event
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
        log::info!("tx apply_chat_turn_atomic start role_id={}", role_id);
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
                    log::error!(
                        "tx step failed code={} step={} role_id={} err={} elapsed_ms={}",
                        $code,
                        $step_name,
                        role_id,
                        msg,
                        started.elapsed().as_millis()
                    );
                    if let Err(rb_err) = tx.rollback().await {
                        log::error!(
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
                log::debug!(
                    "tx step ok step={} role_id={} step_elapsed_ms={} tx_elapsed_ms={}",
                    $step_name,
                    role_id,
                    _step_started.elapsed().as_millis(),
                    started.elapsed().as_millis()
                );
            };
        }

        txn_step!(
            "TXN_RUNTIME_ENSURE_FAILED",
            "ensure_runtime",
            sqlx::query(
            "INSERT OR IGNORE INTO role_runtime (role_id, current_favorability, current_emotion, relation_state, emotion_updated_at, relation_updated_at, updated_at) VALUES (?, 0.0, ?, ?, ?, ?, ?)",
        )
            .bind(role_id)
            .bind(current_emotion)
            .bind(relation_state)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
        );

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

        let urk = input.user_relation_key;
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

        txn_step!(
            "TXN_IDENTITY_FAVOR_UPDATE_FAILED",
            "update_role_identity_stats",
            sqlx::query(
                "UPDATE role_identity_stats
             SET favorability = favorability + ?,
                 relation_state = ?,
                 updated_at = ?
             WHERE role_id = ? AND user_relation_key = ?",
            )
            .bind(favor_delta)
            .bind(relation_state)
            .bind(&now)
            .bind(role_id)
            .bind(urk)
            .execute(&mut *tx)
        );

        txn_step!(
            "TXN_RUNTIME_MIRROR_FAILED",
            "mirror_favor_from_identity",
            sqlx::query(
                "UPDATE role_runtime SET
                 current_favorability = (
                     SELECT favorability FROM role_identity_stats
                     WHERE role_id = ? AND user_relation_key = ?),
                 relation_state = (
                     SELECT relation_state FROM role_identity_stats
                     WHERE role_id = ? AND user_relation_key = ?),
                 current_emotion = ?,
                 emotion_updated_at = ?,
                 relation_updated_at = ?,
                 updated_at = ?
                 WHERE role_id = ?",
            )
            .bind(role_id)
            .bind(urk)
            .bind(role_id)
            .bind(urk)
            .bind(current_emotion)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .bind(role_id)
            .execute(&mut *tx)
        );

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
            log::info!("tx memory skipped role_id={} reason=low_value", role_id);
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
                "DELETE FROM short_term_memory WHERE role_id = ? AND id IN (
                    SELECT id FROM (
                        SELECT id FROM short_term_memory WHERE role_id = ?
                        ORDER BY datetime(created_at) ASC
                        LIMIT (SELECT MAX(0, (SELECT COUNT(*) FROM short_term_memory WHERE role_id = ?) - ?))
                    )
                )",
            )
            .bind(role_id)
            .bind(role_id)
            .bind(role_id)
            .bind(SHORT_TERM_FIFO_LIMIT)
            .execute(&mut *tx)
        );

        let favor_current: f64 = match sqlx::query_scalar(
            "SELECT current_favorability FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let msg = e.to_string();
                log::error!(
                    "tx step failed code=TXN_FAVORABILITY_READ_FAILED role_id={} err={} elapsed_ms={}",
                    role_id,
                    msg,
                    started.elapsed().as_millis()
                );
                if let Err(rb_err) = tx.rollback().await {
                    log::error!(
                        "tx rollback failed code=TXN_ROLLBACK_FAILED role_id={} err={} elapsed_ms={}",
                        role_id,
                        rb_err,
                        started.elapsed().as_millis()
                    );
                }
                return Err(AppError::TransactionError {
                    code: "TXN_FAVORABILITY_READ_FAILED",
                    message: msg,
                });
            }
        };

        tx.commit().await.map_err(|e| {
            log::error!(
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
        log::info!(
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

    /// 加载角色的长期记忆
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

        let memories = rows
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
            .collect();

        Ok(memories)
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

        let memories = rows
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
            .collect();

        Ok(memories)
    }

    /// 最近一次长期记忆时间（用于 `get_role_info`）
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

    pub async fn role_runtime_exists(&self, role_id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM role_runtime WHERE role_id = ? LIMIT 1")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.is_some())
    }

    /// 删除记忆
    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM long_term_memory WHERE id = ?")
            .bind(memory_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除指定角色的长期记忆（`id` 须属于该 `role_id`）
    pub async fn delete_memory_for_role(&self, role_id: &str, memory_id: &str) -> Result<bool> {
        let r = sqlx::query("DELETE FROM long_term_memory WHERE id = ? AND role_id = ?")
            .bind(memory_id)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    // ===== 性格操作 =====

    /// 保存性格向量
    pub async fn save_personality_vector(
        &self,
        role_id: &str,
        personality: &PersonalityVector,
        reason: &str,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO personality_vector
             (role_id, effective_personality, reason, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(personality.to_json_vec())
        .bind(reason)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取最新的性格向量（历史快照：有效七维 JSON）
    pub async fn get_latest_personality_vector(
        &self,
        role_id: &str,
    ) -> Result<Option<PersonalityVector>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT effective_personality
             FROM personality_vector
             WHERE role_id = ?
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some((json,)) = row {
            let p = PersonalityVector::from_json_vec(&json)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }

    // --- 用户身份：全局 `user_relation`、场景覆盖、`role_identity_stats` 按键存好感/阶段 ---

    pub async fn get_user_relation(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT user_relation FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(s,)| s))
    }

    pub async fn set_user_relation(&self, role_id: &str, relation: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE role_runtime SET user_relation = ?, updated_at = ? WHERE role_id = ?")
            .bind(relation)
            .bind(&now)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_user_relation_for_scene(
        &self,
        role_id: &str,
        scene_id: &str,
    ) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT user_relation FROM role_scene_identity WHERE role_id = ? AND scene_id = ?",
        )
        .bind(role_id)
        .bind(scene_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(s,)| s))
    }

    pub async fn set_user_relation_for_scene(
        &self,
        role_id: &str,
        scene_id: &str,
        relation: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO role_scene_identity (role_id, scene_id, user_relation, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(role_id, scene_id)
             DO UPDATE SET user_relation = excluded.user_relation, updated_at = excluded.updated_at",
        )
        .bind(role_id)
        .bind(scene_id)
        .bind(relation)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn clear_user_relation_for_scene(&self, role_id: &str, scene_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM role_scene_identity WHERE role_id = ? AND scene_id = ?")
            .bind(role_id)
            .bind(scene_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 全局身份模式下调 `set_user_relation` 时清空，避免残留 `role_scene_identity` 在改回 `per_scene` 时误生效。
    pub async fn clear_all_scene_identities_for_role(&self, role_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM role_scene_identity WHERE role_id = ?")
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_use_manifest_default(&self, role_id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(use_manifest_default, 0) FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v != 0).unwrap_or(false))
    }

    pub async fn set_use_manifest_default(&self, role_id: &str, v: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = if v { 1i64 } else { 0i64 };
        sqlx::query(
            "UPDATE role_runtime SET use_manifest_default = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(n)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_remote_life_enabled(&self, role_id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(remote_life_enabled, 0) FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v != 0).unwrap_or(false))
    }

    pub async fn set_remote_life_enabled(&self, role_id: &str, v: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = if v { 1i64 } else { 0i64 };
        sqlx::query(
            "UPDATE role_runtime SET remote_life_enabled = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(n)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_favorability_for_identity(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<Option<f64>> {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT favorability FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(role_id)
        .bind(user_relation_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(f,)| f))
    }

    pub async fn get_relation_state_for_identity(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT relation_state FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(role_id)
        .bind(user_relation_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(s,)| s))
    }

    /// 若不存在则插入（`INSERT OR IGNORE`），用于对话前保证本回合可 UPDATE。
    pub async fn ensure_identity_stats_row(
        &self,
        role_id: &str,
        user_relation_key: &str,
        seed_favorability: f64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR IGNORE INTO role_identity_stats (role_id, user_relation_key, favorability, relation_state, updated_at)
             VALUES (?, ?, ?, 'Stranger', ?)",
        )
        .bind(role_id)
        .bind(user_relation_key)
        .bind(seed_favorability)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn set_identity_favorability_value(
        &self,
        role_id: &str,
        user_relation_key: &str,
        value: f64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_identity_stats SET favorability = ?, updated_at = ? WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(value)
        .bind(&now)
        .bind(role_id)
        .bind(user_relation_key)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        self.mirror_runtime_from_identity(role_id, user_relation_key)
            .await
    }

    /// 将 `role_identity_stats` 中该身份的好感与关系阶段复制到 `role_runtime`（单角色当前选中身份）。
    pub async fn mirror_runtime_from_identity(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<()> {
        let row: Option<(f64, String)> = sqlx::query_as(
            "SELECT favorability, relation_state FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(role_id)
        .bind(user_relation_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        if let Some((f, rs)) = row {
            sqlx::query(
                "UPDATE role_runtime SET current_favorability = ?, relation_state = ?, updated_at = ? WHERE role_id = ?",
            )
            .bind(f)
            .bind(rs)
            .bind(&now)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_event_impact_factor(&self, role_id: &str) -> Result<Option<f64>> {
        let row: Option<(f64,)> =
            sqlx::query_as("SELECT event_impact_factor FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(f,)| f))
    }

    pub async fn set_event_impact_factor(&self, role_id: &str, factor: f64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET event_impact_factor = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(factor)
        .bind(&now)
        .bind(role_id)
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

    pub async fn set_core_delta_personality_json(
        &self,
        role_id: &str,
        core_json: &str,
        delta_json: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET core_personality = ?, delta_personality = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(core_json)
        .bind(delta_json)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 人设优先模式：相处中由模型维护的「可变性格档案」正文（与 manifest 核心性格档案并列）。
    pub async fn get_mutable_personality(&self, role_id: &str) -> Result<String> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT mutable_personality FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(c,)| c).unwrap_or_default())
    }

    pub async fn set_mutable_personality(&self, role_id: &str, text: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET mutable_personality = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(text)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ===== 好感度操作 =====

    /// 确保 `role_runtime` 中存在该角色（性格/记忆外键依赖）
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

    /// 保存好感度（仅更新数值列，避免 `INSERT OR REPLACE` 整行覆盖导致 `user_relation` 等丢失）
    pub async fn save_favorability(&self, role_id: &str, value: f64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET current_favorability = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(value)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            sqlx::query(
                "INSERT INTO role_runtime (role_id, current_favorability, updated_at) VALUES (?, ?, ?)",
            )
            .bind(role_id)
            .bind(value)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }

    /// 获取好感度
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

    /// 按身份读 `role_identity_stats`，缺失时回退到全局 `role_runtime.current_favorability`（与 UI / 对话引擎一致）。
    pub async fn favorability_for_identity_with_runtime_fallback(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<f64> {
        let identity_fav = self
            .get_favorability_for_identity(role_id, user_relation_key)
            .await?;
        Ok(identity_fav
            .or(self.get_favorability(role_id).await?)
            .unwrap_or(0.0))
    }

    pub async fn get_current_emotion(&self, role_id: &str) -> Result<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT current_emotion FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v))
    }

    /// 仅更新立绘/运行时展示用情绪（小写英文，如 `neutral`），用于 `load_role` 启动默认「正常」等。
    pub async fn set_current_emotion(&self, role_id: &str, emotion: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET current_emotion = ?, emotion_updated_at = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(emotion)
        .bind(&now)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::InvalidParameter(
                "role_runtime row missing; call ensure_role_runtime first".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn get_relation_state(&self, role_id: &str) -> Result<Option<String>> {
        let value: Option<String> =
            sqlx::query_scalar("SELECT relation_state FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(value)
    }

    pub async fn get_current_scene(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT current_scene FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(s,)| s))
    }

    pub async fn set_current_scene(&self, role_id: &str, scene_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET current_scene = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(scene_id)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::InvalidParameter(
                "role_runtime row missing; call load_role first".to_string(),
            ));
        }
        Ok(())
    }

    /// 用户叙事/发消息上下文场景（可与 `current_scene` 不同＝异地）
    pub async fn get_user_presence_scene(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT user_presence_scene FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(s,)| s))
    }

    pub async fn set_user_presence_scene(&self, role_id: &str, scene_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET user_presence_scene = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(scene_id)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::InvalidParameter(
                "role_runtime row missing; call load_role first".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn get_virtual_time_ms(&self, role_id: &str) -> Result<Option<i64>> {
        sqlx::query_scalar::<_, i64>("SELECT virtual_time_ms FROM role_runtime WHERE role_id = ?")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn set_virtual_time_ms(&self, role_id: &str, ms: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET virtual_time_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::InvalidParameter(
                "role_runtime row missing; call load_role first".to_string(),
            ));
        }
        Ok(())
    }

    /// 旧版全局 `app_settings.interaction_mode`（迁移用）。
    async fn get_legacy_app_interaction_mode(&self) -> Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_settings WHERE key = 'interaction_mode' LIMIT 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row
            .map(|(v,)| v)
            .filter(|s| s == InteractionMode::IMMERSIVE || s == InteractionMode::PURE_CHAT))
    }

    /// 首次为 `role_runtime.interaction_mode` 赋值：优先旧版全局设置，否则 `settings.json` 建议值，否则沉浸。
    pub async fn ensure_interaction_mode_seeded(
        &self,
        role_id: &str,
        pack_default: Option<&str>,
    ) -> Result<()> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT interaction_mode FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let Some((current,)) = row else {
            return Ok(());
        };
        if current.is_some() {
            return Ok(());
        }
        let legacy = self.get_legacy_app_interaction_mode().await?;
        let mode = if let Some(l) = legacy {
            InteractionMode::normalize(Some(l.as_str()))
        } else {
            InteractionMode::normalize(pack_default)
        };
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET interaction_mode = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(mode.as_str())
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_interaction_mode(&self, role_id: &str) -> Result<InteractionMode> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT interaction_mode FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let raw = row.and_then(|(v,)| v);
        Ok(InteractionMode::normalize(raw.as_deref()))
    }

    pub async fn set_interaction_mode_for_role(&self, role_id: &str, mode: &str) -> Result<()> {
        let normalized = InteractionMode::normalize(Some(mode));
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET interaction_mode = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(normalized.as_str())
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::InvalidParameter(
                "role_runtime row missing; call load_role first".to_string(),
            ));
        }
        Ok(())
    }

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

    /// 导出用：按时间升序返回短期对话
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

    /// 好感度增量（更新 `role_runtime.current_favorability`，并写入 `favorability_history`）
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

    // ===== 事件操作 =====

    /// 保存事件
    pub async fn save_event(&self, role_id: &str, event: &Event) -> Result<String> {
        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(format!("{:?}", event.event_type))
        .bind(&event.user_emotion)
        .bind(&event.bot_emotion)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid().to_string())
    }

    pub async fn count_events(&self, role_id: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.0)
    }

    /// 分页事件列表（含 `id` / `resolution`，供 `query_events`）
    pub async fn list_events_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<EventListRow>> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            "SELECT id, role_id, event_type, user_emotion, bot_emotion, resolution, created_at
             FROM events
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
                |(id, role_id, event_type, user_emotion, bot_emotion, resolution, created_at)| {
                    EventListRow {
                        id,
                        role_id,
                        event_type,
                        user_emotion,
                        bot_emotion,
                        resolution,
                        created_at,
                    }
                },
            )
            .collect())
    }

    /// 手动插入事件（`create_event` API），返回 `(id, created_at)`
    pub async fn insert_manual_event(
        &self,
        role_id: &str,
        event_type: &EventType,
        user_emotion: &str,
        bot_emotion: &str,
        resolution: Option<&str>,
    ) -> Result<(i64, String)> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, resolution, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(format!("{:?}", event_type))
        .bind(user_emotion)
        .bind(bot_emotion)
        .bind(resolution)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok((result.last_insert_rowid(), now))
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

        let events = rows
            .into_iter()
            .map(|(event_type, user_emotion, bot_emotion, _resolution)| {
                // 简化事件类型解析，实际应更完善
                let event_type = match event_type.as_str() {
                    "Quarrel" => EventType::Quarrel,
                    "Apology" => EventType::Apology,
                    "Praise" => EventType::Praise,
                    "Complaint" => EventType::Complaint,
                    "Confession" => EventType::Confession,
                    "Joke" => EventType::Joke,
                    "Ignore" => EventType::Ignore,
                    _ => EventType::Ignore, // 默认值
                };

                Event {
                    event_type,
                    user_emotion,
                    bot_emotion,
                }
            })
            .collect();

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Result<SqlitePool> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // 创建表
        sqlx::query(include_str!("../../migrations/001_init.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/002_add_current_emotion.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/004_add_relation_state.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/005_add_virtual_time.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/006_role_pack_runtime.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/007_role_scene_identity.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/008_role_identity_stats.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/009_remote_life_enabled.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/010_user_presence_scene.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/011_app_settings.sql"))
            .execute(&pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!(
            "../../migrations/012_role_runtime_interaction_mode.sql"
        ))
        .execute(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(include_str!("../../migrations/013_mutable_personality.sql"))
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
