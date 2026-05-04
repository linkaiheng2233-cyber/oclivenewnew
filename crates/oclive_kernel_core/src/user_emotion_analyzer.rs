//! 用户/文本情绪分析可替换门面（实现留在 runtime）。

use crate::error::Result;
use crate::models::EmotionResult;

pub trait UserEmotionAnalyzer: Send + Sync {
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}
