//! Shared long-term memory dedupe / merge (keyword overlap similarity).

use crate::domain::memory_engine::MemoryEngine;
use crate::error::{AppError, Result};
use chrono::Utc;
use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeOutcome {
    New,
    Updated,
    Skipped,
}

/// SQLite executor for merge: active transaction or connection pool.
pub enum TxOrPool<'a> {
    Tx(&'a mut Transaction<'a, Sqlite>),
    Pool(&'a SqlitePool),
}

pub async fn merge_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    role_id: &str,
    scene_id: &str,
    trimmed: &str,
    importance: f64,
    similarity_threshold: f64,
) -> Result<MergeOutcome> {
    let candidates: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, content FROM long_term_memory WHERE role_id = ? ORDER BY created_at DESC LIMIT 40",
    )
    .bind(role_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    for (id, existing) in candidates {
        let sim = MemoryEngine::keyword_overlap_similarity(trimmed, existing.as_str());
        if sim >= similarity_threshold {
            sqlx::query(
                "UPDATE long_term_memory SET mention_count = mention_count + 1 WHERE id = ? AND role_id = ?",
            )
            .bind(id)
            .bind(role_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            return Ok(MergeOutcome::Updated);
        }
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at, scene_id, mention_count)
         VALUES (?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(role_id)
    .bind(trimmed)
    .bind(importance)
    .bind(1.0)
    .bind(&now)
    .bind(scene_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(MergeOutcome::New)
}

async fn merge_in_pool(
    pool: &SqlitePool,
    role_id: &str,
    scene_id: &str,
    trimmed: &str,
    importance: f64,
    similarity_threshold: f64,
) -> Result<MergeOutcome> {
    let candidates: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, content FROM long_term_memory WHERE role_id = ? ORDER BY created_at DESC LIMIT 40",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    for (id, existing) in candidates {
        let sim = MemoryEngine::keyword_overlap_similarity(trimmed, existing.as_str());
        if sim >= similarity_threshold {
            sqlx::query(
                "UPDATE long_term_memory SET mention_count = mention_count + 1 WHERE id = ? AND role_id = ?",
            )
            .bind(id)
            .bind(role_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            return Ok(MergeOutcome::Updated);
        }
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at, scene_id, mention_count)
         VALUES (?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(role_id)
    .bind(trimmed)
    .bind(importance)
    .bind(1.0)
    .bind(&now)
    .bind(scene_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(MergeOutcome::New)
}

/// Merge one memory line into `long_term_memory` (dedupe by keyword overlap).
pub async fn merge_long_term_memory_line(
    conn: TxOrPool<'_>,
    role_id: &str,
    scene_id: &str,
    content: &str,
    importance: f64,
    similarity_threshold: f64,
) -> Result<MergeOutcome> {
    let trimmed = content.trim();
    if importance <= 0.0 || trimmed.is_empty() {
        return Ok(MergeOutcome::Skipped);
    }

    match conn {
        TxOrPool::Tx(tx) => {
            merge_in_tx(tx, role_id, scene_id, trimmed, importance, similarity_threshold).await
        }
        TxOrPool::Pool(pool) => {
            merge_in_pool(pool, role_id, scene_id, trimmed, importance, similarity_threshold).await
        }
    }
}
