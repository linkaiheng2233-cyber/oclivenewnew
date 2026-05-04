//! 情绪分析门面：七维算法在 [`oclive_emotion_builtin::classic`]。

use crate::models::Emotion;
pub use oclive_emotion_builtin::classic::EmotionAnalyzer;
pub use oclive_kernel_core::models::EmotionResult;

/// 将七维结果映射为离散 `Emotion`（扩展 trait，避免在 `oclive_kernel_core` 再依赖分析器）。
pub trait EmotionResultExt {
    fn to_emotion(&self) -> Emotion;
}

impl EmotionResultExt for EmotionResult {
    fn to_emotion(&self) -> Emotion {
        EmotionAnalyzer::get_dominant_emotion(self)
    }
}
