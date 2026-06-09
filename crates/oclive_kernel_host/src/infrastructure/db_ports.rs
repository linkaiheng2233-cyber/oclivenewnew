//! Domain port implementations backed by [`DbManager`].

use crate::domain::ports::app_settings::AppSettingsPort;
use crate::domain::ports::db_health::DbHealthPort;
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use async_trait::async_trait;
use std::collections::HashMap;

/// Borrows [`DbManager`] as [`AppSettingsPort`] without domain FQ refs.
pub struct DbSettingsPort<'a>(pub &'a DbManager);

#[async_trait]
impl AppSettingsPort for DbSettingsPort<'_> {
    async fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        self.0.get_app_setting(key).await
    }

    async fn get_app_settings(&self, keys: &[&str]) -> Result<HashMap<String, String>> {
        self.0.get_app_settings(keys).await
    }

    async fn upsert_app_setting(&self, key: &str, value: &str) -> Result<()> {
        self.0.upsert_app_setting(key, value).await
    }
}

/// Borrows [`DbManager`] as [`DbHealthPort`] without domain FQ refs.
pub struct DbHealthPortAdapter<'a>(pub &'a DbManager);

#[async_trait]
impl DbHealthPort for DbHealthPortAdapter<'_> {
    async fn health_ping(&self) -> Result<()> {
        self.0.health_ping().await
    }
}
