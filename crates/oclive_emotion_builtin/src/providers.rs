//! `feature = "providers"`：进程内 Builtin / BuiltinV2 `UserEmotionAnalyzer`。

use crate::classic::EmotionAnalyzer;
use oclive_kernel_core::error::Result;
use oclive_kernel_core::models::EmotionResult;
use oclive_kernel_core::user_emotion_analyzer::UserEmotionAnalyzer;

/// 关键词七维分析（与 `EmotionAnalyzer::analyze` 一致）。
pub struct BuiltinUserEmotionAnalyzer;

impl UserEmotionAnalyzer for BuiltinUserEmotionAnalyzer {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        EmotionAnalyzer::analyze(text)
    }
}

/// 任意非空输入均返回纯中性七维分布（与 `BuiltinUserEmotionAnalyzer` 可区分）。
pub struct BuiltinUserEmotionAnalyzerV2;

impl UserEmotionAnalyzer for BuiltinUserEmotionAnalyzerV2 {
    fn analyze(&self, _text: &str) -> Result<EmotionResult> {
        Ok(EmotionResult::strong_neutral())
    }
}
