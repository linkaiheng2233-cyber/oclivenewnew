//! Memory-retrieval facade input (pure data structures).

use crate::models::Memory;

/// Retrieval input aligned with `creator-docs/plugin-and-architecture/PLUGIN_V1.md`
pub struct MemoryRetrievalInput<'a> {
    pub memories: &'a [Memory],
    pub user_query: &'a str,
    pub scene_id: Option<&'a str>,
    pub limit: usize,
}
