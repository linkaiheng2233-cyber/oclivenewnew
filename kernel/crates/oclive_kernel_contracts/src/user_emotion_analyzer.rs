//! Replaceable facade trait for user/text emotion analysis.

use oclive_kernel_types::{EmotionResult, Result};

/// Analyzes user text into a seven-dimensional [`EmotionResult`].
///
/// ## When to implement
///
/// - **Who**: emotion analysis backends (builtin keyword / LLM, Remote).
/// - **When**: when the co-present path needs to analyze **user message emotion** to drive events and the prompt.
///
/// ## When not to implement
///
/// - When the default builtin analysis is used and needs no replacement; or when the role disables emotion-related capabilities.
pub trait UserEmotionAnalyzer: Send + Sync {
    /// Analyzes user text and produces a seven-dimensional emotion result.
    ///
    /// # Errors
    ///
    /// Returns an error when the analyzer cannot produce an [`EmotionResult`].
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}
