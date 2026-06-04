//! Directory plugin JSON-RPC: `complex_emotion.resolve_turn`.

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    ComplexEmotionProvider,
};
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

pub struct DirectoryComplexEmotionHttp {
    adapter: RemotePluginAdapterBlocking,
    fallback: BuiltinKeywordComplexEmotionProvider,
}

impl DirectoryComplexEmotionHttp {
    /// # Errors
    ///
    /// Returns [`Err`] when the HTTP client cannot be built.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = RemoteHttpClientBlocking::new_standalone(
            cfg,
            HighRiskGrantStore::load(std::env::temp_dir(), false),
            None,
        )?;
        Ok(Self {
            adapter: RemotePluginAdapterBlocking::from_http(http, remote_fallback_allowed),
            fallback: BuiltinKeywordComplexEmotionProvider,
        })
    }
}

impl ComplexEmotionProvider for DirectoryComplexEmotionHttp {
    fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        let params =
            serde_json::to_value(input).map_err(|e| serde_to_ollama("complex_emotion params json", e))?;
        self.adapter.call_with_builtin_fallback(
            METHOD_RESOLVE_TURN,
            params,
            |v| {
                let mut out: ComplexEmotionOutput = serde_json::from_value(v)
                    .map_err(|e| serde_to_ollama("complex_emotion result decode", e))?;
                out.degraded_to_builtin = false;
                Ok(out)
            },
            || {
                let mut o = self.fallback.resolve_turn_inner(input);
                o.degraded_to_builtin = true;
                Ok(o)
            },
        )
    }
}
