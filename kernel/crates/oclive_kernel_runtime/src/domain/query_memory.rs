//! CP-B3-ALL unit M: the builtin Memory backend that answers the **current retrieval need**.
//!
//! [`QueryMemoryRetrieval`] implements both the Host port ([`MemoryRetrieval`]) and the Base view
//! ([`MemoryBase`]) on top of **one** selection core (`select_indices`), which works on the texts
//! of this call and returns indices into the original entries. Every projection that follows keeps
//! the original entries — the Host view clones the matched [`Memory`] rows (id, weight, time,
//! scene, mention count and `accessed_at` all preserved) and the Base view returns the original
//! text of the selected materials, never a rewritten copy.
//!
//! # Public surface (CP-B3-ALL-R1)
//!
//! This module's public surface is deliberately limited to the module itself,
//! [`QueryMemoryRetrieval`] and the trait implementations that make it usable. The selection core
//! and its term helper are **private implementation details** (`select_indices`, `query_terms`),
//! not published algorithm functions: they were briefly public in the first pass of this unit and
//! have been taken back, so no external caller can depend on them. The contract this implementation
//! offers is the behaviour of the two trait entries below; the internal boundaries of the core are
//! covered by this module's own unit tests.
//!
//! # The finite rule this implementation actually supports
//!
//! The rule is *mechanical keyword relevance*, not natural-language requirement execution, and it
//! is deliberately documented with its failure modes:
//!
//! 1. The query and the matching material are Unicode-lowercased before comparison. The **result**
//!    text is never rewritten; no NFC normalisation, tokenizer, embedding, model, synonym table or
//!    semantic inference is involved.
//! 2. The query is split into contiguous segments on every non-alphanumeric Unicode character.
//!    A segment that is pure ASCII is used whole as one term. A segment containing any non-ASCII
//!    scalar becomes its adjacent **scalar bigrams**, and a single-scalar segment stays itself.
//!    Duplicate terms are dropped (first occurrence wins). Multi-byte characters are only ever
//!    handled as scalars, never split by bytes.
//! 3. A material is selected when at least one term is a substring of that material's lowercased
//!    text. Each material can be selected at most once, and the selected indices keep the original
//!    input order. Two entries with identical text remain two entries.
//! 4. A query whose `trim()` is empty selects **every** explicit material. A non-empty query with
//!    no usable term, or with terms that match nothing, is a normal empty result. Negation,
//!    subject and condition inside the query are text only: this implementation promises no
//!    semantic understanding, and it never deletes negation or subject information from the
//!    material it returns.
//! 5. Host view only: when at least one row matched, the matched rows alone are handed to
//!    [`MemoryEngine::get_relevant_memories`] for the existing weighted ordering and `limit`. When
//!    nothing matched (including an empty candidate set), the previous all-candidate weighted
//!    selection is used as an explicit **Host compatibility fallback**. That fallback is not a Base
//!    retrieval hit and is never reported as one.
//!
//! # What this does not do
//!
//! It reads no database, cache, host state or other slot, and writes nothing back: no DB write, no
//! `accessed_at` update, no model call, no extra work of any kind. `input.scene_id` is carried by
//! the Host input exactly as before, but this implementation does not use it for selection — no new
//! scene isolation is claimed. The previous builtin
//! ([`BuiltinMemoryRetrieval`](super::memory_retrieval::BuiltinMemoryRetrieval)), the Base
//! reference (`KeywordMemoryBase`) and [`MemoryEngine`] itself are unchanged by this unit, and the
//! old `build_context` / `search_memories` behaviour is delegated to [`MemoryEngine`] as before.
//!
//! Known limits, stated rather than hidden: a term that is a common substring can select an
//! unrelated row (false positive), and a paraphrase that shares no term selects nothing (false
//! negative). This is not complete semantic relevance.

use oclive_kernel_contracts::{BaseCallFuture, MemoryBase, MemoryRetrieval};
use oclive_kernel_types::{MemoryBaseRequest, MemoryContext, MemoryRetrievalInput};

use super::memory_engine::MemoryEngine;
use crate::error::Result;
use crate::models::Memory;

/// Appends `term` when it is not present yet, keeping the first occurrence.
fn push_unique(terms: &mut Vec<String>, term: String) {
    if !terms.iter().any(|existing| existing == &term) {
        terms.push(term);
    }
}

/// The retrieval terms of `query` under this implementation's finite rule (see the module docs).
///
/// Private on purpose (CP-B3-ALL-R1): the finite rule is a property of this implementation, not a
/// published algorithm function.
#[must_use]
fn query_terms(query: &str) -> Vec<String> {
    let lowered = query.to_lowercase();
    let mut terms: Vec<String> = Vec::new();
    for segment in lowered.split(|c: char| !c.is_alphanumeric()) {
        if segment.is_empty() {
            continue;
        }
        if segment.is_ascii() {
            push_unique(&mut terms, segment.to_string());
            continue;
        }
        let scalars: Vec<char> = segment.chars().collect();
        if scalars.len() == 1 {
            push_unique(&mut terms, scalars[0].to_string());
            continue;
        }
        for window in scalars.windows(2) {
            push_unique(&mut terms, window.iter().collect::<String>());
        }
    }
    terms
}

/// **The one selection core**: indices of the selected entries, in original order.
///
/// It reads the supplied texts and the query, and nothing else. An empty (`trim`-empty) query
/// selects every entry; a query with no usable term selects none.
///
/// Private on purpose (CP-B3-ALL-R1): both trait entries below call it, and only this module's unit
/// tests observe it directly.
#[must_use]
fn select_indices(materials: &[&str], query: &str) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..materials.len()).collect();
    }
    let terms = query_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }
    materials
        .iter()
        .enumerate()
        .filter(|(_, material)| {
            let lowered = material.to_lowercase();
            terms.iter().any(|term| lowered.contains(term.as_str()))
        })
        .map(|(index, _)| index)
        .collect()
}

/// The builtin memory backend selected for `plugin_backends.memory = builtin`.
///
/// See the module documentation for the exact selection rule, its known failure modes, the Host
/// compatibility fallback and the facts this type deliberately keeps intact.
pub struct QueryMemoryRetrieval;

impl Default for QueryMemoryRetrieval {
    fn default() -> Self {
        Self
    }
}

impl MemoryRetrieval for QueryMemoryRetrieval {
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>> {
        let contents: Vec<&str> = input
            .memories
            .iter()
            .map(|memory| memory.content.as_str())
            .collect();
        let selected = select_indices(&contents, input.user_query);
        if selected.is_empty() {
            // Documented Host compatibility fallback: the previous all-candidate weighted
            // selection. This is not a Base retrieval hit.
            return Ok(MemoryEngine::get_relevant_memories(
                input.memories,
                input.limit,
            ));
        }
        let matched: Vec<Memory> = selected
            .iter()
            .map(|&index| input.memories[index].clone())
            .collect();
        // Only the matched rows take part in the existing weighted ordering and limit.
        Ok(MemoryEngine::get_relevant_memories(&matched, input.limit))
    }

    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext {
        MemoryEngine::build_context(memories, max_tokens)
    }

    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory> {
        MemoryEngine::search_memories(keyword, memories)
    }
}

impl MemoryBase for QueryMemoryRetrieval {
    /// Returns the original text of the entries selected for `request.query`.
    ///
    /// This implementation has no failure path: it performs no I/O and makes no model call, so it
    /// never fabricates a failure, a cancellation or a timeout. An empty vector means this
    /// implementation's rule selected nothing for this call (or that there was no usable term).
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move {
            let selected = select_indices(request.materials, request.query);
            Ok(selected
                .into_iter()
                .map(|index| request.materials[index].to_string())
                .collect())
        })
    }
}

#[cfg(test)]
mod cp_b3_all_memory_tests {
    use std::task::{Context, Poll, Waker};

    use super::*;
    use crate::models::Memory;
    use chrono::Utc;

    fn drive(materials: &[&str], query: &str) -> Vec<String> {
        let mut future = QueryMemoryRetrieval.retrieve(MemoryBaseRequest { materials, query });
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(selected)) => selected,
            Poll::Ready(Err(error)) => panic!("this implementation has no failure path: {error}"),
            Poll::Pending => panic!("this implementation completes on the first poll"),
        }
    }

    fn memory(id: &str, content: &str, importance: f64) -> Memory {
        Memory {
            id: id.to_string(),
            role_id: "role".to_string(),
            content: content.to_string(),
            importance,
            weight: 1.0,
            created_at: Utc::now(),
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        }
    }

    // The finite rule's terms, written by hand: ASCII keeps whole words, non-ASCII goes by bigrams.
    #[test]
    fn cp_b3_all_memory_query_terms_follow_the_finite_rule() {
        assert_eq!(query_terms("coffee"), vec!["coffee"]);
        assert_eq!(query_terms("COFFEE"), vec!["coffee"]);
        assert_eq!(query_terms("coffee 雨天"), vec!["coffee", "雨天"]);
        let terms = query_terms("我喜欢咖啡");
        assert_eq!(terms, vec!["我喜", "喜欢", "欢咖", "咖啡"]);
        assert_eq!(query_terms("咖"), vec!["咖"]);
        // Duplicates collapse, punctuation only splits.
        assert_eq!(query_terms("ab ab"), vec!["ab"]);
        assert!(query_terms("，。； ").is_empty());
        assert!(query_terms("   ").is_empty());
    }

    // Base selection: Chinese, ASCII case, mixed script, multi-byte boundaries, empties, no hits,
    // duplicates and preserved negation/subject text.
    #[test]
    fn cp_b3_all_memory_base_selection_is_mechanical_and_order_preserving() {
        let materials = [
            "用户喜欢咖啡",
            "User likes COFFEE",
            "他说“我不喜欢咖啡”，我没有这么说",
            "今天星期三",
            "用户喜欢咖啡",
        ];
        // A two-scalar query is a single bigram, and it selects exactly the entries containing it.
        assert_eq!(query_terms("咖啡"), vec!["咖啡"]);
        assert_eq!(
            drive(&materials, "咖啡"),
            vec![
                "用户喜欢咖啡",
                "他说“我不喜欢咖啡”，我没有这么说",
                "用户喜欢咖啡"
            ]
        );
        let hit = drive(&materials, "喜欢咖啡");
        assert_eq!(
            hit,
            vec![
                "用户喜欢咖啡",
                "他说“我不喜欢咖啡”，我没有这么说",
                "用户喜欢咖啡"
            ]
        );
        // ASCII is case-insensitive on both sides.
        assert_eq!(drive(&materials, "Coffee"), vec!["User likes COFFEE"]);
        // Mixed script: the ASCII part and the CJK bigram each match their own material.
        assert_eq!(
            drive(&materials, "coffee 咖啡"),
            vec![
                "用户喜欢咖啡",
                "User likes COFFEE",
                "他说“我不喜欢咖啡”，我没有这么说",
                "用户喜欢咖啡"
            ]
        );
        // Empty and whitespace-only queries select every explicit material.
        assert_eq!(drive(&materials, "").len(), materials.len());
        assert_eq!(drive(&materials, "   ").len(), materials.len());
        // Punctuation only: no usable term, therefore a normal empty result.
        assert!(drive(&materials, "，。").is_empty());
        // No hit is a normal empty result, not a failure.
        assert!(drive(&materials, "完全不相关词").is_empty());
        // Multi-byte boundary: a single scalar query stays one term.
        assert_eq!(
            drive(&materials, "咖"),
            vec![
                "用户喜欢咖啡",
                "他说“我不喜欢咖啡”，我没有这么说",
                "用户喜欢咖啡"
            ]
        );
        let empty: [&str; 0] = [];
        assert!(drive(&empty, "咖啡").is_empty());
    }

    // The selection core returns original indices, so callers can keep their own structured facts.
    #[test]
    fn cp_b3_all_memory_selection_returns_original_indices() {
        let materials = ["alpha", "beta", "Alpha again"];
        assert_eq!(select_indices(&materials, "alpha"), vec![0, 2]);
        assert_eq!(select_indices(&materials, ""), vec![0, 1, 2]);
        assert!(select_indices(&materials, "gamma").is_empty());
    }

    // Host view: a matched low-weight row beats a non-matching high-weight row (the approved
    // change), while the matched set still keeps the existing weighted ordering and limit.
    #[test]
    fn cp_b3_all_memory_host_view_filters_then_keeps_weighted_order() {
        let memories = [
            memory("high-unmatched", "今天星期三", 1.0),
            memory("low-matched", "用户喜欢咖啡", 0.2),
            memory("mid-matched", "用户喜欢咖啡和茶", 0.6),
        ];
        let ranked = QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &memories,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            })
            .expect("rank");
        let ids: Vec<&str> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["mid-matched", "low-matched"]);
        assert!(
            !ids.contains(&"high-unmatched"),
            "a non-matching high-weight row must not survive a query hit"
        );

        // limit is applied after the weighted order.
        let limited = QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &memories,
                user_query: "咖啡",
                scene_id: None,
                limit: 1,
            })
            .expect("rank");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "mid-matched");

        // limit = 0 selects nothing, whatever matched.
        let none = QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &memories,
                user_query: "咖啡",
                scene_id: None,
                limit: 0,
            })
            .expect("rank");
        assert!(none.is_empty());
    }

    // Host compatibility fallback: with no hit (or no candidate), the previous all-candidate
    // weighted selection is used — and that is not a Base hit.
    #[test]
    fn cp_b3_all_memory_host_fallback_is_the_old_weighted_selection() {
        let memories = [
            memory("high-unmatched", "今天星期三", 1.0),
            memory("low-unmatched", "今天星期四", 0.2),
        ];
        let ranked = QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &memories,
                user_query: "完全不相关词",
                scene_id: None,
                limit: 8,
            })
            .expect("rank");
        let ids: Vec<&str> = ranked.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["high-unmatched", "low-unmatched"]);
        // The Base view of the same query is empty: the fallback never becomes a Base selection.
        assert!(drive(&["今天星期三", "今天星期四"], "完全不相关词").is_empty());

        // Empty candidates stay empty in both views.
        let empty: [Memory; 0] = [];
        assert!(QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &empty,
                user_query: "咖啡",
                scene_id: None,
                limit: 8,
            })
            .expect("rank")
            .is_empty());
    }

    // Structured facts survive the projection, and the call mutates nothing.
    #[test]
    fn cp_b3_all_memory_host_view_preserves_originals_and_writes_nothing() {
        let mut kept = memory("kept", "用户喜欢咖啡", 0.4);
        kept.scene_id = Some("scene-1".to_string());
        kept.mention_count = 7;
        let before = kept.clone();
        let memories = [kept, memory("other", "今天星期三", 0.9)];
        let ranked = QueryMemoryRetrieval
            .rank_memories(MemoryRetrievalInput {
                memories: &memories,
                user_query: "咖啡",
                scene_id: Some("scene-1"),
                limit: 8,
            })
            .expect("rank");
        assert_eq!(ranked.len(), 1);
        let selected = &ranked[0];
        assert_eq!(selected.id, before.id);
        assert_eq!(selected.content, before.content);
        assert_eq!(selected.importance, before.importance);
        assert_eq!(selected.weight, before.weight);
        assert_eq!(selected.created_at, before.created_at);
        assert_eq!(selected.scene_id, before.scene_id);
        assert_eq!(selected.mention_count, before.mention_count);
        assert_eq!(selected.accessed_at, before.accessed_at);
        // The caller's slice is untouched: no write-back, no accessed_at update.
        assert_eq!(memories[0].accessed_at, None);
        assert_eq!(memories[0].mention_count, before.mention_count);
    }

    // The legacy helpers keep delegating to the unchanged engine.
    #[test]
    fn cp_b3_all_memory_legacy_helpers_delegate_unchanged() {
        let memories = [memory("a", "alpha", 0.9), memory("b", "beta", 0.2)];
        let context = QueryMemoryRetrieval.build_context(&memories, 512);
        assert_eq!(
            context.memories.len(),
            MemoryEngine::build_context(&memories, 512).memories.len()
        );
        assert_eq!(
            QueryMemoryRetrieval.search_memories("alp", &memories).len(),
            MemoryEngine::search_memories("alp", &memories).len()
        );
    }

    // The Host factory test lives with the Host registry (that is where the pure factory is), so
    // this module only asserts the selector's own behaviour.
}
