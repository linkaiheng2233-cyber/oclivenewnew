//! Directory plugin JSON-RPC: `complex_emotion.resolve_turn`.

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    ComplexEmotionProvider,
};
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::{
    resolve_turn_rpc, RemotePluginAdapterBlocking,
};
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
        resolve_turn_rpc(&self.adapter, &self.fallback, input)
    }
}
