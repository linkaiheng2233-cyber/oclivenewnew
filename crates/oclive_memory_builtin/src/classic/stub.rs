//! `classic` 关闭时：无重要性排序、其余路径与完整版对齐以便侧车 / 占位仍可做关键词与上下文装配。

use oclive_kernel_core::models::{Memory, MemoryContext};

/// 轻量：按切片顺序取前 `limit` 条（**不**按 importance×weight 排序）。
#[must_use]
pub fn get_relevant_memories(memories: &[Memory], limit: usize) -> Vec<Memory> {
    memories.iter().take(limit).cloned().collect()
}

/// 与完整版相同（体量小，避免远程路径行为分叉）。
#[must_use]
pub fn search_memories(keyword: &str, memories: &[Memory]) -> Vec<Memory> {
    let keyword_lower = keyword.to_lowercase();
    memories
        .iter()
        .filter(|m| m.content.to_lowercase().contains(&keyword_lower))
        .cloned()
        .collect()
}

/// 与完整版相同的 token 估算与截断。
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
    fn stub_search_is_case_insensitive() {
        let memories = vec![m("1", "Hello Tea", 1.0, 1.0)];
        let r = search_memories("tea", &memories);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn stub_take_is_fifo_not_ranked() {
        let memories = vec![m("a", "x", 0.9, 1.0), m("b", "y", 0.1, 1.0)];
        let top = get_relevant_memories(&memories, 1);
        assert_eq!(top[0].id, "a");
    }
}
