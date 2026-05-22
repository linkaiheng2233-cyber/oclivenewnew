//! 记忆检索门面输入（纯数据结构）。

use crate::models::Memory;

/// 与 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 对齐的检索输入
pub struct MemoryRetrievalInput<'a> {
    pub memories: &'a [Memory],
    pub user_query: &'a str,
    pub scene_id: Option<&'a str>,
    pub limit: usize,
}
