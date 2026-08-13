//! [`DbManager`](super::DbManager) role-runtime methods: per-session Ollama model overrides.

#![allow(clippy::missing_errors_doc, unused_imports)]

use crate::domain::role_runtime_snapshot::RoleRuntimeSnapshot;
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use crate::models::*;
use chrono::Utc;
use sqlx::Row;
use std::time::Instant;

impl DbManager {
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
