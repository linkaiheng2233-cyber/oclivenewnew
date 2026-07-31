//! [`DbManager`](super::DbManager) methods for `role_runtime`.

#![allow(clippy::missing_errors_doc, unused_imports)]

mod interaction_mode;
mod virtual_time;

use super::DbManager;
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

    /// Profile-mode archive + seven-dim core/delta in one SQLite transaction (LLM stays outside).
    pub async fn apply_profile_evolution_atomic(
        &self,
        role_id: &str,
        mutable_text: &str,
        core_json: &str,
        delta_json: &str,
    ) -> Result<()> {
        let started = Instant::now();
        let now = Utc::now().to_rfc3339();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        crate::txn_step!(
            role_id,
            started,
            "TXN_MUTABLE_PERSONALITY_FAILED",
            "set_mutable_personality",
            sqlx::query(
                "UPDATE role_runtime SET mutable_personality = ?, updated_at = ? WHERE role_id = ?",
            )
            .bind(mutable_text)
            .bind(&now)
            .bind(role_id)
            .execute(tx.as_mut())
        );

        crate::txn_step!(
            role_id,
            started,
            "TXN_CORE_DELTA_PERSONALITY_FAILED",
            "set_core_delta_personality_json",
            sqlx::query(
                "UPDATE role_runtime SET core_personality = ?, delta_personality = ?, updated_at = ? WHERE role_id = ?",
            )
            .bind(core_json)
            .bind(delta_json)
            .bind(&now)
            .bind(role_id)
            .execute(tx.as_mut())
        );

        tx.commit()
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

    pub async fn get_ephemeral_personality(&self, role_id: &str) -> Result<String> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT ephemeral_personality FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(c,)| c).unwrap_or_default())
    }

    pub async fn set_ephemeral_personality(&self, role_id: &str, text: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET ephemeral_personality = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(text)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_ephemeral_ttl_turns(&self, role_id: &str) -> Result<u32> {
        let row: Option<(Option<i64>,)> =
            sqlx::query_as("SELECT ephemeral_ttl_turns FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(v,)| v).unwrap_or(0).max(0) as u32)
    }

    pub async fn set_ephemeral_ttl_turns(&self, role_id: &str, ttl: u32) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET ephemeral_ttl_turns = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(i64::from(ttl))
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_deep_latch_active(&self, role_id: &str) -> Result<bool> {
        let row: Option<(Option<i64>,)> =
            sqlx::query_as("SELECT deep_latch_active FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(v,)| v).unwrap_or(0) != 0)
    }

    pub async fn set_deep_latch_active(&self, role_id: &str, active: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = if active { 1i64 } else { 0i64 };
        sqlx::query(
            "UPDATE role_runtime SET deep_latch_active = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(n)
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

    pub async fn preflight_turn_runtime(
        &self,
        role_id: &str,
        scene_id: &str,
        seed_interaction_mode: bool,
    ) -> Result<RoleRuntimeSnapshot> {
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
                Option<String>,
                Option<i64>,
                Option<i64>,
                Option<String>,
                Option<String>,
                i64,
            ),
        >(
            "SELECT current_favorability, current_emotion, relation_state, current_scene,
                    interaction_mode, COALESCE(remote_life_enabled, 0), mutable_personality,
                    event_impact_factor, ephemeral_personality, ephemeral_ttl_turns,
                    COALESCE(deep_latch_active, 0), continuity_scene_id,
                    continuity_state_id, continuity_revision
             FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let Some((
            favorability,
            emotion,
            relation_state,
            scene,
            interaction_mode_raw,
            remote_life_enabled,
            mutable_personality,
            event_impact_factor,
            ephemeral_personality,
            ephemeral_ttl_turns,
            deep_latch_active,
            continuity_scene_id,
            continuity_state_id,
            continuity_revision,
        )) = row
        else {
            return Err(AppError::RoleRuntimeNotReady);
        };

        let mut snapshot = RoleRuntimeSnapshot {
            favorability: Some(favorability),
            emotion,
            relation_state,
            scene,
            interaction_mode: Some(InteractionMode::normalize(interaction_mode_raw.as_deref())),
            remote_life_enabled: remote_life_enabled.map(|v| v != 0),
            mutable_personality,
            event_impact_factor,
            ephemeral_personality,
            ephemeral_ttl_turns: ephemeral_ttl_turns.map(|v| v.max(0) as u32),
            deep_latch_active: deep_latch_active.map(|v| v != 0),
            continuity_scene_id,
            continuity_state_id,
            continuity_revision: continuity_revision.max(0) as u64,
        };

        if seed_interaction_mode && interaction_mode_raw.is_none() {
            let legacy = self.get_legacy_app_interaction_mode().await?;
            let mode = if let Some(l) = legacy {
                InteractionMode::normalize(Some(l.as_str()))
            } else {
                InteractionMode::PureChat
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
            snapshot.interaction_mode = Some(mode);
        }

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

        Ok(snapshot)
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
                Option<String>,
                Option<i64>,
                Option<i64>,
                Option<String>,
                Option<String>,
                i64,
            ),
        >(
            "SELECT current_favorability, current_emotion, relation_state, current_scene,
                    interaction_mode, COALESCE(remote_life_enabled, 0), mutable_personality,
                    event_impact_factor, ephemeral_personality, ephemeral_ttl_turns,
                    COALESCE(deep_latch_active, 0), continuity_scene_id,
                    continuity_state_id, continuity_revision
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
                ephemeral_personality,
                ephemeral_ttl_turns,
                deep_latch_active,
                continuity_scene_id,
                continuity_state_id,
                continuity_revision,
            )| RoleRuntimeSnapshot {
                favorability: Some(favorability),
                emotion,
                relation_state,
                scene,
                interaction_mode: Some(InteractionMode::normalize(interaction_mode.as_deref())),
                remote_life_enabled: remote_life_enabled.map(|v| v != 0),
                mutable_personality,
                event_impact_factor,
                ephemeral_personality,
                ephemeral_ttl_turns: ephemeral_ttl_turns.map(|v| v.max(0) as u32),
                deep_latch_active: deep_latch_active.map(|v| v != 0),
                continuity_scene_id,
                continuity_state_id,
                continuity_revision: continuity_revision.max(0) as u64,
            },
        ))
    }

    pub async fn set_current_scene(&self, role_id: &str, scene_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = sqlx::query(
            "UPDATE role_runtime
             SET continuity_scene_id = CASE WHEN current_scene = ? THEN continuity_scene_id ELSE NULL END,
                 continuity_state_id = CASE WHEN current_scene = ? THEN continuity_state_id ELSE NULL END,
                 continuity_revision = CASE
                     WHEN current_scene = ? THEN continuity_revision
                     ELSE continuity_revision + 1
                 END,
                 current_scene = ?,
                 updated_at = ?
             WHERE role_id = ?",
        )
        .bind(scene_id)
        .bind(scene_id)
        .bind(scene_id)
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

    pub async fn get_narrative_continuity_state(
        &self,
        role_id: &str,
    ) -> Result<Option<(String, String, u64)>> {
        let row: Option<(Option<String>, Option<String>, i64)> = sqlx::query_as(
            "SELECT continuity_scene_id, continuity_state_id, continuity_revision
             FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(scene_id, state_id, revision)| {
            scene_id
                .zip(state_id)
                .map(|(scene_id, state_id)| (scene_id, state_id, revision.max(0) as u64))
        }))
    }

    pub async fn set_narrative_continuity_state(
        &self,
        role_id: &str,
        scene_id: &str,
        state_id: &str,
        expected_revision: u64,
    ) -> Result<Option<u64>> {
        let expected_revision = i64::try_from(expected_revision).unwrap_or(i64::MAX);
        let now = Utc::now().to_rfc3339();
        let revision: Option<i64> = sqlx::query_scalar(
            "UPDATE role_runtime
             SET continuity_scene_id = ?,
                 continuity_state_id = ?,
                 continuity_revision = continuity_revision + 1,
                 updated_at = ?
             WHERE role_id = ?
               AND continuity_revision = ?
               AND (current_scene IS NULL OR current_scene = ?)
             RETURNING continuity_revision",
        )
        .bind(scene_id)
        .bind(state_id)
        .bind(&now)
        .bind(role_id)
        .bind(expected_revision)
        .bind(scene_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(revision.map(|value| value.max(0) as u64))
    }

    pub async fn transition_narrative_continuity_state(
        &self,
        role_id: &str,
        scene_id: &str,
        from_state_id: &str,
        expected_revision: u64,
        to_state_id: &str,
    ) -> Result<bool> {
        let expected_revision = i64::try_from(expected_revision).unwrap_or(i64::MAX);
        let now = Utc::now().to_rfc3339();
        let changed = sqlx::query(
            "UPDATE role_runtime
             SET continuity_state_id = ?,
                 continuity_revision = continuity_revision + 1,
                 updated_at = ?
             WHERE role_id = ?
               AND continuity_scene_id = ?
               AND continuity_state_id = ?
               AND continuity_revision = ?",
        )
        .bind(to_state_id)
        .bind(&now)
        .bind(role_id)
        .bind(scene_id)
        .bind(from_state_id)
        .bind(expected_revision)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        Ok(changed == 1)
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

#[cfg(test)]
mod profile_evolution_atomic_tests {
    use super::*;
    use crate::infrastructure::test_db;
    use crate::models::PersonalityVector;

    #[tokio::test]
    async fn apply_profile_evolution_atomic_commits_mutable_and_delta() {
        let db = test_db::mem_db_manager().await;
        let role_id = "atomic_ok";
        db.ensure_role_runtime(role_id).await.expect("ensure");
        db.set_mutable_personality(role_id, "before")
            .await
            .expect("seed mutable");
        let core = PersonalityVector::zero();
        let delta = PersonalityVector {
            warmth: 0.5,
            ..PersonalityVector::zero()
        };
        db.apply_profile_evolution_atomic(
            role_id,
            "after",
            &core.to_json_vec(),
            &delta.to_json_vec(),
        )
        .await
        .expect("atomic");
        assert_eq!(
            db.get_mutable_personality(role_id).await.expect("read"),
            "after"
        );
        let (_, delta_s) = db
            .get_core_delta_personality_json(role_id)
            .await
            .expect("read delta");
        let stored = PersonalityVector::from_json_vec(delta_s.as_deref().unwrap()).expect("parse");
        assert!((stored.warmth - 0.5).abs() < 1e-6);
    }

    #[tokio::test]
    async fn apply_profile_evolution_atomic_rolls_back_on_delta_failure() {
        let db = test_db::mem_db_manager().await;
        let role_id = "atomic_rollback";
        db.ensure_role_runtime(role_id).await.expect("ensure");
        db.set_mutable_personality(role_id, "before")
            .await
            .expect("seed mutable");
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS test_abort_delta
             BEFORE UPDATE OF delta_personality ON role_runtime
             WHEN NEW.delta_personality LIKE '%TRIGGER_FAIL%'
             BEGIN SELECT RAISE(ABORT, 'test'); END;",
        )
        .execute(&db.pool)
        .await
        .expect("trigger");
        let core = PersonalityVector::zero();
        let delta = PersonalityVector::zero();
        let mut fail_delta = delta.to_json_vec();
        fail_delta.push_str("TRIGGER_FAIL");
        let err = db
            .apply_profile_evolution_atomic(role_id, "after", &core.to_json_vec(), &fail_delta)
            .await
            .expect_err("should fail");
        assert!(matches!(err, AppError::TransactionError { .. }));
        assert_eq!(
            db.get_mutable_personality(role_id).await.expect("read"),
            "before"
        );
    }
}

#[cfg(test)]
mod narrative_continuity_db_tests {
    use super::*;
    use crate::infrastructure::test_db;

    #[tokio::test]
    async fn continuity_transition_rejects_a_stale_revision() {
        let db = test_db::mem_db_manager().await;
        let role_id = "continuity_cas";
        db.ensure_role_runtime(role_id).await.expect("ensure");
        let revision = db
            .set_narrative_continuity_state(role_id, "home", "sofa", 0)
            .await
            .expect("initialize")
            .expect("revision");
        assert_eq!(revision, 1);
        assert_eq!(
            db.get_narrative_continuity_state(role_id)
                .await
                .expect("read"),
            Some(("home".into(), "sofa".into(), 1))
        );
        assert!(db
            .transition_narrative_continuity_state(role_id, "home", "sofa", revision, "bedroom")
            .await
            .expect("transition"));
        assert!(!db
            .transition_narrative_continuity_state(role_id, "home", "sofa", revision, "kitchen")
            .await
            .expect("stale transition"));
    }

    #[tokio::test]
    async fn changing_scene_clears_state_but_reselecting_same_scene_preserves_it() {
        let db = test_db::mem_db_manager().await;
        let role_id = "continuity_scene";
        db.ensure_role_runtime(role_id).await.expect("ensure");
        db.set_current_scene(role_id, "home")
            .await
            .expect("set scene");
        let revision = db
            .set_narrative_continuity_state(role_id, "home", "sofa", 1)
            .await
            .expect("initialize")
            .expect("revision");
        db.set_current_scene(role_id, "home")
            .await
            .expect("same scene");
        assert_eq!(
            db.get_narrative_continuity_state(role_id)
                .await
                .expect("read"),
            Some(("home".into(), "sofa".into(), revision))
        );

        db.set_current_scene(role_id, "school")
            .await
            .expect("switch scene");
        assert_eq!(
            db.get_narrative_continuity_state(role_id)
                .await
                .expect("read"),
            None
        );
        let snapshot = db
            .get_role_runtime_snapshot(role_id)
            .await
            .expect("snapshot")
            .expect("row");
        assert_eq!(snapshot.continuity_revision, revision + 1);
    }
}
