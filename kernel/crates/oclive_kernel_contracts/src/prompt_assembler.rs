//! Replaceable facade trait for prompt assembly.

use oclive_kernel_types::{PromptInput, Result, Role};

/// Builds the final LLM prompt string from role, scene, and turn context.
///
/// ## When to implement
///
/// - **Who**: prompt assembly backends (builtin templates, Remote, directory plugin).
/// - **When**: when a **custom** prompt structure is needed (section ordering, conditional blocks, top_topic).
///
/// ## When not to implement
///
/// - When the default `BuiltinPromptAssembler` / `PromptBuilder` already meets the role's needs, no new implementation is required.
pub trait PromptAssembler: Send + Sync {
    /// Assembles this turn's prompt body.
    ///
    /// # Errors
    ///
    /// Returns the `Err` variant of [`Result`] when template or input validation fails.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String>;

    /// Returns the topic hint for the current scene (optional).
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String>;
}
