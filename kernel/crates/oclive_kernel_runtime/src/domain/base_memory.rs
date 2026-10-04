//! B2-C2: a reference [`MemoryBase`] implementation built on this crate's existing literal text
//! matching.
//!
//! [`KeywordMemoryBase`] is a native Base implementation that reuses the same production matching
//! rule as `MemoryEngine::search_memories`: both call the one crate-internal helper in
//! [`super::memory_engine`] that lower-cases the candidate and does a substring test. It is **not**
//! a migration of the old retrieval port — the old `MemoryRetrieval` ranking path stays exactly as
//! it is, and this type does not wrap it.
//!
//! # The matching contract of this implementation
//!
//! - `query` is a **literal substring**, compared after `str::to_lowercase` on both sides. It is
//!   not a natural-language request, a regular expression, a term set, a query language or an
//!   instruction: a query that reads like "exclude coffee" is still matched as those exact
//!   characters.
//! - The query is **not** trimmed, tokenised, stemmed, synonym-expanded or Unicode-normalised.
//!   Matching is Unicode-aware `to_lowercase`, which is not the full Unicode case-folding and does
//!   not make this implementation understand arbitrary natural-language needs.
//! - Only the `materials` supplied in this call are scanned. Nothing is read from a database,
//!   cache, host state or another slot, and nothing is written back.
//! - A match returns the material's **original text**, not a lower-cased copy: case, line breaks,
//!   white space, subject, time, negation and uncertainty are kept as given. Matching a material
//!   does not assert that it is true, that it is about the current speaker, or that it is the only
//!   relevant material.
//! - Results keep the input order and duplicate entries; there is no ranking, scoring, top-k
//!   selection, de-duplication or summarisation.
//! - No match, no materials, or an empty material list all return an empty result. An **empty
//!   query matches every material**, including a material that is itself empty — so selecting
//!   `[""]` is not the same outcome as selecting nothing. This is this implementation's rule under
//!   the shared helper, not a Kernel-wide statement about empty queries.
//!
//! A caller may use this implementation when that limited contract satisfies the current need. It
//! does not claim exhaustive search of real memories, semantic relevance, source filtering, time
//! conditions or natural-language understanding.
//!
//! # Scope
//!
//! The type is stateless, holds no client or configuration, and has **no production Host binding**:
//! no `slot_runner`, `AppState`, database or plugin path constructs it. The independent
//! `minimal_role_host` example explicitly selects it for one six-Base operation. Every call runs the shared
//! helper synchronously and returns a future that is ready immediately; there is no executor,
//! timer, I/O or cancellation source behind it.
//!
//! # Example
//!
//! ```no_run
//! use std::task::{Context, Poll, Waker};
//!
//! use oclive_kernel_contracts::MemoryBase;
//! use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
//! use oclive_kernel_types::MemoryBaseRequest;
//!
//! let materials = ["Alice 不喜欢咖啡", "Bob 喜欢茶"];
//! let request = MemoryBaseRequest {
//!     materials: &materials,
//!     query: "咖啡",
//! };
//! let mut future = KeywordMemoryBase.retrieve(request);
//! let waker = Waker::noop();
//! let mut cx = Context::from_waker(waker);
//! if let Poll::Ready(Ok(hits)) = future.as_mut().poll(&mut cx) {
//!     assert_eq!(hits, vec!["Alice 不喜欢咖啡".to_string()]);
//! }
//! ```

use oclive_kernel_contracts::{BaseCallFuture, MemoryBase};
use oclive_kernel_types::MemoryBaseRequest;

use super::memory_engine::matches_keyword_literal;

/// A reference [`MemoryBase`] implementation that selects materials containing a literal substring.
///
/// See the [module documentation](self) for the exact matching contract, the scope of what it
/// reads, and what it does not promise. The type carries no state, so it can be shared or used as a
/// zero-sized value:
///
/// ```no_run
/// use oclive_kernel_contracts::MemoryBase;
/// use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
///
/// let slot: &dyn MemoryBase = &KeywordMemoryBase;
/// let _ = slot;
/// ```
pub struct KeywordMemoryBase;

impl MemoryBase for KeywordMemoryBase {
    /// Returns the materials whose text contains `request.query` as a literal substring.
    ///
    /// The scan covers only `request.materials`, keeps their order and duplicates, and returns the
    /// original text of each match. An empty query selects every material. There is no error path
    /// in this implementation: it performs no I/O and makes no model call, so it does not fabricate
    /// a failure, cancellation or timeout.
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move {
            let query_lower = request.query.to_lowercase();
            Ok(request
                .materials
                .iter()
                .filter(|material| matches_keyword_literal(material, &query_lower))
                .map(|material| (*material).to_string())
                .collect())
        })
    }
}

#[cfg(test)]
mod b2_c2_tests {
    use std::task::{Context, Poll, Waker};

    use super::*;

    fn collect(materials: &[&str], query: &str) -> Vec<String> {
        let request = MemoryBaseRequest { materials, query };
        let mut future = KeywordMemoryBase.retrieve(request);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(Ok(hits)) => hits,
            Poll::Ready(Err(error)) => panic!("this implementation has no failure path: {error}"),
            Poll::Pending => panic!("this implementation completes on the first poll"),
        }
    }

    #[test]
    fn b2_c2_query_actually_selects() {
        let materials = ["咖啡与牛奶", "雨天记录", "咖啡豆"];
        // Two different queries over the same material set select different subsets ...
        assert_eq!(collect(&materials, "咖啡"), vec!["咖啡与牛奶", "咖啡豆"]);
        assert_eq!(collect(&materials, "雨天"), vec!["雨天记录"]);
        // ... and an unrelated query selects nothing, which rules out a "return everything" shape.
        assert!(collect(&materials, "编程").is_empty());
        assert!(collect(&materials, "咖啡").len() < materials.len());
    }

    #[test]
    fn b2_c2_returns_original_text_verbatim() {
        let materials = [
            "Coffee at 09:00",
            "Alice 不喜欢咖啡，她只是提过",
            "line one\nline two",
            "也许下周搬家（不确定）",
        ];
        // ASCII case-insensitive matching still returns the original casing.
        assert_eq!(collect(&materials, "coffee"), vec!["Coffee at 09:00"]);
        // Negation, subject, time and uncertainty survive byte for byte.
        assert_eq!(
            collect(&materials, "咖啡"),
            vec!["Alice 不喜欢咖啡，她只是提过"]
        );
        assert_eq!(collect(&materials, "LINE ONE"), vec!["line one\nline two"]);
        assert_eq!(
            collect(&materials, "下周搬家"),
            vec!["也许下周搬家（不确定）"]
        );
    }

    #[test]
    fn b2_c2_keeps_order_and_duplicates() {
        let materials = ["b coffee", "a coffee", "b coffee"];
        assert_eq!(
            collect(&materials, "coffee"),
            vec!["b coffee", "a coffee", "b coffee"],
            "input order and duplicate count are preserved; no ranking or de-duplication"
        );
    }

    #[test]
    fn b2_c2_empty_results_are_normal_and_empty_input_is_not_a_failure() {
        assert!(collect(&["a", "b"], "zzz").is_empty());
        let empty: [&str; 0] = [];
        assert!(collect(&empty, "anything").is_empty());
        assert!(collect(&empty, "").is_empty());
    }

    #[test]
    fn b2_c2_empty_query_selects_everything_and_empty_material_is_distinct() {
        let materials = ["", "coffee"];
        assert_eq!(
            collect(&materials, ""),
            vec!["", "coffee"],
            "an empty query selects every material, including the empty one"
        );
        // A list containing one empty string is a different outcome from an empty list.
        assert_eq!(collect(&[""], ""), vec![""]);
        assert!(collect(&empty_list(), "").is_empty());
        // A whitespace-only query is matched literally, not trimmed away.
        assert!(collect(&materials, " ").is_empty());
    }

    fn empty_list() -> [&'static str; 0] {
        []
    }

    #[test]
    fn b2_c2_query_is_literal_text_not_a_pattern_or_instruction() {
        let materials = ["请排除咖啡", "cof fee", "coffee"];
        // A query that looks like an instruction is matched as characters; no instruction runs.
        assert_eq!(collect(&materials, "请排除咖啡"), vec!["请排除咖啡"]);
        // The substring "排除" appears inside "请排除咖啡", so it selects that material: nothing
        // interpreted the word as a filtering command either way.
        assert_eq!(collect(&materials, "排除"), vec!["请排除咖啡"]);
        // Regular-expression punctuation has no special meaning.
        assert_eq!(collect(&materials, "cof fee"), vec!["cof fee"]);
        assert!(collect(&materials, "cof.*fee").is_empty());
        // No tokenising: the spaces are part of the literal.
        assert_eq!(collect(&materials, "coffee"), vec!["coffee"]);
    }

    #[test]
    fn b2_c2_call_scope_does_not_leak_between_calls() {
        let first = ["alpha coffee", "beta tea"];
        assert_eq!(collect(&first, "coffee"), vec!["alpha coffee"]);
        // Second call with different materials must not see the first batch.
        let second = ["gamma tea"];
        assert!(collect(&second, "coffee").is_empty());
        assert_eq!(collect(&second, "tea"), vec!["gamma tea"]);
    }

    #[test]
    fn b2_c2_matches_the_legacy_search_rule_through_the_shared_helper() {
        // Same rule, observed at both call sites: the legacy search keeps full `Memory` values
        // while the Base implementation returns the same texts, in the same order.
        let memories = vec![
            oclive_kernel_types::Memory {
                id: "m1".to_string(),
                role_id: "r".to_string(),
                content: "咖啡与牛奶".to_string(),
                importance: 0.9,
                weight: 1.0,
                created_at: chrono::Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
            oclive_kernel_types::Memory {
                id: "m2".to_string(),
                role_id: "r".to_string(),
                content: "Coffee beans".to_string(),
                importance: 0.1,
                weight: 1.0,
                created_at: chrono::Utc::now(),
                scene_id: None,
                mention_count: 1,
                accessed_at: None,
            },
        ];
        let legacy: Vec<String> =
            super::super::memory_engine::MemoryEngine::search_memories("COFFEE", &memories)
                .into_iter()
                .map(|m| m.content)
                .collect();
        let materials: Vec<&str> = memories.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(collect(&materials, "COFFEE"), legacy);
        assert_eq!(legacy, vec!["Coffee beans".to_string()]);
    }
}
