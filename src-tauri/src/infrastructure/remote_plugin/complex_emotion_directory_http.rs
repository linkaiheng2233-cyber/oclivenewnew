//! 目录插件 JSON-RPC：`complex_emotion.resolve_turn`。

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    ComplexEmotionProvider,
};
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

pub struct DirectoryComplexEmotionHttp {
    http: RemoteHttpClientBlocking,
    fallback: BuiltinKeywordComplexEmotionProvider,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl DirectoryComplexEmotionHttp {
    /// # Errors
    ///
    /// Returns [`Err`] when the HTTP client cannot be built.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = RemoteHttpClientBlocking::new(
            cfg,
            HighRiskGrantStore::load(std::env::temp_dir(), false),
            None,
        )?;
        Ok(Self {
            http,
            fallback: BuiltinKeywordComplexEmotionProvider,
            remote_fallback_allowed,
        })
    }
}

impl ComplexEmotionProvider for DirectoryComplexEmotionHttp {
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        let params =
            serde_json::to_value(input).map_err(|e| serde_to_ollama("complex_emotion params json", e))?;
        match self.http.call_plugin(METHOD_RESOLVE_TURN, params) {
            Ok(v) => {
                let mut out: ComplexEmotionOutput = serde_json::from_value(v)
                    .map_err(|e| serde_to_ollama("complex_emotion result decode", e))?;
                out.degraded_to_builtin = false;
                Ok(out)
            }
            Err(e) => {
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "complex_emotion.resolve_turn directory failed endpoint={} err={}; fallback=builtin",
                        self.http.endpoint(),
                        e
                    );
                    let mut o = self.fallback.resolve_turn_inner(input);
                    o.degraded_to_builtin = true;
                    Ok(o)
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "complex_emotion.resolve_turn directory failed endpoint={} err={}",
                        self.http.endpoint(),
                        e
                    )))
                }
            }
        }
    }
}
