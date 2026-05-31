//! Replaceable facade trait for complex emotion resolution.

use oclive_kernel_types::{ComplexEmotionInput, ComplexEmotionOutput, Result};

/// Resolves complex emotion labels and narrative hints for co-present turns.
///
/// ## When to implement
///
/// - **Who**: complex emotion / `narrative_hint` providers (builtin keyword, Remote, directory plugin).
/// - **When**: when a role needs an **abstract emotional narrative hint** written into the next turn's prompt.
///
/// ## When not to implement
///
/// - Simple roles whose `complex_emotion` slot is `none`, or that do not need a `narrative_hint`.
pub trait ComplexEmotionProvider: Send + Sync {
    /// Resolves this turn's complex emotion labels and narrative hint.
    ///
    /// # Errors
    ///
    /// Propagates [`oclive_kernel_types::AppError`] from the underlying implementation.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput>;
}
