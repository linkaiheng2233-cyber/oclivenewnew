//! 复杂情感：内置关键词模式（设施 crate）+ 可选 Remote/Directory 侧车。

#[cfg(not(feature = "default-complex-emotion-providers"))]
use crate::domain::disabled_default_providers::DisabledComplexEmotionProvider;
use crate::error::Result;
pub use oclive_complex_emotion_builtin::classic::affect_metrics_from_seven_dim;
#[cfg(feature = "default-complex-emotion-providers")]
pub use oclive_complex_emotion_builtin::BuiltinKeywordComplexEmotionProvider;
pub use oclive_kernel_core::complex_emotion::{
    ComplexEmotionInput, ComplexEmotionOutput, ComplexEmotionProvider,
};
use std::sync::Arc;

/// 空 Provider：不做复盘、不注入 prompt；输出稳定、可预期。
pub struct NoneComplexEmotionProvider;

impl ComplexEmotionProvider for NoneComplexEmotionProvider {
    fn resolve_turn(&self, _input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        Ok(ComplexEmotionOutput {
            source: "none".to_string(),
            narrative_hint: None,
            labels: vec![],
            pattern: None,
            confidence: 0.0,
            intensity: 0.0,
            dissonance_score: 0.0,
            degraded_to_builtin: false,
        })
    }
}

/// 降级 Provider：用 builtin 产出，但强制标记 `degraded_to_builtin=true`（用于 env 缺失/目录插件不可用等场景）。
pub struct DegradedToBuiltinComplexEmotionProvider {
    fallback: Arc<dyn ComplexEmotionProvider>,
    warned: std::sync::atomic::AtomicBool,
    warn_message: &'static str,
}

impl DegradedToBuiltinComplexEmotionProvider {
    pub fn new(warn_message: &'static str) -> Self {
        Self {
            fallback: default_complex_emotion_keyword_arc(),
            warned: std::sync::atomic::AtomicBool::new(false),
            warn_message,
        }
    }

    fn warn_once(&self) {
        use std::sync::atomic::Ordering;
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            log::warn!(target: "oclive_plugin", "{}", self.warn_message);
        }
    }
}

impl ComplexEmotionProvider for DegradedToBuiltinComplexEmotionProvider {
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        self.warn_once();
        let mut o = self.fallback.resolve_turn(input)?;
        o.degraded_to_builtin = true;
        Ok(o)
    }
}

#[must_use]
pub fn default_complex_emotion_keyword_arc() -> Arc<dyn ComplexEmotionProvider> {
    #[cfg(feature = "default-complex-emotion-providers")]
    {
        Arc::new(BuiltinKeywordComplexEmotionProvider)
    }
    #[cfg(not(feature = "default-complex-emotion-providers"))]
    {
        Arc::new(DisabledComplexEmotionProvider)
    }
}
