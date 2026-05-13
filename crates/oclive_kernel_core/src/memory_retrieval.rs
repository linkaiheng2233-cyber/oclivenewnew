//! 记忆检索可替换门面。

use crate::models::{Memory, MemoryContext};

/// 与 `PLUGIN_V1` 对齐的检索输入
pub struct MemoryRetrievalInput<'a> {
    pub memories: &'a [Memory],
    pub user_query: &'a str,
    pub scene_id: Option<&'a str>,
    pub limit: usize,
}

pub trait MemoryRetrieval: Send + Sync {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory>;
    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext;
    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory>;

    /// 遥测 / 单测：仅 `LocalPluginMemoryRetrieval` 返回选中的本地 `provider_id`。
    #[must_use]
    fn diagnostic_local_provider_id(&self) -> Option<&str> {
        None
    }
}
