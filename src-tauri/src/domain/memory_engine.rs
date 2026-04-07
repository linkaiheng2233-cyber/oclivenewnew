//! 记忆引擎模块
//! 管理短期和长期记忆，支持记忆检索和更新

use crate::models::{Memory, MemoryContext};
use std::collections::VecDeque;

/// 短期记忆缓冲区（最多保留最近N条对话）
const SHORT_TERM_CAPACITY: usize = 10;

/// 记忆引擎
pub struct MemoryEngine {
    short_term: VecDeque<Memory>,
}

impl MemoryEngine {
    /// 创建新的记忆引擎
    pub fn new() -> Self {
        Self {
            short_term: VecDeque::with_capacity(SHORT_TERM_CAPACITY),
        }
    }

    /// 添加短期记忆
    ///
    /// # Arguments
    /// * `memory` - 记忆对象
    pub fn add_short_term(&mut self, memory: Memory) {
        if self.short_term.len() >= SHORT_TERM_CAPACITY {
            self.short_term.pop_front();
        }
        self.short_term.push_back(memory);
    }

    /// 获取所有短期记忆
    pub fn get_short_term(&self) -> Vec<Memory> {
        self.short_term.iter().cloned().collect()
    }

    /// 清空短期记忆
    pub fn clear_short_term(&mut self) {
        self.short_term.clear();
    }

    /// 根据关键词检索记忆
    ///
    /// # Arguments
    /// * `keyword` - 搜索关键词
    /// * `memories` - 长期记忆列表
    ///
    /// # Returns
    /// 匹配的记忆列表
    pub fn search_memories(keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        let keyword_lower = keyword.to_lowercase();
        memories
            .iter()
            .filter(|m| m.content.to_lowercase().contains(&keyword_lower))
            .cloned()
            .collect()
    }

    /// 获取最相关的记忆（按重要性和权重排序）
    ///
    /// # Arguments
    /// * `memories` - 记忆列表
    /// * `limit` - 返回数量限制
    ///
    /// # Returns
    /// 排序后的记忆列表
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

    /// 构建记忆上下文（用于提示词）
    ///
    /// # Arguments
    /// * `memories` - 记忆列表
    /// * `max_tokens` - 最大token数
    ///
    /// # Returns
    /// 记忆上下文
    pub fn build_context(memories: &[Memory], max_tokens: usize) -> MemoryContext {
        let mut context_memories = Vec::new();
        let mut total_tokens = 0;

        for memory in memories {
            let tokens = memory.content.len() / 4; // 粗略估计
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

    /// 更新记忆的重要性
    ///
    /// # Arguments
    /// * `memory` - 记忆对象
    /// * `delta` - 重要性变化量
    ///
    /// # Returns
    /// 更新后的记忆
    pub fn update_importance(mut memory: Memory, delta: f64) -> Memory {
        memory.importance = (memory.importance + delta).clamp(0.0, 1.0);
        memory
    }

    /// 衰减记忆权重（基于时间）
    ///
    /// # Arguments
    /// * `memory` - 记忆对象
    /// * `days_passed` - 经过的天数
    ///
    /// # Returns
    /// 衰减后的记忆
    pub fn decay_weight(mut memory: Memory, days_passed: f64) -> Memory {
        let decay_factor = 0.95_f64.powf(days_passed);
        memory.weight *= decay_factor;
        memory.weight = memory.weight.max(0.1); // 最小权重
        memory
    }

    /// 合并相似记忆
    ///
    /// # Arguments
    /// * `memories` - 记忆列表
    ///
    /// # Returns
    /// 合并后的记忆列表
    pub fn merge_similar_memories(memories: &[Memory]) -> Vec<Memory> {
        if memories.is_empty() {
            return Vec::new();
        }

        let mut merged = Vec::new();
        let mut processed = vec![false; memories.len()];

        for (i, mem_a) in memories.iter().enumerate() {
            if processed[i] {
                continue;
            }

            let mut combined = mem_a.clone();
            processed[i] = true;

            for (j, mem_b) in memories.iter().enumerate().skip(i + 1) {
                if processed[j] {
                    continue;
                }

                if Self::is_similar(&mem_a.content, &mem_b.content) {
                    combined.importance = (combined.importance + mem_b.importance) / 2.0;
                    combined.weight += mem_b.weight;
                    processed[j] = true;
                }
            }

            merged.push(combined);
        }

        merged
    }

    /// 判断两个记忆是否相似（简单的关键词匹配）
    fn is_similar(content_a: &str, content_b: &str) -> bool {
        let words_a: Vec<&str> = content_a.split_whitespace().collect();
        let words_b: Vec<&str> = content_b.split_whitespace().collect();

        let common = words_a.iter().filter(|w| words_b.contains(w)).count();

        let total = words_a.len().max(words_b.len());
        total > 0 && common as f64 / total as f64 > 0.5
    }
}

impl Default for MemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_memory(id: &str, content: &str, importance: f64) -> Memory {
        Memory {
            id: id.to_string(),
            role_id: "test_role".to_string(),
            content: content.to_string(),
            importance,
            weight: 1.0,
            created_at: Utc::now(),
            scene_id: None,
        }
    }

    #[test]
    fn test_add_short_term() {
        let mut engine = MemoryEngine::new();
        let mem = create_test_memory("1", "test content", 0.8);
        engine.add_short_term(mem.clone());

        assert_eq!(engine.get_short_term().len(), 1);
        assert_eq!(engine.get_short_term()[0].id, "1");
    }

    #[test]
    fn test_short_term_capacity() {
        let mut engine = MemoryEngine::new();
        for i in 0..15 {
            let mem = create_test_memory(&i.to_string(), "content", 0.5);
            engine.add_short_term(mem);
        }

        assert_eq!(engine.get_short_term().len(), SHORT_TERM_CAPACITY);
    }

    #[test]
    fn test_search_memories() {
        let memories = vec![
            create_test_memory("1", "用户喜欢咖啡", 0.8),
            create_test_memory("2", "用户讨厌下雨", 0.7),
            create_test_memory("3", "用户爱好编程", 0.9),
        ];

        let results = MemoryEngine::search_memories("用户", &memories);
        assert_eq!(results.len(), 3);

        let results = MemoryEngine::search_memories("咖啡", &memories);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_relevant_memories() {
        let memories = vec![
            create_test_memory("1", "content1", 0.5),
            create_test_memory("2", "content2", 0.9),
            create_test_memory("3", "content3", 0.7),
        ];

        let relevant = MemoryEngine::get_relevant_memories(&memories, 2);
        assert_eq!(relevant.len(), 2);
        assert_eq!(relevant[0].importance, 0.9);
        assert_eq!(relevant[1].importance, 0.7);
    }

    #[test]
    fn test_build_context() {
        let memories = vec![
            create_test_memory("1", "short", 0.8),
            create_test_memory("2", "medium content here", 0.7),
        ];

        let context = MemoryEngine::build_context(&memories, 100);
        assert!(context.total_tokens > 0);
        assert!(!context.memories.is_empty());
    }

    #[test]
    fn test_update_importance() {
        let mem = create_test_memory("1", "content", 0.5);
        let updated = MemoryEngine::update_importance(mem, 0.3);
        assert_eq!(updated.importance, 0.8);
    }

    #[test]
    fn test_update_importance_clamp() {
        let mem = create_test_memory("1", "content", 0.9);
        let updated = MemoryEngine::update_importance(mem, 0.5);
        assert_eq!(updated.importance, 1.0);
    }

    #[test]
    fn test_decay_weight() {
        let mem = create_test_memory("1", "content", 0.8);
        let decayed = MemoryEngine::decay_weight(mem, 10.0);
        assert!(decayed.weight < 1.0);
        assert!(decayed.weight >= 0.1);
    }

    #[test]
    fn test_clear_short_term() {
        let mut engine = MemoryEngine::new();
        let mem = create_test_memory("1", "content", 0.8);
        engine.add_short_term(mem);
        engine.clear_short_term();

        assert_eq!(engine.get_short_term().len(), 0);
    }
}
