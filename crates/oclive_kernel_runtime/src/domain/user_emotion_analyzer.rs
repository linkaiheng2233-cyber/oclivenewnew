//! Swappable user/text emotion-analysis facade; default delegates to [`EmotionAnalyzer`](super::emotion_analyzer::EmotionAnalyzer).

use crate::domain::emotion_analyzer::{EmotionAnalyzer, EmotionResult};
use crate::error::Result;
use std::sync::atomic::{AtomicBool, Ordering};

pub use oclive_kernel_contracts::UserEmotionAnalyzer;

pub struct BuiltinUserEmotionAnalyzer;

impl UserEmotionAnalyzer for BuiltinUserEmotionAnalyzer {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        EmotionAnalyzer::analyze(text)
    }
}

pub struct RemoteUserEmotionAnalyzerPlaceholder {
    inner: BuiltinUserEmotionAnalyzer,
    warned: AtomicBool,
}

impl RemoteUserEmotionAnalyzerPlaceholder {
    #[must_use]
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
            tracing::warn!(
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
