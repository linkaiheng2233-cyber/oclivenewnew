//! Domain port implementations backed by [`DbManager`].

use crate::domain::ports::app_settings::AppSettingsPort;
use crate::domain::ports::db_health::DbHealthPort;
use crate::domain::ports::turn_thinking_state::TurnThinkingStatePort;
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

/// Borrows [`DbManager`] as [`TurnThinkingStatePort`] without domain FQ refs.
pub struct TurnThinkingStateAdapter<'a>(pub &'a DbManager);

#[async_trait]
impl TurnThinkingStatePort for TurnThinkingStateAdapter<'_> {
    async fn set_deep_latch_active(&self, role_id: &str, active: bool) -> Result<()> {
        self.0.set_deep_latch_active(role_id, active).await
    }

    async fn get_ephemeral_ttl_turns(&self, role_id: &str) -> Result<u32> {
        self.0.get_ephemeral_ttl_turns(role_id).await
    }

    async fn get_ephemeral_personality(&self, role_id: &str) -> Result<String> {
        self.0.get_ephemeral_personality(role_id).await
    }

    async fn set_ephemeral_personality(&self, role_id: &str, text: &str) -> Result<()> {
        self.0.set_ephemeral_personality(role_id, text).await
    }

    async fn set_ephemeral_ttl_turns(&self, role_id: &str, ttl: u32) -> Result<()> {
        self.0.set_ephemeral_ttl_turns(role_id, ttl).await
    }
}
