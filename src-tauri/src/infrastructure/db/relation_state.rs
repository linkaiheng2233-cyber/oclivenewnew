//! `relation state` 相关 [`DbManager`](super::DbManager) 方法。

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::DbManager;
use crate::error::{AppError, Result};
use chrono::Utc;

impl DbManager {
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

    pub async fn clear_all_scene_identities_for_role(&self, role_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM role_scene_identity WHERE role_id = ?")
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

    pub async fn set_identity_relation_state(
        &self,
        role_id: &str,
        user_relation_key: &str,
        relation_state: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE role_identity_stats SET relation_state = ?, updated_at = ? WHERE role_id = ? AND user_relation_key = ?",
        )
        .bind(relation_state)
        .bind(&now)
        .bind(role_id)
        .bind(user_relation_key)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        self.mirror_runtime_from_identity(role_id, user_relation_key)
            .await
    }

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
}
