//! [`DbManager`](super::DbManager) role-runtime methods: favorability, emotion, relation, and scene.

#![allow(clippy::missing_errors_doc, unused_imports)]

use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use crate::models::*;
use chrono::Utc;
use sqlx::Row;
use std::time::Instant;

impl DbManager {
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
}
