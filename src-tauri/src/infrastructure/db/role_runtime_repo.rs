//! [`DbManager`] 子集 trait，便于 domain / 测试按表注入 mock。

use super::DbManager;
use crate::error::Result;
use async_trait::async_trait;

/// `role_runtime` 表常用读路径（好感 / 情绪 / 场景）。
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
