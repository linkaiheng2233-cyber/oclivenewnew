//! 用户/文本情绪分析可替换门面；默认委托 [`EmotionAnalyzer`](super::emotion_analyzer::EmotionAnalyzer)。

use crate::domain::emotion_analyzer::EmotionResult;
#[cfg(feature = "default-emotion-providers")]
pub use oclive_emotion_builtin::{BuiltinUserEmotionAnalyzer, BuiltinUserEmotionAnalyzerV2};
#[cfg(not(feature = "default-emotion-providers"))]
use crate::domain::disabled_default_providers::DisabledUserEmotionAnalyzer;
use crate::error::Result;
pub use oclive_kernel_core::user_emotion_analyzer::UserEmotionAnalyzer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[must_use]
pub fn default_user_emotion_slot_v1() -> Arc<dyn UserEmotionAnalyzer> {
    #[cfg(feature = "default-emotion-providers")]
    {
        Arc::new(BuiltinUserEmotionAnalyzer)
    }
    #[cfg(not(feature = "default-emotion-providers"))]
    {
        Arc::new(DisabledUserEmotionAnalyzer)
    }
}

#[must_use]
pub fn default_user_emotion_slot_v2() -> Arc<dyn UserEmotionAnalyzer> {
    #[cfg(feature = "default-emotion-providers")]
    {
        Arc::new(BuiltinUserEmotionAnalyzerV2)
    }
    #[cfg(not(feature = "default-emotion-providers"))]
    {
        Arc::new(DisabledUserEmotionAnalyzer)
    }
}

pub struct RemoteUserEmotionAnalyzerPlaceholder {
    inner: Arc<dyn UserEmotionAnalyzer>,
    warned: AtomicBool,
}

impl RemoteUserEmotionAnalyzerPlaceholder {
    pub fn new() -> Self {
        Self {
            inner: default_user_emotion_slot_v1(),
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

#[cfg(all(test, feature = "default-emotion-providers"))]
mod tests {
    use super::*;
    use crate::models::Emotion;
    use oclive_emotion_builtin::classic::EmotionAnalyzer;

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
