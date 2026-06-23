//! Swappable memory-retrieval facade; default delegates to [`MemoryEngine`](super::memory_engine::MemoryEngine).

use crate::domain::memory_engine::MemoryEngine;
use crate::error::Result;
use crate::models::{Memory, MemoryContext};
pub use oclive_kernel_contracts::MemoryRetrieval;
pub use oclive_kernel_types::MemoryRetrievalInput;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Builtin: sort by importance × weight (matches historical behavior).
pub struct BuiltinMemoryRetrieval;

impl MemoryRetrieval for BuiltinMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        Ok(MemoryEngine::get_relevant_memories(
            input.memories,
            input.limit,
        ))
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
    }
}

/// Remote placeholder: falls back to builtin and logs a one-time warning.
pub struct RemoteMemoryRetrievalPlaceholder {
    inner: BuiltinMemoryRetrieval,
    warned: AtomicBool,
}

impl RemoteMemoryRetrievalPlaceholder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: BuiltinMemoryRetrieval,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_plugin",
                "memory backend Remote is not connected; using builtin ranking"
            );
        }
    }
}

impl MemoryRetrieval for RemoteMemoryRetrievalPlaceholder {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        self.warn_once();
        self.inner.rank_memories(input)
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        self.warn_once();
        self.inner.build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        self.warn_once();
        self.inner.search_memories(keyword, memories)
    }
}

impl Default for RemoteMemoryRetrievalPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

/// `plugin_backends.memory = local`: registry-selected local provider (observability + future hook; ranking delegates to `fallback`).
pub struct LocalPluginMemoryRetrieval {
    fallback: Arc<dyn MemoryRetrieval>,
    resolved_provider_id: Option<String>,
}

impl LocalPluginMemoryRetrieval {
    pub fn new(fallback: Arc<dyn MemoryRetrieval>, resolved_provider_id: Option<String>) -> Self {
        Self {
            fallback,
            resolved_provider_id,
        }
    }
}

impl MemoryRetrieval for LocalPluginMemoryRetrieval {
    fn diagnostic_local_provider_id(&self) -> Option<&str> {
        self.resolved_provider_id.as_deref()
    }

    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        if let Some(id) = &self.resolved_provider_id {
            tracing::debug!(
                target: "oclive_plugin",
                "memory.local rank_memories provider_id={} (stub delegates to builtin slot)",
                id
            );
        }
        self.fallback.rank_memories(input)
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        self.fallback.build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        self.fallback.search_memories(keyword, memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Memory;
    use chrono::Utc;
    use std::sync::Arc;

    #[test]
    fn builtin_ranks_by_importance_weight() {
        let t = Utc::now();
        let m_high = Memory {
            id: "high".into(),
            role_id: "r".into(),
            content: "no overlap with query".into(),
            importance: 1.01,
            weight: 1.0,
            created_at: t,
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        };
        let m_low = Memory {
            id: "low".into(),
            role_id: "r".into(),
            content: "matchtoken appears here".into(),
            importance: 1.0,
            weight: 1.0,
            created_at: t,
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        };
        let slice = &[m_high.clone(), m_low.clone()];
        let input = MemoryRetrievalInput {
            memories: slice,
            user_query: "matchtoken",
            scene_id: None,
            limit: 1,
        };
        let top = BuiltinMemoryRetrieval.rank_memories(input).expect("rank");
        assert_eq!(top[0].id, "high");
    }

    #[test]
    fn local_plugin_memory_stub_matches_fallback_ranking() {
        let t = Utc::now();
        let m_a = Memory {
            id: "a".into(),
            role_id: "r".into(),
            content: "alpha token".into(),
            importance: 1.0,
            weight: 1.0,
            created_at: t,
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        };
        let m_b = Memory {
            id: "b".into(),
            role_id: "r".into(),
            content: "no overlap".into(),
            importance: 1.2,
            weight: 1.0,
            created_at: t,
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        };
        let slice = &[m_a.clone(), m_b.clone()];
        let mk_input = || MemoryRetrievalInput {
            memories: slice,
            user_query: "alpha",
            scene_id: None,
            limit: 1,
        };
        let builtin = Arc::new(BuiltinMemoryRetrieval) as Arc<dyn MemoryRetrieval>;
        let local = LocalPluginMemoryRetrieval::new(builtin.clone(), Some("demo.local".into()));
        let a: Vec<_> = local
            .rank_memories(mk_input())
            .expect("rank")
            .into_iter()
            .map(|m| m.id)
            .collect();
        let b: Vec<_> = builtin
            .rank_memories(mk_input())
            .expect("rank")
            .into_iter()
            .map(|m| m.id)
            .collect();
        assert_eq!(a, b);
    }
}
