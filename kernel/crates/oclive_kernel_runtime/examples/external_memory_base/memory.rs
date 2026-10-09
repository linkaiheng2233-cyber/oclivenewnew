//! Caller-owned Memory, implemented outside the runtime library.
//!
//! Agreement: materials are ordered oldest to newest by the caller. A query must
//! be `literal:<exact substring>`; matching is case-sensitive, the empty literal
//! matches all materials. Select at most the configured number of latest matches
//! and return their original text in the original order. No summarization, hidden
//! corpus, persistence, ranking or claim that every relevant fact was found.

use std::cell::Cell;

use oclive_kernel_contracts::{BaseCallFuture, MemoryBase};
use oclive_kernel_types::{BaseCallError, BaseCallErrorKind, MemoryBaseRequest};

pub(super) struct RecentLiteralMemory {
    max_results: usize,
    calls: Cell<usize>,
}

impl RecentLiteralMemory {
    pub(super) fn new(max_results: usize) -> Self {
        Self {
            max_results,
            calls: Cell::new(0),
        }
    }

    pub(super) fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl MemoryBase for RecentLiteralMemory {
    fn retrieve<'a>(&'a self, request: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            let literal = request
                .query
                .strip_prefix("literal:")
                .ok_or_else(|| BaseCallError {
                    kind: BaseCallErrorKind::Unsupported,
                    detail: Some("RecentLiteralMemory requires literal:<exact substring>.".into()),
                })?;
            let mut selected: Vec<String> = request
                .materials
                .iter()
                .rev()
                .filter(|text| text.contains(literal))
                .take(self.max_results)
                .map(|text| (*text).to_owned())
                .collect();
            selected.reverse();
            Ok(selected)
        })
    }
}
