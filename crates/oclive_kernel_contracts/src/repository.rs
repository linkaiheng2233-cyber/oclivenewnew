//! 数据访问端口（trait），由宿主 `infrastructure` 实现。

use async_trait::async_trait;
use oclive_kernel_types::{Memory, Result};

/// Persistence port for role-scoped long-term memories.
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// 持久化一条长期记忆并返回其 ID。
    ///
    /// # Errors
    ///
    /// 数据库 I/O 失败、约束冲突或序列化失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn save_memory(&self, role_id: &str, content: &str, importance: f64) -> Result<String>;

    /// 按角色加载最近若干条记忆。
    ///
    /// # Errors
    ///
    /// 数据库查询失败或行反序列化失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>>;

    /// 统计角色下记忆条数。
    ///
    /// # Errors
    ///
    /// 数据库查询失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn count_memories(&self, role_id: &str) -> Result<i64>;

    /// 分页加载角色记忆。
    ///
    /// # Errors
    ///
    /// 数据库查询失败或行反序列化失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn load_memories_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Memory>>;
}

/// Persistence port for role favorability scores.
#[async_trait]
pub trait FavorabilityRepository: Send + Sync {
    /// 读取角色当前好感度。
    ///
    /// # Errors
    ///
    /// 数据库查询失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn get(&self, role_id: &str) -> Result<Option<f64>>;

    /// 对角色好感度应用增量变更。
    ///
    /// # Errors
    ///
    /// 数据库读写失败或更新约束冲突时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    async fn apply_delta(&self, role_id: &str, delta: f64) -> Result<()>;
}
