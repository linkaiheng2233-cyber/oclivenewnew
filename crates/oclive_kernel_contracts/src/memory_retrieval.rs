//! Replaceable facade trait for memory retrieval.

use oclive_kernel_types::{Memory, MemoryContext, MemoryRetrievalInput, Result};

/// Ranks and formats memories for prompt injection.
///
/// ## When to implement
///
/// - **Who**: memory retrieval backends (builtin ranking, local directory plugin, Remote HTTP).
/// - **When**: when a role enables the memory system and needs relevant memories injected into the prompt.
///
/// ## When not to implement
///
/// - When there is no long-term memory, or the builtin `BuiltinMemoryRetrieval` is used as-is and its behavior already meets the requirements.
pub trait MemoryRetrieval: Send + Sync {
    /// Ranks memories by relevance.
    ///
    /// # Errors
    ///
    /// The implementation returns the `Err` variant of [`Result`] when retrieval/ranking fails.
    ///
    /// # Panics
    ///
    /// Does not panic.
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
