//! Replaceable facade trait for memory retrieval.

use oclive_kernel_types::{Memory, MemoryContext, MemoryRetrievalInput, Result};

/// Ranks and formats memories for prompt injection.
///
/// ## When to implement
///
/// - **Who**: memory retrieval backends (builtin ranking, local directory plugin, Remote HTTP).
/// - **When**: when a role enables the memory system and needs relevant memories injected into the prompt.
///
/// ## When not to implement
///
/// - When there is no long-term memory, or the builtin `BuiltinMemoryRetrieval` is used as-is and its behavior already meets the requirements.
pub trait MemoryRetrieval: Send + Sync {
    /// Ranks memories by relevance.
    ///
    /// # Errors
    ///
    /// The implementation returns the `Err` variant of [`Result`] when retrieval/ranking fails.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn rank_memories(&self, input: MemoryRetrievalInput<'_>) -> Result<Vec<Memory>>;

    /// Formats a list of memories into context that can be injected into the prompt.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn build_context(&self, memories: &[Memory], max_tokens: usize) -> MemoryContext;

    /// Searches existing memories by keyword.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn search_memories(&self, keyword: &str, memories: &[Memory]) -> Vec<Memory>;

    /// Telemetry hook: local-plugin retrieval may expose the selected `provider_id` (default `None`).
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    #[must_use]
    fn diagnostic_local_provider_id(&self) -> Option<&str> {
        None
    }
}
