//! Repository 具体实现（SQLx / DbManager）

use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::models::Memory;
use async_trait::async_trait;
use std::sync::Arc;

/// 基于 `DbManager` 的长期记忆仓库
pub struct SqliteMemoryRepository {
    inner: Arc<DbManager>,
}

impl SqliteMemoryRepository {
    pub fn new(inner: Arc<DbManager>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl MemoryRepository for SqliteMemoryRepository {
    async fn save_memory(&self, role_id: &str, content: &str, importance: f64) -> Result<String> {
        self.inner.save_memory(role_id, content, importance).await
    }

    async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>> {
        self.inner.load_memories(role_id, limit).await
    }

    async fn count_memories(&self, role_id: &str) -> Result<i64> {
        self.inner.count_memories(role_id).await
    }

    async fn load_memories_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Memory>> {
        self.inner.load_memories_paged(role_id, limit, offset).await
    }
}

/// 基于 `DbManager` 的好感度仓库
pub struct SqliteFavorabilityRepository {
    inner: Arc<DbManager>,
}

impl SqliteFavorabilityRepository {
    pub fn new(inner: Arc<DbManager>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl FavorabilityRepository for SqliteFavorabilityRepository {
    async fn get(&self, role_id: &str) -> Result<Option<f64>> {
        self.inner.get_favorability(role_id).await
    }

    async fn apply_delta(&self, role_id: &str, delta: f64) -> Result<()> {
        self.inner.apply_favorability_delta(role_id, delta).await
    }
}
