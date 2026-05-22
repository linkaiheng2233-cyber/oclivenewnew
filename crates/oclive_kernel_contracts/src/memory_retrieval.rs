//! 记忆检索可替换门面 trait。

use oclive_kernel_types::{Memory, MemoryContext, MemoryRetrievalInput, Result};

pub trait MemoryRetrieval: Send + Sync {
    /// 按相关性对记忆排序。
    ///
    /// # Errors
    ///
    /// 实现方在检索/排序失败时返回 [`Result`] 的 `Err` 变体。
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>>;
    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext;
    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory>;

    /// 遥测 / 单测：仅 `LocalPluginMemoryRetrieval` 返回选中的本地 `provider_id`。
    #[must_use]
    fn diagnostic_local_provider_id(&self) -> Option<&str> {
        None
    }
}
