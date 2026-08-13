//! [`DbManager`](super::DbManager) role-runtime methods: turn preflight, snapshots, and narrative continuity.

#![allow(clippy::missing_errors_doc, unused_imports)]

use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use crate::models::*;
use chrono::Utc;
use sqlx::Row;
use std::time::Instant;

impl DbManager {
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
}
