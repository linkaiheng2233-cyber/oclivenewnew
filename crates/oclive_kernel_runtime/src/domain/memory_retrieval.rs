//! 记忆检索可替换门面；默认实现见设施 crate [`oclive_memory_builtin`]。

#[cfg(not(feature = "default-memory-providers"))]
use crate::domain::disabled_default_providers::DisabledMemoryRetrieval;
use crate::models::{Memory, MemoryContext};
pub use oclive_kernel_core::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
#[cfg(feature = "default-memory-providers")]
pub use oclive_memory_builtin::{BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[must_use]
pub fn default_memory_slot_v1() -> Arc<dyn MemoryRetrieval> {
    #[cfg(feature = "default-memory-providers")]
    {
        Arc::new(BuiltinMemoryRetrieval)
    }
    #[cfg(not(feature = "default-memory-providers"))]
    {
        Arc::new(DisabledMemoryRetrieval)
    }
}

#[must_use]
pub fn default_memory_slot_v2() -> Arc<dyn MemoryRetrieval> {
    #[cfg(feature = "default-memory-providers")]
    {
        Arc::new(BuiltinMemoryRetrievalV2)
    }
    #[cfg(not(feature = "default-memory-providers"))]
    {
        Arc::new(DisabledMemoryRetrieval)
    }
}

/// Remote 占位：回退 builtin 并记一次警告
pub struct RemoteMemoryRetrievalPlaceholder {
    inner: Arc<dyn MemoryRetrieval>,
    warned: AtomicBool,
}

impl RemoteMemoryRetrievalPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: default_memory_slot_v1(),
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::warn!(
                target: "oclive_plugin",
                "memory backend Remote is not connected; using builtin ranking"
            );
        }
    }
}

impl MemoryRetrieval for RemoteMemoryRetrievalPlaceholder {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
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

/// `plugin_backends.memory = local`：按注册表选中的本地 provider（当前仅用于观测与后续接入；排序委托 `fallback`）。
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

    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        if let Some(id) = &self.resolved_provider_id {
            log::debug!(
                target: "oclive_plugin",
                "memory.local rank_memories provider_id={} (stub delegates to builtin_v2 slot)",
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

#[cfg(all(test, feature = "default-memory-providers"))]
mod tests {
    use super::*;
    use crate::models::Memory;
    use chrono::Utc;
    use std::sync::Arc;

    #[test]
    fn builtin_v2_can_outrank_higher_score_when_query_overlaps() {
        let t = Utc::now();
        let m_high = Memory {
            id: "high".into(),
            role_id: "r".into(),
            content: "no overlap with query".into(),
            importance: 1.01,
            weight: 1.0,
            created_at: t,
            scene_id: None,
        };
        let m_match = Memory {
            id: "match".into(),
            role_id: "r".into(),
            content: "matchtoken appears here".into(),
            importance: 1.0,
            weight: 1.0,
            created_at: t,
            scene_id: None,
        };
        let slice = &[m_high.clone(), m_match.clone()];
        let input_v1 = MemoryRetrievalInput {
            memories: slice,
            user_query: "matchtoken",
            scene_id: None,
            limit: 1,
        };
        let top_v1 = BuiltinMemoryRetrieval.rank_memories(input_v1);
        assert_eq!(top_v1[0].id, "high");

        let input_v2 = MemoryRetrievalInput {
            memories: slice,
            user_query: "matchtoken",
            scene_id: None,
            limit: 1,
        };
        let top_v2 = BuiltinMemoryRetrievalV2.rank_memories(input_v2);
        assert_eq!(top_v2[0].id, "match");
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
        };
        let m_b = Memory {
            id: "b".into(),
            role_id: "r".into(),
            content: "no overlap".into(),
            importance: 1.2,
            weight: 1.0,
            created_at: t,
            scene_id: None,
        };
        let slice = &[m_a.clone(), m_b.clone()];
        let mk_input = || MemoryRetrievalInput {
            memories: slice,
            user_query: "alpha",
            scene_id: None,
            limit: 1,
        };
        let v2 = Arc::new(BuiltinMemoryRetrievalV2) as Arc<dyn MemoryRetrieval>;
        let local = LocalPluginMemoryRetrieval::new(v2.clone(), Some("demo.local".into()));
        let a: Vec<_> = local
            .rank_memories(mk_input())
            .into_iter()
            .map(|m| m.id)
            .collect();
        let b: Vec<_> = v2
            .rank_memories(mk_input())
            .into_iter()
            .map(|m| m.id)
            .collect();
        assert_eq!(a, b);
    }
}
