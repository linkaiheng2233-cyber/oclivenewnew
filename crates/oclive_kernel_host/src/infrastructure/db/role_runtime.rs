//! [`DbManager`](super::DbManager) methods for `role_runtime`.

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::{DbManager, EventListRow};
use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::{AppError, Result};
use crate::models::*;
use chrono::Utc;
use sqlx::Row;
use std::time::Instant;

impl DbManager {
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
            return Err(AppError::RoleRuntimeNotReady);
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

    pub async fn get_role_runtime_snapshot(
        &self,
        role_id: &str,
    ) -> Result<Option<RoleRuntimeSnapshot>> {
        let row = sqlx::query_as::<
            _,
            (
                f64,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<String>,
                Option<f64>,
            ),
        >(
            "SELECT current_favorability, current_emotion, relation_state, current_scene,
                    interaction_mode, COALESCE(remote_life_enabled, 0), mutable_personality,
                    event_impact_factor
             FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(
            |(
                favorability,
                emotion,
                relation_state,
                scene,
                interaction_mode,
                remote_life_enabled,
                mutable_personality,
                event_impact_factor,
            )| RoleRuntimeSnapshot {
                favorability: Some(favorability),
                emotion,
                relation_state,
                scene,
                interaction_mode: Some(InteractionMode::normalize(interaction_mode.as_deref())),
                remote_life_enabled: remote_life_enabled.map(|v| v != 0),
                mutable_personality,
                event_impact_factor,
            },
        ))
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
            return Err(AppError::RoleRuntimeNotReady);
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
            return Err(AppError::RoleRuntimeNotReady);
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

    pub async fn get_virtual_time_anchors(&self, role_id: &str) -> Result<(i64, i64, i64)> {
        let row: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT virtual_time_anchor_real_ms, virtual_time_anchor_virtual_ms, virtual_time_ms
             FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.unwrap_or((0, 0, 0)))
    }

    pub async fn set_virtual_time_anchors(
        &self,
        role_id: &str,
        anchor_real_ms: i64,
        anchor_virtual_ms: i64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET virtual_time_anchor_real_ms = ?, virtual_time_anchor_virtual_ms = ?, virtual_time_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(anchor_real_ms)
        .bind(anchor_virtual_ms)
        .bind(anchor_virtual_ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    pub async fn get_last_interaction_at(
        &self,
        role_id: &str,
    ) -> Result<Option<chrono::DateTime<Utc>>> {
        let raw: Option<String> =
            sqlx::query_scalar("SELECT last_interaction_at FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(raw.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s.trim())
                .ok()
                .map(|d| d.with_timezone(&Utc))
        }))
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
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    pub async fn get_last_personality_evolution_virtual_ms(&self, role_id: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_personality_evolution_virtual_ms FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v).unwrap_or(0))
    }

    pub async fn set_last_personality_evolution_virtual_ms(
        &self,
        role_id: &str,
        virtual_ms: i64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime SET last_personality_evolution_virtual_ms = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(virtual_ms)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        if n == 0 {
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    /// Legacy global `app_settings.interaction_mode` (migration only).
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
            return Err(AppError::RoleRuntimeNotReady);
        }
        Ok(())
    }

    /// Global favorability delta (non-turn paths: settings, admin tools).
    ///
    /// Updates both `role_runtime` and **all** `role_identity_stats` rows for `role_id`.
    /// Rows are created by `ensure_identity_stats_row` during chat turns; if none exist,
    /// the identity-stats UPDATE is a no-op while runtime still receives the delta.
    pub async fn apply_favorability_delta(&self, role_id: &str, delta: f64) -> Result<()> {
        let now_str = Utc::now().to_rfc3339();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "UPDATE role_identity_stats SET favorability = favorability + ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(delta)
        .bind(&now_str)
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let res = sqlx::query(
            "UPDATE role_runtime SET current_favorability = current_favorability + ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(delta)
        .bind(&now_str)
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if res.rows_affected() == 0 {
            sqlx::query(
                "INSERT INTO role_runtime (role_id, current_favorability, updated_at) VALUES (?, ?, ?)",
            )
            .bind(role_id)
            .bind(delta)
            .bind(&now_str)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        sqlx::query(
            "INSERT INTO favorability_history (role_id, delta, reason, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(delta)
        .bind("apply_delta")
        .bind(&now_str)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn save_event(&self, role_id: &str, event: &Event) -> Result<String> {
        let now = Utc::now();

        let result = sqlx::query(
            "INSERT INTO events (role_id, event_type, user_emotion, bot_emotion, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(role_id)
        .bind(event.event_type.as_ref())
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
        .bind(event_type.as_ref())
        .bind(user_emotion)
        .bind(bot_emotion)
        .bind(resolution)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok((result.last_insert_rowid(), now))
    }

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
                // Simplified event-type parsing; production should be more complete
                let event_type = match event_type.as_str() {
                    "Quarrel" => EventType::Quarrel,
                    "Apology" => EventType::Apology,
                    "Praise" => EventType::Praise,
                    "Complaint" => EventType::Complaint,
                    "Confession" => EventType::Confession,
                    "Joke" => EventType::Joke,
                    "Ignore" => EventType::Ignore,
                    _ => EventType::Ignore, // default
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

    pub async fn get_session_ollama_model_override(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT session_ollama_model_override FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(v,)| v).filter(|s| !s.trim().is_empty()))
    }

    pub async fn set_session_ollama_model_override(
        &self,
        role_id: &str,
        model: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET session_ollama_model_override = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(model.trim())
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn clear_session_ollama_model_override(&self, role_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET session_ollama_model_override = NULL, updated_at = ? WHERE role_id = ?",
        )
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
