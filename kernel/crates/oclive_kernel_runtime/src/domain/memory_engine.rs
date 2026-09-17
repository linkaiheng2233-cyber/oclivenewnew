//! Memory engine module.
//! Manages short- and long-term memory; supports retrieval and updates.

use crate::models::{Memory, MemoryContext, RolePackMemoryConfig};
use std::collections::HashSet;
use std::collections::VecDeque;

/// Short-term buffer (keeps at most the last N conversation turns).
const SHORT_TERM_CAPACITY: usize = 10;

/// Memory engine.
pub struct MemoryEngine {
    short_term: VecDeque<Memory>,
}

impl MemoryEngine {
    /// Creates a new memory engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            short_term: VecDeque::with_capacity(SHORT_TERM_CAPACITY),
        }
    }

    /// Adds a short-term memory entry.
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

    /// Selects the memories whose content contains `keyword`, using this crate's literal text
    /// matching rule.
    ///
    /// The keyword is lower-cased once per call and matched without trimming, tokenising or
    /// interpreting it (see the crate-internal `matches_keyword_literal` rule). Matches keep their
    /// input order and duplicates; an empty keyword selects every entry. Callers get full
    /// [`Memory`] values, so this keeps working for paths that need ids and weights.
    #[must_use]
    pub fn search_memories(keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        let keyword_lower = keyword.to_lowercase();
        memories
            .iter()
            .filter(|m| matches_keyword_literal(&m.content, &keyword_lower))
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

    /// Ebbinghaus exponential decay: remaining strength = initial × e^(-λ × virtual days).
    /// Effective half-life grows with `mention_count` (reinforcement on review).
    pub fn apply_time_decay(memory: &mut Memory, virtual_days: f64, cfg: &RolePackMemoryConfig) {
        if virtual_days <= 0.0 {
            return;
        }
        let base_halflife = cfg.decay_halflife_days.max(0.1);
        let mentions = f64::from(memory.mention_count.max(1));
        let effective_halflife =
            base_halflife * (1.0 + cfg.reinforcement_factor * (mentions - 1.0));
        let lambda = std::f64::consts::LN_2 / effective_halflife;
        memory.weight *= (-lambda * virtual_days).exp();
        memory.weight = memory.weight.max(0.0);
    }

    /// Decays a batch by virtual-clock age and drops entries below the prompt threshold.
    pub fn apply_time_decay_batch(
        memories: &mut Vec<Memory>,
        virtual_now_ms: i64,
        cfg: &RolePackMemoryConfig,
    ) {
        use super::virtual_time::virtual_days_between_ms;
        Self::decay_memories_in_place(
            memories,
            |m| {
                let created_ms = m.created_at.timestamp_millis();
                virtual_days_between_ms(created_ms, virtual_now_ms)
            },
            cfg,
        );
        memories.retain(|m| m.effective_strength() >= cfg.min_strength_for_prompt);
    }

    /// Applies decay in place without filtering (for persistence before threshold filter).
    pub fn decay_memories_in_place(
        memories: &mut [Memory],
        virtual_days_for: impl Fn(&Memory) -> f64,
        cfg: &RolePackMemoryConfig,
    ) {
        for m in memories.iter_mut() {
            let days = virtual_days_for(m);
            Self::apply_time_decay(m, days, cfg);
        }
    }

    /// Wall-clock decay using `accessed_at` (or `created_at`) as the reference instant.
    pub fn apply_wall_clock_decay_batch(
        memories: &mut Vec<Memory>,
        now_ms: i64,
        cfg: &RolePackMemoryConfig,
    ) {
        use super::virtual_time::virtual_days_between_ms;
        Self::decay_memories_in_place(
            memories,
            |m| {
                let ref_ms = m.accessed_at.unwrap_or(m.created_at).timestamp_millis();
                virtual_days_between_ms(ref_ms, now_ms)
            },
            cfg,
        );
        memories.retain(|m| m.effective_strength() >= cfg.min_strength_for_prompt);
    }

    #[must_use]
    pub fn filter_for_prompt_threshold(
        memories: Vec<Memory>,
        cfg: &RolePackMemoryConfig,
    ) -> Vec<Memory> {
        memories
            .into_iter()
            .filter(|m| m.effective_strength() >= cfg.min_strength_for_prompt)
            .collect()
    }

    /// Extracts keywords for similarity (whitespace/punctuation tokens + CJK bigram fragments).
    fn keyword_tokens(text: &str) -> HashSet<String> {
        let lower = text.to_lowercase();
        let mut set = HashSet::new();
        for word in lower.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
            if word.chars().count() >= 2 {
                set.insert(word.to_string());
            }
        }
        let chars: Vec<char> = lower
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
            .collect();
        if chars.len() >= 2 {
            for pair in chars.windows(2) {
                set.insert(pair.iter().collect());
            }
        }
        set
    }

    /// Keyword Jaccard overlap \[0, 1\] for both sides; CJK text also weights 4-character contiguous chunks.
    #[must_use]
    pub fn keyword_overlap_similarity(content_a: &str, content_b: &str) -> f64 {
        let clean = |s: &str| {
            s.to_lowercase()
                .chars()
                .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
                .collect::<String>()
        };
        let sa = clean(content_a);
        let sb = clean(content_b);
        if !sa.is_empty() && !sb.is_empty() {
            let (short, long) = if sa.len() <= sb.len() {
                (&sa, &sb)
            } else {
                (&sb, &sa)
            };
            if short.len() >= 6 && long.contains(short.as_str()) {
                return 1.0;
            }
        }
        let a = Self::keyword_tokens(content_a);
        let b = Self::keyword_tokens(content_b);
        let jaccard = if a.is_empty() || b.is_empty() {
            0.0
        } else {
            let inter = a.intersection(&b).count() as f64;
            let union = a.union(&b).count() as f64;
            inter / union
        };
        jaccard.max(Self::shared_cjk_chunk_score(content_a, content_b))
    }

    fn shared_cjk_chunk_score(a: &str, b: &str) -> f64 {
        let clean = |s: &str| {
            s.to_lowercase()
                .chars()
                .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
                .collect::<String>()
        };
        let sa = clean(a);
        let sb = clean(b);
        if sa.len() < 4 || sb.len() < 4 {
            return 0.0;
        }
        let (short, long) = if sa.len() <= sb.len() {
            (&sa, &sb)
        } else {
            (&sb, &sa)
        };
        let chars: Vec<char> = short.chars().collect();
        let windows = chars.len().saturating_sub(3);
        if windows == 0 {
            return 0.0;
        }
        let hits = chars
            .windows(4)
            .filter(|w| long.contains(&w.iter().collect::<String>()))
            .count();
        if hits == 0 {
            0.0
        } else {
            (hits as f64 / windows as f64).clamp(0.0, 1.0)
        }
    }

    /// Decays memory weight (legacy API kept for compatibility tests).
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

/// The literal text-matching rule this crate's memory search uses.
///
/// `keyword` is expected already lower-cased by the caller; each candidate is lower-cased here and
/// compared with [`str::contains`]. The rule is a plain substring search:
///
/// - it does not trim, tokenise, stem, normalise Unicode, expand synonyms or interpret the keyword
///   as an instruction, a pattern or a query language;
/// - the keyword is matched against the whole candidate, so an empty keyword selects every
///   candidate, including empty ones;
/// - matches keep their input order, their duplicates and their original text.
///
/// It is the single implementation of that rule: the legacy [`MemoryEngine::search_memories`] and
/// the Base reference implementation in [`super::base_memory`] both call it, so neither can drift
/// apart from the other.
pub(super) fn matches_keyword_literal(candidate: &str, keyword_lower: &str) -> bool {
    candidate.to_lowercase().contains(keyword_lower)
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
            accessed_at: None,
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

    /// B2-C2: the shared literal-matching helper is the rule the legacy search actually uses.
    ///
    /// The helper takes an already lower-cased keyword by contract, so callers lower-case once (the
    /// legacy search does that in [`MemoryEngine::search_memories`]). The expectation comes from
    /// spelling out the documented rule for chosen fixtures, not from calling the helper twice.
    #[test]
    fn b2_c2_literal_matcher_is_a_plain_lowercased_substring_test() {
        assert!(matches_keyword_literal("Alice 不喜欢咖啡", "咖啡"));
        assert!(matches_keyword_literal("Coffee", "coffee"));
        // Callers lower-case first, so an upper-cased query reaches the helper lower-cased.
        assert!(matches_keyword_literal("coffee", &"COFFEE".to_lowercase()));
        assert!(!matches_keyword_literal("Coffee", "tea"));
        // Empty keyword: the documented rule selects everything, including empty text.
        assert!(matches_keyword_literal("", ""));
        assert!(matches_keyword_literal("anything", ""));
        // No trimming and no tokenising: the keyword is compared as written.
        assert!(!matches_keyword_literal("coffee", " coffee"));
        assert!(!matches_keyword_literal("coffee", "cof fee"));
        // A term that looks like an instruction is still a literal substring.
        assert!(!matches_keyword_literal("咖啡", "请排除咖啡"));
    }

    /// B2-C2: the legacy search keeps its signature, its full `Memory` values, its input order, its
    /// duplicates and its empty-keyword behaviour after the extraction.
    #[test]
    fn b2_c2_legacy_search_keeps_order_duplicates_and_fields() {
        let memories = vec![
            create_test_memory("second", "Coffee again", 0.9),
            create_test_memory("first", "咖啡与牛奶", 0.1),
            create_test_memory("dup", "Coffee again", 0.2),
        ];

        let hits = MemoryEngine::search_memories("coffee", &memories);
        assert_eq!(
            hits.len(),
            2,
            "case-insensitive literal match, duplicates kept"
        );
        assert_eq!(hits[0].id, "second");
        assert_eq!(hits[1].id, "dup");
        // fields survive the projection to full Memory values
        assert_eq!(hits[0].content, "Coffee again");
        assert_eq!(hits[0].importance, 0.9);
        assert_eq!(hits[0].role_id, "test_role");

        let all = MemoryEngine::search_memories("", &memories);
        assert_eq!(
            all.len(),
            memories.len(),
            "empty keyword selects every entry"
        );

        let none = MemoryEngine::search_memories("unrelated", &memories);
        assert!(none.is_empty());

        let empty_input: Vec<Memory> = Vec::new();
        assert!(MemoryEngine::search_memories("", &empty_input).is_empty());
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
        let mut mem = create_test_memory("1", "content", 1.0);
        MemoryEngine::apply_time_decay(&mut mem, 7.0, &cfg);
        assert!((mem.weight - 0.5).abs() < 0.05, "weight={}", mem.weight);
    }

    #[test]
    fn reinforced_memory_decays_slower() {
        let cfg = default_mem_cfg();
        let mut once = create_test_memory("1", "用户喜欢冒险旅行", 1.0);
        let mut thrice = create_test_memory("2", "用户喜欢冒险旅行", 1.0);
        thrice.mention_count = 3;
        MemoryEngine::apply_time_decay(&mut once, 14.0, &cfg);
        MemoryEngine::apply_time_decay(&mut thrice, 14.0, &cfg);
        assert!(
            thrice.weight > once.weight,
            "once={} thrice={}",
            once.weight,
            thrice.weight
        );
    }

    #[test]
    fn keyword_overlap_detects_similar_topics() {
        let sim =
            MemoryEngine::keyword_overlap_similarity("用户谈到冒险旅行", "他又谈到冒险旅行计划");
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
