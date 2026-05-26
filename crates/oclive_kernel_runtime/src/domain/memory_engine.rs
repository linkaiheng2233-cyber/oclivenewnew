//! 记忆引擎模块
//! 管理短期和长期记忆，支持记忆检索和更新

use crate::models::{Memory, MemoryContext, RolePackMemoryConfig};
use std::collections::HashSet;
use std::collections::VecDeque;

/// 短期记忆缓冲区（最多保留最近N条对话）
const SHORT_TERM_CAPACITY: usize = 10;

/// 记忆引擎
pub struct MemoryEngine {
    short_term: VecDeque<Memory>,
}

impl MemoryEngine {
    /// 创建新的记忆引擎
    #[must_use]
    pub fn new() -> Self {
        Self {
            short_term: VecDeque::with_capacity(SHORT_TERM_CAPACITY),
        }
    }

    /// 添加短期记忆
    pub fn add_short_term(&mut self, memory: Memory) {
        if self.short_term.len() >= SHORT_TERM_CAPACITY {
            self.short_term.pop_front();
        }
        self.short_term.push_back(memory);
    }

    #[must_use]
    pub fn get_short_term(&self) -> Vec<Memory> {
        self.short_term.iter().cloned().collect()
    }

    pub fn clear_short_term(&mut self) {
        self.short_term.clear();
    }

    #[must_use]
    pub fn search_memories(keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        let keyword_lower = keyword.to_lowercase();
        memories
            .iter()
            .filter(|m| m.content.to_lowercase().contains(&keyword_lower))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn get_relevant_memories(memories: &[Memory], limit: usize) -> Vec<Memory> {
        let mut sorted = memories.to_vec();
        sorted.sort_by(|a, b| {
            let score_a = a.effective_strength();
            let score_b = b.effective_strength();
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(limit).collect()
    }

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

    #[must_use]
    pub fn update_importance(mut memory: Memory, delta: f64) -> Memory {
        memory.importance = (memory.importance + delta).clamp(0.0, 1.0);
        memory
    }

    /// 艾宾浩斯指数衰减：剩余强度 = 初始强度 × e^(-λ × 虚拟天数)。
    /// 有效半衰期随 `mention_count` 延长（复习强化）。
    #[must_use]
    pub fn apply_time_decay(
        mut memory: Memory,
        virtual_days: f64,
        cfg: &RolePackMemoryConfig,
    ) -> Memory {
        if virtual_days <= 0.0 {
            return memory;
        }
        let base_halflife = cfg.decay_halflife_days.max(0.1);
        let mentions = f64::from(memory.mention_count.max(1));
        let effective_halflife =
            base_halflife * (1.0 + cfg.reinforcement_factor * (mentions - 1.0));
        let lambda = std::f64::consts::LN_2 / effective_halflife;
        memory.weight *= (-lambda * virtual_days).exp();
        memory.weight = memory.weight.max(0.0);
        memory
    }

    /// 对一批记忆按虚拟时钟年龄衰减，并剔除低于 Prompt 阈值的条目。
    pub fn apply_time_decay_batch(
        memories: &mut Vec<Memory>,
        virtual_now_ms: i64,
        cfg: &RolePackMemoryConfig,
    ) {
        use super::virtual_time::virtual_days_between_ms;
        memories.retain_mut(|m| {
            let created_ms = m.created_at.timestamp_millis();
            let days = virtual_days_between_ms(created_ms, virtual_now_ms);
            *m = Self::apply_time_decay(m.clone(), days, cfg);
            m.effective_strength() >= cfg.min_strength_for_prompt
        });
    }

    /// 提取用于相似度比较的关键词（去空白、小写、长度≥2）。
    fn keyword_tokens(text: &str) -> HashSet<String> {
        text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|w| w.chars().count() >= 2)
            .map(|w| w.to_lowercase())
            .collect()
    }

    /// 双方关键词 Jaccard 重叠度 \[0, 1\]。
    #[must_use]
    pub fn keyword_overlap_similarity(content_a: &str, content_b: &str) -> f64 {
        let a = Self::keyword_tokens(content_a);
        let b = Self::keyword_tokens(content_b);
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        let inter = a.intersection(&b).count() as f64;
        let union = a.union(&b).count() as f64;
        if union <= 0.0 {
            0.0
        } else {
            inter / union
        }
    }

    /// 衰减记忆权重（旧 API，保留兼容测试）。
    #[must_use]
    pub fn decay_weight(mut memory: Memory, days_passed: f64) -> Memory {
        let decay_factor = 0.95_f64.powf(days_passed);
        memory.weight *= decay_factor;
        memory.weight = memory.weight.max(0.1);
        memory
    }

    #[must_use]
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

                if Self::keyword_overlap_similarity(&mem_a.content, &mem_b.content) > 0.5 {
                    combined.importance = (combined.importance + mem_b.importance) / 2.0;
                    combined.weight += mem_b.weight;
                    combined.mention_count += mem_b.mention_count.max(1);
                    processed[j] = true;
                }
            }

            merged.push(combined);
        }

        merged
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
            mention_count: 1,
        }
    }

    fn default_mem_cfg() -> RolePackMemoryConfig {
        RolePackMemoryConfig::default()
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
    fn ebbinghaus_halflife_about_half_after_seven_virtual_days() {
        let cfg = default_mem_cfg();
        let mem = create_test_memory("1", "content", 1.0);
        let decayed = MemoryEngine::apply_time_decay(mem, 7.0, &cfg);
        assert!(
            (decayed.weight - 0.5).abs() < 0.05,
            "weight={}",
            decayed.weight
        );
    }

    #[test]
    fn reinforced_memory_decays_slower() {
        let cfg = default_mem_cfg();
        let mut once = create_test_memory("1", "用户喜欢冒险旅行", 1.0);
        let mut thrice = create_test_memory("2", "用户喜欢冒险旅行", 1.0);
        thrice.mention_count = 3;
        once = MemoryEngine::apply_time_decay(once, 14.0, &cfg);
        thrice = MemoryEngine::apply_time_decay(thrice, 14.0, &cfg);
        assert!(
            thrice.weight > once.weight,
            "once={} thrice={}",
            once.weight,
            thrice.weight
        );
    }

    #[test]
    fn keyword_overlap_detects_similar_topics() {
        let sim = MemoryEngine::keyword_overlap_similarity(
            "用户很喜欢冒险和旅行",
            "他又提起冒险旅行计划",
        );
        assert!(sim >= 0.6, "sim={sim}");
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
