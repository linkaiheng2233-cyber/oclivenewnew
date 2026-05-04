//! 数据访问端口（trait），由 `oclive_kernel_runtime::infrastructure` 实现。

use crate::error::Result;
use crate::models::Memory;
use async_trait::async_trait;

#[async_trait]
pub trait MemoryRepository: Send + Sync {
    async fn save_memory(&self, role_id: &str, content: &str, importance: f64) -> Result<String>;
    async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>>;
    async fn count_memories(&self, role_id: &str) -> Result<i64>;
    async fn load_memories_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Memory>>;
}

#[async_trait]
pub trait FavorabilityRepository: Send + Sync {
    async fn get(&self, role_id: &str) -> Result<Option<f64>>;
    async fn apply_delta(&self, role_id: &str, delta: f64) -> Result<()>;
}

/// Module 9: Expert Models + PromptStyle runtime JSON persistence on `role_runtime`.
///
/// - Role default: stored on manifest `role_id` row.
/// - Session override: stored on session namespace row (`role_id__sess__xxx`).
#[async_trait]
pub trait ExpertModelsRepository: Send + Sync {
    async fn get_expert_models_role_default_json(&self, role_id: &str) -> Result<Option<String>>;
    async fn set_expert_models_role_default_json(
        &self,
        role_id: &str,
        json: Option<&str>,
    ) -> Result<()>;

    async fn get_expert_models_session_override_json(
        &self,
        session_namespace: &str,
    ) -> Result<Option<String>>;
    async fn set_expert_models_session_override_json(
        &self,
        session_namespace: &str,
        json: Option<&str>,
    ) -> Result<()>;

    async fn get_expert_prompt_style_role_default_json(
        &self,
        role_id: &str,
    ) -> Result<Option<String>>;
    async fn set_expert_prompt_style_role_default_json(
        &self,
        role_id: &str,
        json: Option<&str>,
    ) -> Result<()>;

    async fn get_expert_prompt_style_session_override_json(
        &self,
        session_namespace: &str,
    ) -> Result<Option<String>>;
    async fn set_expert_prompt_style_session_override_json(
        &self,
        session_namespace: &str,
        json: Option<&str>,
    ) -> Result<()>;

    async fn get_expert_models_run_history_json(
        &self,
        session_namespace: &str,
    ) -> Result<Option<String>>;
    async fn set_expert_models_run_history_json(
        &self,
        session_namespace: &str,
        json: Option<&str>,
    ) -> Result<()>;
}
