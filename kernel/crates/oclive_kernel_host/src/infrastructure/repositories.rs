//! Concrete repository implementations (SQLx / DbManager)

use crate::domain::repository::{FavorabilityRepository, MemoryRepository};
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use crate::models::Memory;
use async_trait::async_trait;
use std::sync::Arc;

/// Long-term memory repository backed by `DbManager`
pub struct SqliteMemoryRepository {
    inner: Arc<DbManager>,
}

impl SqliteMemoryRepository {
    #[must_use]
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

    async fn load_memories_for_context(
        &self,
        role_id: &str,
        limit: i32,
        include_adult: bool,
    ) -> Result<Vec<Memory>> {
        self.inner
            .load_memories_for_context(role_id, limit, include_adult)
            .await
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

    async fn load_memories_paged_for_scope(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
        scope: Option<&str>,
    ) -> Result<Vec<(Memory, String)>> {
        self.inner
            .load_memories_paged_for_scope(role_id, limit, offset, scope)
            .await
    }

    async fn save_memory_merged(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        similarity_threshold: f64,
        scene_id: &str,
    ) -> Result<String> {
        self.inner
            .save_memory_merged(role_id, content, importance, similarity_threshold, scene_id)
            .await
    }

    async fn delete_memory_for_role(&self, role_id: &str, memory_id: &str) -> Result<bool> {
        self.inner.delete_memory_for_role(role_id, memory_id).await
    }
}

/// Favorability repository backed by `DbManager`
pub struct SqliteFavorabilityRepository {
    inner: Arc<DbManager>,
}

impl SqliteFavorabilityRepository {
    #[must_use]
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
