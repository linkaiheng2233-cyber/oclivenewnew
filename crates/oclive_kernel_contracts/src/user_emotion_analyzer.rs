//! 用户/文本情绪分析可替换门面 trait。

use oclive_kernel_types::{EmotionResult, Result};

pub trait UserEmotionAnalyzer: Send + Sync {
    /// # Errors
    ///
    /// Returns an error when the analyzer cannot produce an [`EmotionResult`].
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}
