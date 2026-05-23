//! `plugin state` / `app_settings` 相关 [`DbManager`](super::DbManager) 方法。

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::DbManager;
use crate::error::{AppError, Result};

impl DbManager {
    pub async fn upsert_app_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ? LIMIT 1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
