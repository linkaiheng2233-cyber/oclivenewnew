use crate::error::{AppError, Result};
use crate::models::InteractionMode;
use crate::models::{Event, EventType, Memory, PersonalityVector};
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::SqlitePool;
use std::time::Instant;

pub struct ChatTurnTxInput<'a> {
    pub role_id: &'a str,
    pub personality: &'a PersonalityVector,
    pub current_emotion: &'a str,
    pub relation_state: &'a str,
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

    // ===== Plugin permissions + audit (subset) =====
    pub async fn is_plugin_permission_granted(
        &self,
        plugin_id: &str,
        permission: &str,
    ) -> Result<bool> {
        let pid = plugin_id.trim();
        let perm = permission.trim();
        if pid.is_empty() || perm.is_empty() {
            return Ok(false);
        }
        let row = sqlx::query(
            "SELECT enabled FROM plugin_permission_grants WHERE plugin_id = ? AND permission = ?",
        )
        .bind(pid)
        .bind(perm)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let Some(r) = row else {
            return Ok(false);
        };
        let en: i64 = r.try_get("enabled").unwrap_or(0);
        Ok(en != 0)
    }

    pub async fn insert_plugin_audit_log(
        &self,
        plugin_id: &str,
        action: &str,
        permission: Option<&str>,
        allowed: bool,
        meta_json: &str,
    ) -> Result<()> {
        let pid = plugin_id.trim();
        let action = action.trim();
        if pid.is_empty() || action.is_empty() {
            return Ok(());
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO plugin_audit_log (plugin_id, action, permission, allowed, meta_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(pid)
        .bind(action)
        .bind(permission.map(str::trim).filter(|s| !s.is_empty()))
        .bind(if allowed { 1i64 } else { 0i64 })
        .bind(meta_json)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
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

    // ===== complex_emotion hint (per-session namespace) =====

    pub async fn get_complex_emotion_hint(&self, role_id: &str) -> Result<Option<String>> {
        let rid = role_id.trim();
        if rid.is_empty() {
            return Ok(None);
        }
        let row = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT complex_emotion_hint FROM role_runtime WHERE role_id = ?",
        )
        .bind(rid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|x| x.0))
    }

    pub async fn set_complex_emotion_hint(&self, role_id: &str, hint: Option<&str>) -> Result<()> {
        let rid = role_id.trim();
        if rid.is_empty() {
            return Err(AppError::InvalidParameter("role_id required".into()));
        }
        self.ensure_role_runtime(rid).await?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET complex_emotion_hint = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(hint.map(|s| s.to_string()))
        .bind(now)
        .bind(rid)
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

    pub async fn get_user_relation(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT user_relation FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(s,)| s))
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

    pub async fn get_event_impact_factor(&self, role_id: &str) -> Result<Option<f64>> {
        let row: Option<(f64,)> =
            sqlx::query_as("SELECT event_impact_factor FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(f,)| f))
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

    pub async fn get_virtual_time_ms(&self, role_id: &str) -> Result<Option<i64>> {
        sqlx::query_scalar::<_, i64>("SELECT virtual_time_ms FROM role_runtime WHERE role_id = ?")
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
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

    /// Atomic write for one chat turn: personality + favorability + memory + event + short-term.
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
            ($code:literal, $future:expr) => {
                if let Err(e) = $future.await {
                    let msg = e.to_string();
                    let _ = tx.rollback().await;
                    return Err(AppError::TransactionError {
                        code: $code,
                        message: msg,
                    });
                }
            };
        }

        txn_step!(
            "TXN_RUNTIME_ENSURE_FAILED",
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
            sqlx::query(
                "INSERT INTO personality_vector (role_id, effective_personality, reason, created_at) VALUES (?, ?, ?, ?)",
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
            sqlx::query(
                "UPDATE role_identity_stats SET favorability = favorability + ?, relation_state = ?, updated_at = ? WHERE role_id = ? AND user_relation_key = ?",
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
            sqlx::query(
                "UPDATE role_runtime SET
                 current_favorability = (SELECT favorability FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?),
                 relation_state = (SELECT relation_state FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?),
                 updated_at = ?
                 WHERE role_id = ?",
            )
            .bind(role_id)
            .bind(urk)
            .bind(role_id)
            .bind(urk)
            .bind(&now)
            .bind(role_id)
            .execute(&mut *tx)
        );

        // Save long-term memory with FIFO trimming.
        txn_step!(
            "TXN_MEMORY_INSERT_FAILED",
            sqlx::query(
                "INSERT INTO long_term_memory (role_id, content, importance, weight, created_at, scene_id) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(memory_content)
            .bind(memory_importance)
            .bind(1.0)
            .bind(&now)
            .bind(scene_id)
            .execute(&mut *tx)
        );
        txn_step!(
            "TXN_MEMORY_FIFO_TRIM_FAILED",
            sqlx::query(
                "DELETE FROM long_term_memory
                 WHERE id IN (
                    SELECT id FROM long_term_memory WHERE role_id = ?
                    ORDER BY datetime(created_at) DESC
                    LIMIT -1 OFFSET ?
                 )",
            )
            .bind(role_id)
            .bind(memory_fifo_limit)
            .execute(&mut *tx)
        );

        // Save short-term turn.
        txn_step!(
            "TXN_SHORT_TERM_INSERT_FAILED",
            sqlx::query(
                "INSERT INTO short_term_memory (role_id, user_input, bot_reply, emotion, scene, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(user_message)
            .bind(bot_reply)
            .bind(current_emotion)
            .bind(scene_id)
            .bind(&now)
            .execute(&mut *tx)
        );

        // Save event.
        txn_step!(
            "TXN_EVENT_INSERT_FAILED",
            sqlx::query(
                "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(role_id)
            .bind(format!("{:?}", event.event_type))
            .bind(&event.user_emotion)
            .bind(&event.bot_emotion)
            .bind(&now)
            .execute(&mut *tx)
        );

        tx.commit().await.map_err(|e| AppError::TransactionError {
            code: "TXN_COMMIT_FAILED",
            message: e.to_string(),
        })?;

        // Return the now-current favorability.
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT favorability FROM role_identity_stats WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(role_id)
        .bind(urk)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let favor_current = row.map(|(f,)| f).unwrap_or(0.0);
        let _ = started; // reserved for future timing logs
        Ok(favor_current)
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
