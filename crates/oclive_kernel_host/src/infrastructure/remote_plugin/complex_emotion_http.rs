//! JSON-RPC: `complex_emotion.resolve_turn` (Remote-only endpoint, separate from generic `OCLIVE_REMOTE_PLUGIN_URL`).

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::{resolve_turn_rpc, RemotePluginAdapterBlocking};
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
        resolve_turn_rpc(&self.adapter, &self.fallback, input)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn method_name_matches_doc() {
        assert_eq!("complex_emotion.resolve_turn", "complex_emotion.resolve_turn");
    }
}
