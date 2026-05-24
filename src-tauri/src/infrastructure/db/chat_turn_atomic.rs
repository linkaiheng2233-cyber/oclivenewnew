#![allow(clippy::too_many_arguments)]

use super::{ChatTurnTxInput, DbManager, log_txn_finish};
use crate::error::{AppError, Result};
use crate::models::PersonalityVector;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use std::sync::atomic::Ordering;
use std::time::Instant;

async fn persist_personality(
    tx: &mut Transaction<'_, Sqlite>,
    role_id: &str,
    personality: &PersonalityVector,
    user_relation_key: &str,
    now: &str,
    started: &Instant,
) -> Result<()> {
    crate::txn_step!(
        role_id,
        started,
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
        .bind(now)
        .execute(tx.as_mut())
    );

    crate::txn_step!(
        role_id,
        started,
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
        .bind(user_relation_key)
        .bind(role_id)
        .bind(role_id)
        .bind(now)
        .execute(tx.as_mut())
    );

    Ok(())
}

async fn upsert_relation_favor(
    tx: &mut Transaction<'_, Sqlite>,
    role_id: &str,
    user_relation_key: &str,
    relation_state: &str,
    favor_delta: f64,
    current_emotion: &str,
    now: &str,
    started: &Instant,
) -> Result<f64> {
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
    .bind(now)
    .bind(role_id)
    .bind(user_relation_key)
    .fetch_one(tx.as_mut())
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
    .bind(now)
    .bind(now)
    .bind(now)
    .fetch_one(tx.as_mut())
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
            return Err(AppError::TransactionError {
                code: "TXN_RUNTIME_UPSERT_FAILED",
                message: msg,
            });
        }
    };

    crate::txn_step!(
        role_id,
        started,
        "TXN_FAVORABILITY_HISTORY_INSERT_FAILED",
        "insert_favorability_history",
        sqlx::query(
            "INSERT INTO favorability_history (role_id, delta, reason, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(favor_delta)
        .bind("chat")
        .bind(now)
        .execute(tx.as_mut())
    );

    Ok(favor_current)
}

async fn record_memory_and_event(
    db: &DbManager,
    tx: &mut Transaction<'_, Sqlite>,
    role_id: &str,
    memory_content: &str,
    memory_importance: f64,
    memory_fifo_limit: i32,
    event: &crate::models::Event,
    scene_id: &str,
    now: &str,
    started: &Instant,
) -> Result<()> {
    if memory_importance > 0.0 && !memory_content.trim().is_empty() {
        crate::txn_step!(
            role_id,
            started,
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
            .bind(now)
            .bind(scene_id)
            .execute(tx.as_mut())
        );
    } else {
        tracing::info!(role_id = role_id, reason = "low_value", "tx memory skipped");
    }

    let mut memory_count = db
        .long_term_row_counts
        .get(role_id)
        .map(|n| n.load(Ordering::Relaxed))
        .unwrap_or(-1);
    if memory_count < 0 {
        memory_count = sqlx::query_scalar("SELECT COUNT(*) FROM long_term_memory WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_MEMORY_COUNT_FAILED",
                message: e.to_string(),
            })?;
    }
    if memory_importance > 0.0 && !memory_content.trim().is_empty() {
        memory_count += 1;
    }
    db.set_long_term_count(role_id, memory_count);
    if memory_count > i64::from(memory_fifo_limit) {
        crate::txn_step!(
            role_id,
            started,
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
            .execute(tx.as_mut())
        );
        db.set_long_term_count(role_id, i64::from(memory_fifo_limit));
    }

    crate::txn_step!(
        role_id,
        started,
        "TXN_EVENT_INSERT_FAILED",
        "insert_event",
        sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(event.event_type.as_ref())
        .bind(&event.user_emotion)
        .bind(&event.bot_emotion)
        .bind(now)
        .execute(tx.as_mut())
    );

    Ok(())
}

async fn record_short_term(
    db: &DbManager,
    tx: &mut Transaction<'_, Sqlite>,
    role_id: &str,
    user_message: &str,
    bot_reply: &str,
    current_emotion: &str,
    scene_id: &str,
    memory_fifo_limit: i32,
    now: &str,
    started: &Instant,
) -> Result<()> {
    crate::txn_step!(
        role_id,
        started,
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
        .bind(now)
        .execute(tx.as_mut())
    );

    let mut short_term_count = db
        .short_term_row_counts
        .get(role_id)
        .map(|n| n.load(Ordering::Relaxed))
        .unwrap_or(-1);
    if short_term_count < 0 {
        short_term_count = sqlx::query_scalar("SELECT COUNT(*) FROM short_term_memory WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_SHORT_TERM_COUNT_FAILED",
                message: e.to_string(),
            })?;
    }
    short_term_count += 1;
    db.set_short_term_count(role_id, short_term_count);
    let short_term_limit = i64::from(memory_fifo_limit);
    if short_term_count > short_term_limit {
        crate::txn_step!(
            role_id,
            started,
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
            .bind(short_term_limit)
            .execute(tx.as_mut())
        );
        db.set_short_term_count(role_id, short_term_limit);
    }

    Ok(())
}

async fn commit_chat_turn(
    tx: Transaction<'_, Sqlite>,
    role_id: &str,
    favor_current: f64,
    started: Instant,
) -> Result<f64> {
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
    log_txn_finish(
        "apply_chat_turn_atomic",
        role_id,
        started.elapsed().as_millis(),
    );
    Ok(favor_current)
}

pub async fn apply_chat_turn_atomic(db: &DbManager, input: ChatTurnTxInput<'_>) -> Result<f64> {
    let role_id = input.role_id;
    let started = Instant::now();
    let mut tx = db
        .pool
        .begin()
        .await
        .map_err(|e| AppError::TransactionError {
            code: "TXN_BEGIN_FAILED",
            message: e.to_string(),
        })?;
    let now = Utc::now().to_rfc3339();

    persist_personality(
        &mut tx,
        role_id,
        input.personality,
        input.user_relation_key,
        &now,
        &started,
    )
    .await?;
    let favor_current = upsert_relation_favor(
        &mut tx,
        role_id,
        input.user_relation_key,
        input.relation_state,
        input.favor_delta,
        input.current_emotion,
        &now,
        &started,
    )
    .await?;
    record_memory_and_event(
        db,
        &mut tx,
        role_id,
        input.memory_content,
        input.memory_importance,
        input.memory_fifo_limit,
        input.event,
        input.scene_id,
        &now,
        &started,
    )
    .await?;
    record_short_term(
        db,
        &mut tx,
        role_id,
        input.user_message,
        input.bot_reply,
        input.current_emotion,
        input.scene_id,
        input.memory_fifo_limit,
        &now,
        &started,
    )
    .await?;

    commit_chat_turn(tx, role_id, favor_current, started).await
}
