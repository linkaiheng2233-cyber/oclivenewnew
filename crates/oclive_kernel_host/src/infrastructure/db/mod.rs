#![allow(clippy::missing_errors_doc)]

mod chat_turn_atomic;
pub mod memory_merge;
#[macro_use]
mod helpers;

pub use memory_merge::{merge_in_tx, merge_long_term_memory_line, MergeOutcome, TxOrPool};

use crate::error::{AppError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::Mutex;
#[allow(unused_imports)]
use sqlx::{Row, SqlitePool};
use std::sync::atomic::AtomicI64;
use std::time::{Duration, Instant};

/// Short-term conversation FIFO cap (aligned with long-term memory 500-entry policy).
pub const SHORT_TERM_FIFO_LIMIT: i64 = 500;

const TX_WARN_MS: u128 = 100;
const TX_ERROR_MS: u128 = 300;

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

struct HealthPingCache {
    ok_until: Option<Instant>,
}

/// Database operation manager.
pub struct DbManager {
    pub(crate) pool: SqlitePool,
    health_ping_cache: Mutex<HealthPingCache>,
    pub(crate) long_term_row_counts: DashMap<String, AtomicI64>,
    pub(crate) short_term_row_counts: DashMap<String, AtomicI64>,
}

/// Paginated row from `events` table (API `query_events`).
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
    /// Manifest user-identity key this turn's favorability/relation state write belongs to (matches `role_identity_stats`).
    pub user_relation_key: &'a str,
    pub favor_delta: f64,
    pub memory_content: &'a str,
    pub memory_importance: f64,
    pub memory_fifo_limit: i32,
    pub memory_similarity_threshold: f64,
    pub event: &'a Event,
    pub user_message: &'a str,
    pub bot_reply: &'a str,
    pub scene_id: &'a str,
}

pub(crate) fn log_txn_finish(tx_name: &str, role_id: &str, elapsed_ms: u128) {
    if elapsed_ms >= TX_ERROR_MS {
        tracing::error!(
            tx_name = tx_name,
            role_id = role_id,
            elapsed_ms = elapsed_ms,
            code = "TXN_SLOW_CRITICAL",
            "tx slow",
        );
    } else if elapsed_ms >= TX_WARN_MS {
        tracing::warn!(
            tx_name = tx_name,
            role_id = role_id,
            elapsed_ms = elapsed_ms,
            code = "TXN_SLOW_WARN",
            "tx slow",
        );
    } else {
        tracing::info!(
            tx_name = tx_name,
            role_id = role_id,
            elapsed_ms = elapsed_ms,
            "tx finish",
        );
    }
}


impl DbManager {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            health_ping_cache: Mutex::new(HealthPingCache { ok_until: None }),
            long_term_row_counts: DashMap::new(),
            short_term_row_counts: DashMap::new(),
        }
    }

    /// Highest successfully applied migration version, if the tracking table exists.
    pub async fn max_applied_migration_version(&self) -> Result<Option<i64>> {
        let v: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("max migration version: {e}")))?;
        Ok(v)
    }

    pub async fn health_ping(&self) -> Result<()> {
        const TTL: Duration = Duration::from_secs(5);
        {
            let guard = self.health_ping_cache.lock();
            if guard
                .ok_until
                .is_some_and(|until| Instant::now() < until)
            {
                return Ok(());
            }
        }
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("health_ping: {e}")))?;
        {
            let mut guard = self.health_ping_cache.lock();
            guard.ok_until = Some(Instant::now() + TTL);
        }
        Ok(())
    }

    pub(crate) fn set_long_term_count(&self, role_id: &str, count: i64) {
        self.long_term_row_counts
            .insert(role_id.to_string(), AtomicI64::new(count));
    }

    pub(crate) fn set_short_term_count(&self, role_id: &str, count: i64) {
        self.short_term_row_counts
            .insert(role_id.to_string(), AtomicI64::new(count));
    }

    pub async fn save_memory_and_event_atomic(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        event: &Event,
    ) -> Result<(String, String)> {
        let started = Instant::now();
        tracing::info!(role_id = role_id, "tx save_memory_and_event_atomic start");
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::TransactionError {
                code: "TXN_BEGIN_FAILED",
                message: e.to_string(),
            })?;

        let now = Utc::now().to_rfc3339();
        let memory_id: i64 = sqlx::query_scalar(
            "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at)
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(role_id)
        .bind(content)
        .bind(importance)
        .bind(1.0)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::TransactionError {
            code: "TXN_MEMORY_INSERT_FAILED",
            message: e.to_string(),
        })?;

        let event_id: i64 = sqlx::query_scalar(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(role_id)
        .bind(event.event_type.as_ref())
        .bind(&event.user_emotion)
        .bind(&event.bot_emotion)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::TransactionError {
            code: "TXN_EVENT_INSERT_FAILED",
            message: e.to_string(),
        })?;
        let memory_id = memory_id.to_string();
        let event_id = event_id.to_string();

        tx.commit().await.map_err(|e| AppError::TransactionError {
            code: "TXN_COMMIT_FAILED",
            message: e.to_string(),
        })?;
        let elapsed_ms = started.elapsed().as_millis();
        tracing::info!(
            role_id = role_id,
            memory_id = %memory_id,
            event_id = %event_id,
            elapsed_ms = elapsed_ms,
            "tx save_memory_and_event_atomic committed",
        );
        log_txn_finish("save_memory_and_event_atomic", role_id, elapsed_ms);

        Ok((memory_id, event_id))
    }

    pub async fn apply_chat_turn_atomic(&self, input: ChatTurnTxInput<'_>) -> Result<f64> {
        chat_turn_atomic::apply_chat_turn_atomic(self, input).await
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

        if !ids.is_empty() {
            let placeholders = std::iter::repeat_n("?", ids.len())
                .collect::<Vec<_>>()
                .join(", ");
            for (table, key_column) in [
                ("short_term_memory", "role_id"),
                ("long_term_memory", "role_id"),
                ("events", "role_id"),
                ("favorability_history", "role_id"),
                ("personality_vector", "role_id"),
                ("operation_logs", "role_id"),
                ("role_scene_identity", "role_id"),
                ("role_identity_stats", "role_id"),
                ("role_feedback", "role_id"),
                ("complex_emotion_hint", "srid"),
                ("role_runtime", "role_id"),
            ] {
                let sql = format!("DELETE FROM {table} WHERE {key_column} IN ({placeholders})");
                let mut q = sqlx::query(&sql);
                for id in &ids {
                    q = q.bind(id);
                }
                q.execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }
            for id in &ids {
                self.long_term_row_counts.remove(id);
                self.short_term_row_counts.remove(id);
            }
        }

        self.delete_chat_data_for_manifest_role_in_tx(mid, &mut tx)
            .await?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(ids)
    }
}

mod complex_emotion_hint;
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
    use crate::infrastructure::test_db;

    async fn setup_test_db() -> Result<SqlitePool> {
        let pool = test_db::connect_memory_migrated().await;

        // Create role_runtime row for tests
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

        // Simplified query: basic fields only
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
    async fn delete_manifest_role_data_clears_hint_and_chat_tables() {
        let pool = test_db::connect_memory_migrated().await;
        let db = DbManager::new(pool);
        let mid = "role_del_test";
        let sess = format!("{mid}__sess__abc");

        for rid in [mid, sess.as_str()] {
            sqlx::query(
                "INSERT INTO role_runtime (role_id, current_favorability) VALUES (?, 0)",
            )
            .bind(rid)
            .execute(&db.pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO complex_emotion_hint (srid, narrative_hint, updated_at) VALUES (?, 'hint', datetime('now'))",
            )
            .bind(rid)
            .execute(&db.pool)
            .await
            .unwrap();
            sqlx::query("INSERT INTO role_feedback (role_id, message) VALUES (?, 'fb')")
                .bind(rid)
                .execute(&db.pool)
                .await
                .unwrap();
        }

        let session_id = sess.clone();
        sqlx::query(
            "INSERT INTO chat_sessions (session_id, role_id, scene_id, created_at, updated_at)
             VALUES (?, ?, 'default', datetime('now'), datetime('now'))",
        )
        .bind(&session_id)
        .bind(mid)
        .execute(&db.pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_messages (id, session_id, turn_index, sender, content, created_at)
             VALUES ('m1', ?, 0, 'user', 'hi', datetime('now'))",
        )
        .bind(&session_id)
        .execute(&db.pool)
        .await
        .unwrap();

        let removed = db.delete_all_data_for_manifest_role(mid).await.unwrap();
        assert!(removed.iter().any(|id| id == mid));
        assert!(removed.iter().any(|id| id == &sess));

        let hint_n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM complex_emotion_hint WHERE srid IN (?, ?)")
                .bind(mid)
                .bind(&sess)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(hint_n, 0);

        let chat_n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE session_id = ?")
                .bind(&session_id)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(chat_n, 0);

        let fb_n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_feedback WHERE role_id = ?")
            .bind(mid)
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(fb_n, 0);
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
                memory_similarity_threshold: 0.6,
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
                memory_similarity_threshold: 0.6,
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
