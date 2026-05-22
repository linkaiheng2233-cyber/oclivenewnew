//! 用户/文本情绪分析可替换门面 trait。

use oclive_kernel_types::{EmotionResult, Result};

/// Analyzes user text into a seven-dimensional [`EmotionResult`].
pub trait UserEmotionAnalyzer: Send + Sync {
    /// 分析用户文本并产出七维情绪结果。
    ///
    /// # Errors
    ///
    /// Returns an error when the analyzer cannot produce an [`EmotionResult`].
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}
