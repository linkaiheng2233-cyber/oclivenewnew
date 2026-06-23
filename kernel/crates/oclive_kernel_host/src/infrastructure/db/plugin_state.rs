//! [`DbManager`](super::DbManager) methods for `plugin state` / `app_settings`.

#![allow(clippy::missing_errors_doc, unused_imports)]

use super::DbManager;
use crate::error::{AppError, Result};
use std::collections::HashMap;

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

    /// Batch-read app settings; missing keys are omitted from the map.
    pub async fn get_app_settings(&self, keys: &[&str]) -> Result<HashMap<String, String>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let placeholders = std::iter::repeat_n("?", keys.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT key, value FROM app_settings WHERE key IN ({placeholders})");
        let mut query = sqlx::query_as::<_, (String, String)>(&sql);
        for key in keys {
            query = query.bind(key);
        }
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(rows.into_iter().collect())
    }
}
