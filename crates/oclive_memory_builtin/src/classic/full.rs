//! 与历史 `MemoryEngine` 静态方法一致的纯算法（无数据库、无全局状态）。

use oclive_kernel_core::models::{Memory, MemoryContext};

/// 按 **importance × weight** 降序取前 `limit` 条。
#[must_use]
pub fn get_relevant_memories(memories: &[Memory], limit: usize) -> Vec<Memory> {
    let mut sorted = memories.to_vec();
    sorted.sort_by(|a, b| {
        let score_a = a.importance * a.weight;
        let score_b = b.importance * b.weight;
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.into_iter().take(limit).collect()
}

/// 关键词子串匹配（大小写不敏感）。
#[must_use]
pub fn search_memories(keyword: &str, memories: &[Memory]) -> Vec<Memory> {
    let keyword_lower = keyword.to_lowercase();
    memories
        .iter()
        .filter(|m| m.content.to_lowercase().contains(&keyword_lower))
        .cloned()
        .collect()
}

/// 按内容长度粗略估计 token，在 `max_tokens` 内装入尽可能多的记忆。
#[must_use]
pub fn build_context(memories: &[Memory], max_tokens: usize) -> MemoryContext {
    let mut context_memories = Vec::new();
    let mut total_tokens = 0;

    for memory in memories {
        let tokens = memory.content.len() / 4;
        if total_tokens + tokens <= max_tokens {
            context_memories.push(memory.clone());
            total_tokens += tokens;
        } else {
            break;
        }
    }

    MemoryContext {
        memories: context_memories,
        total_tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn m(id: &str, content: &str, importance: f64, weight: f64) -> Memory {
        Memory {
            id: id.to_string(),
            role_id: "r".into(),
            content: content.into(),
            importance,
            weight,
            created_at: Utc::now(),
            scene_id: None,
        }
    }

    #[test]
    fn get_relevant_orders_by_score() {
        let memories = vec![m("a", "x", 0.5, 1.0), m("b", "y", 0.9, 1.0)];
        let top = get_relevant_memories(&memories, 1);
        assert_eq!(top[0].id, "b");
    }

    #[test]
    fn search_is_case_insensitive() {
        let memories = vec![m("1", "Hello Tea", 1.0, 1.0)];
        let r = search_memories("tea", &memories);
        assert_eq!(r.len(), 1);
    }
}
