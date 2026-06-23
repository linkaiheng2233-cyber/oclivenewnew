//! [`DbManager`](super::super::DbManager) interaction-mode fields on `role_runtime`.

#![allow(clippy::missing_errors_doc)]

use super::super::DbManager;
use crate::error::{AppError, Result};
use crate::models::InteractionMode;
use chrono::Utc;

impl DbManager {
    /// Legacy global `app_settings.interaction_mode` (migration only).
    pub(super) async fn get_legacy_app_interaction_mode(&self) -> Result<Option<String>> {
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
        distro_default: Option<&str>,
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
        let _ = (pack_default, distro_default);
        let legacy = self.get_legacy_app_interaction_mode().await?;
        // First run: always pure_chat. User choice persists in role_runtime after set_interaction_mode.
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
        self.ensure_role_runtime(role_id).await?;
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
            sqlx::query(
                "INSERT INTO role_runtime (role_id, interaction_mode, current_favorability, updated_at)
                 VALUES (?, ?, 0.0, ?)",
            )
            .bind(role_id)
            .bind(normalized.as_str())
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::DbManager;
    use crate::infrastructure::test_db;
    use crate::models::InteractionMode;

    #[tokio::test]
    async fn set_interaction_mode_for_role_upserts_without_prior_runtime_row() {
        let db = DbManager::new(test_db::connect_memory_migrated().await);
        let role_id = "mumu_test_mode";

        db.set_interaction_mode_for_role(role_id, InteractionMode::IMMERSIVE)
            .await
            .expect("set interaction mode on fresh role");

        let mode = db
            .get_interaction_mode(role_id)
            .await
            .expect("read interaction mode");
        assert_eq!(mode, InteractionMode::Immersive);

        db.set_interaction_mode_for_role(role_id, InteractionMode::PURE_CHAT)
            .await
            .expect("update interaction mode");
        let mode = db
            .get_interaction_mode(role_id)
            .await
            .expect("read updated interaction mode");
        assert_eq!(mode, InteractionMode::PureChat);
    }
}
