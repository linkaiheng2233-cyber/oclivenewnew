//! App settings DB port (decouples domain from `DbManager`).

use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Read/write `app_settings` rows for LLM env sync and related keys.
#[async_trait]
pub trait AppSettingsPort: Send + Sync {
    /// # Errors
    ///
    /// Database read failures propagate as [`crate::error::AppError`].
    async fn get_app_setting(&self, key: &str) -> Result<Option<String>>;

    /// # Errors
    ///
    /// Database read failures propagate as [`crate::error::AppError`].
    async fn get_app_settings(&self, keys: &[&str]) -> Result<HashMap<String, String>>;

    /// # Errors
    ///
    /// Database write failures propagate as [`crate::error::AppError`].
    async fn upsert_app_setting(&self, key: &str, value: &str) -> Result<()>;
}
