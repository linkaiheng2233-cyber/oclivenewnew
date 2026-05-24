//! JSON-RPC：`complex_emotion.resolve_turn`（Remote 专用端点，与通用 `OCLIVE_REMOTE_PLUGIN_URL` 分离）。

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

pub struct RemoteComplexEmotionHttp {
    adapter: RemotePluginAdapterBlocking,
    fallback: BuiltinKeywordComplexEmotionProvider,
}

impl RemoteComplexEmotionHttp {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = crate::infrastructure::remote_plugin::RemoteHttpClientBlocking::new_standalone(
            cfg,
            high_risk_grants,
            Some(NETWORK_GRANT_REMOTE_PLUGIN.to_string()),
        )?;
        Ok(Self {
            adapter: RemotePluginAdapterBlocking::from_http(http, remote_fallback_allowed),
            fallback: BuiltinKeywordComplexEmotionProvider,
        })
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_name_matches_doc() {
        assert_eq!(METHOD_RESOLVE_TURN, "complex_emotion.resolve_turn");
    }
}
