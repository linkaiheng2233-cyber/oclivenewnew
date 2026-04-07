//! 数据访问端口（trait），由 `infrastructure` 实现

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
