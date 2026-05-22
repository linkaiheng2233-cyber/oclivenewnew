//! 记忆检索可替换门面 trait。

use oclive_kernel_types::{Memory, MemoryContext, MemoryRetrievalInput, Result};

/// Ranks and formats memories for prompt injection.
///
/// ## When to implement
///
/// - **谁**：记忆检索后端（内置排序、本地目录插件、Remote HTTP）。
/// - **何时**：角色启用记忆系统并需在 Prompt 中注入相关记忆时。
///
/// ## When not to implement
///
/// - 无长期记忆、或固定使用内置 `BuiltinMemoryRetrieval` 且行为已满足需求时。
pub trait MemoryRetrieval: Send + Sync {
    /// 按相关性对记忆排序。
    ///
    /// # Errors
    ///
    /// 实现方在检索/排序失败时返回 [`Result`] 的 `Err` 变体。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>>;

    /// 将记忆列表格式化为可注入 Prompt 的上下文。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext;

    /// 按关键词在已有记忆中搜索。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory>;

    /// Telemetry hook: local-plugin retrieval may expose the selected `provider_id` (default `None`).
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    #[must_use]
    fn diagnostic_local_provider_id(&self) -> Option<&str> {
        None
    }
}
