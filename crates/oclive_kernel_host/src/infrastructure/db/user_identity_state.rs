//! [`DbManager`](super::DbManager) methods for User Identity Prompt Template session state.

#![allow(clippy::missing_errors_doc)]

use super::DbManager;
use crate::error::{AppError, Result};
use chrono::Utc;

impl DbManager {
    /// Single round-trip for global identity session state (`use_manifest_default`, `active_user_identity_id`).
    pub async fn get_global_identity_state(&self, role_id: &str) -> Result<(bool, Option<String>)> {
        let row: Option<(i64, Option<String>)> = sqlx::query_as(
            "SELECT COALESCE(use_manifest_default_identity, 1), active_user_identity_id FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row
            .map(|(use_manifest, active_id)| (use_manifest != 0, active_id))
            .unwrap_or((true, None)))
    }

    pub async fn get_use_manifest_default_identity(&self, role_id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(use_manifest_default_identity, 1) FROM role_runtime WHERE role_id = ?",
        )
        .bind(role_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|(v,)| v != 0).unwrap_or(true))
    }

    pub async fn set_use_manifest_default_identity(&self, role_id: &str, v: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let n = if v { 1i64 } else { 0i64 };
        sqlx::query(
            "UPDATE role_runtime SET use_manifest_default_identity = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(n)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_active_user_identity_id(&self, role_id: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT active_user_identity_id FROM role_runtime WHERE role_id = ?")
                .bind(role_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(s,)| s))
    }

    pub async fn set_active_user_identity_id(
        &self,
        role_id: &str,
        identity_id: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_runtime SET active_user_identity_id = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(identity_id)
        .bind(&now)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_user_identity_id_for_scene(
        &self,
        role_id: &str,
        scene_id: &str,
    ) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT user_identity_id FROM role_scene_identity WHERE role_id = ? AND scene_id = ?",
        )
        .bind(role_id)
        .bind(scene_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.and_then(|(s,)| s))
    }

    pub async fn set_user_identity_for_scene(
        &self,
        role_id: &str,
        scene_id: &str,
        identity_id: &str,
        relation: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO role_scene_identity (role_id, scene_id, user_relation, user_identity_id, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(role_id, scene_id)
             DO UPDATE SET user_identity_id = excluded.user_identity_id,
                           user_relation = excluded.user_relation,
                           updated_at = excluded.updated_at",
        )
        .bind(role_id)
        .bind(scene_id)
        .bind(relation)
        .bind(identity_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn clear_user_identity_for_scene(&self, role_id: &str, scene_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_scene_identity SET user_identity_id = NULL, updated_at = ? WHERE role_id = ? AND scene_id = ?",
        )
        .bind(&now)
        .bind(role_id)
        .bind(scene_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
