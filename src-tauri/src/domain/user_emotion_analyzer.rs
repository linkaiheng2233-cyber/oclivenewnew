//! 用户/文本情绪分析可替换门面；默认委托 [`EmotionAnalyzer`](super::emotion_analyzer::EmotionAnalyzer)。

use crate::domain::emotion_analyzer::{EmotionAnalyzer, EmotionResult};
use crate::error::Result;
use std::sync::atomic::{AtomicBool, Ordering};

pub trait UserEmotionAnalyzer: Send + Sync {
    fn analyze(&self, text: &str) -> Result<EmotionResult>;
}

pub struct BuiltinUserEmotionAnalyzer;

impl UserEmotionAnalyzer for BuiltinUserEmotionAnalyzer {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        EmotionAnalyzer::analyze(text)
    }
}

/// 第二套内置：任意非空输入均返回 **纯中性** 七维分布（与 `BuiltinUserEmotionAnalyzer` 可区分，用于验证后端枚举）。
pub struct BuiltinUserEmotionAnalyzerV2;

impl UserEmotionAnalyzer for BuiltinUserEmotionAnalyzerV2 {
    fn analyze(&self, _text: &str) -> Result<EmotionResult> {
        Ok(EmotionResult {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 1.0,
        })
    }
}

pub struct RemoteUserEmotionAnalyzerPlaceholder {
    inner: BuiltinUserEmotionAnalyzer,
    warned: AtomicBool,
}

impl RemoteUserEmotionAnalyzerPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: BuiltinUserEmotionAnalyzer,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::warn!(
                target: "oclive_plugin",
                "emotion backend Remote is not connected; using builtin analyzer"
            );
        }
    }
}

impl UserEmotionAnalyzer for RemoteUserEmotionAnalyzerPlaceholder {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        self.warn_once();
        self.inner.analyze(text)
    }
}

impl Default for RemoteUserEmotionAnalyzerPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::emotion_analyzer::EmotionAnalyzer;
    use crate::models::Emotion;

    #[test]
    fn builtin_v2_neutral_differs_from_builtin_on_clear_joy() {
        let text = "我很开心！";
        let b = BuiltinUserEmotionAnalyzer.analyze(text).unwrap();
        let v2 = BuiltinUserEmotionAnalyzerV2.analyze(text).unwrap();
        assert!(b.joy > 0.2, "builtin should see joy");
        assert!(v2.neutral >= 0.99);
        assert_ne!(
            EmotionAnalyzer::get_dominant_emotion(&b),
            EmotionAnalyzer::get_dominant_emotion(&v2)
        );
        assert_eq!(EmotionAnalyzer::get_dominant_emotion(&v2), Emotion::Neutral);
    }
}
