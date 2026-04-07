//! 记忆检索可替换门面；默认实现委托 [`MemoryEngine`](super::memory_engine::MemoryEngine)。

use crate::domain::memory_engine::MemoryEngine;
use crate::models::{Memory, MemoryContext};
use std::sync::atomic::{AtomicBool, Ordering};

/// 与 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 对齐的检索输入
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
}

/// 内置：按重要性 × 权重排序（与历史行为一致）
pub struct BuiltinMemoryRetrieval;

impl MemoryRetrieval for BuiltinMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        MemoryEngine::get_relevant_memories(input.memories, input.limit)
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
    }
}

/// 第二套内置：在 builtin 分数上叠加与用户查询的正文重合度
pub struct BuiltinMemoryRetrievalV2;

fn query_overlap_boost(query: &str, content: &str) -> f64 {
    let q = query.trim();
    if q.is_empty() {
        return 0.0;
    }
    let ql = q.to_lowercase();
    let cl = content.to_lowercase();
    let mut hits = 0usize;
    for w in ql.split_whitespace() {
        if w.len() >= 2 && cl.contains(w) {
            hits += 1;
        }
    }
    if hits == 0 && ql.chars().count() >= 2 {
        for w in ql.as_str().chars().collect::<Vec<_>>().windows(2) {
            let s: String = w.iter().collect();
            if cl.contains(&s) {
                hits += 1;
            }
        }
    }
    (hits as f64 * 0.15).min(0.6)
}

impl MemoryRetrieval for BuiltinMemoryRetrievalV2 {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        let limit = input.limit.max(1);
        let q = input.user_query;
        let mut scored: Vec<(f64, Memory)> = input
            .memories
            .iter()
            .map(|m| {
                let base = m.importance * m.weight;
                let boost = query_overlap_boost(q, &m.content);
                (base * (1.0 + boost), m.clone())
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(_, m)| m).collect()
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
    }
}

/// Remote 占位：回退 builtin 并记一次警告
pub struct RemoteMemoryRetrievalPlaceholder {
    inner: BuiltinMemoryRetrieval,
    warned: AtomicBool,
}

impl RemoteMemoryRetrievalPlaceholder {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Memory;
    use chrono::Utc;

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
}
