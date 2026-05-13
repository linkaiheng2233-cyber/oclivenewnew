//! `feature = "providers"`：进程内 Builtin / BuiltinV2。

use crate::classic;
use oclive_kernel_core::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
use oclive_kernel_core::models::{Memory, MemoryContext};

/// 内置：按重要性 × 权重排序（与历史行为一致）。
pub struct BuiltinMemoryRetrieval;

impl MemoryRetrieval for BuiltinMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Vec<Memory> {
        classic::get_relevant_memories(input.memories, input.limit)
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        classic::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        classic::search_memories(keyword, memories)
    }
}

/// 第二套内置：在 builtin 分数上叠加与用户查询的正文重合度。
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
        classic::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        classic::search_memories(keyword, memories)
    }
}
