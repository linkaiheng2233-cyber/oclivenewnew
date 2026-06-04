//! [`DbManager`] subset trait for per-table mocks in domain / tests.

use super::DbManager;
use crate::error::Result;
use async_trait::async_trait;

/// Common `role_runtime` read paths (favorability / emotion / scene).
#[async_trait]
pub trait RoleRuntimeRepo: Send + Sync {
    async fn get_favorability(&self, role_id: &str) -> Result<Option<f64>>;
    async fn get_current_emotion(&self, role_id: &str) -> Result<Option<String>>;
    async fn get_current_scene(&self, role_id: &str) -> Result<Option<String>>;
}

#[async_trait]
impl RoleRuntimeRepo for super::DbManager {
    async fn get_favorability(&self, role_id: &str) -> Result<Option<f64>> {
        DbManager::get_favorability(self, role_id).await
    }

    async fn get_current_emotion(&self, role_id: &str) -> Result<Option<String>> {
        DbManager::get_current_emotion(self, role_id).await
    }

    async fn get_current_scene(&self, role_id: &str) -> Result<Option<String>> {
        DbManager::get_current_scene(self, role_id).await
    }
}
