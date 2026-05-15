//! JSON-RPC：`complex_emotion.resolve_turn`（Remote 专用端点，与通用 `OCLIVE_REMOTE_PLUGIN_URL` 分离）。

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::error::{AppError, Result};
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

pub struct RemoteComplexEmotionHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinKeywordComplexEmotionProvider,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
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
        let client = reqwest::blocking::Client::builder()
            .timeout(cfg.timeout)
            .connect_timeout(cfg.connect_timeout())
            .build()?;
        Ok(Self {
            client,
            cfg,
            fallback: BuiltinKeywordComplexEmotionProvider,
            remote_fallback_allowed,
            high_risk_grants,
        })
    }

    fn network_grant(&self) -> Option<(&HighRiskGrantStore, &str)> {
        Some((self.high_risk_grants.as_ref(), NETWORK_GRANT_REMOTE_PLUGIN))
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        let params = serde_json::to_value(input).map_err(|e| {
            crate::error::AppError::OllamaError(format!("complex_emotion params json: {}", e))
        })?;
        match jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_RESOLVE_TURN,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
        ) {
            Ok(v) => {
                let mut out: ComplexEmotionOutput = serde_json::from_value(v).map_err(|e| {
                    crate::error::AppError::OllamaError(format!(
                        "complex_emotion result decode: {}",
                        e
                    ))
                })?;
                out.degraded_to_builtin = false;
                Ok(out)
            }
            Err(e) => {
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "complex_emotion.resolve_turn remote failed endpoint={} err={}; fallback=builtin",
                        self.cfg.endpoint,
                        e
                    );
                    let mut o = self.fallback.resolve_turn_inner(input);
                    o.degraded_to_builtin = true;
                    Ok(o)
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "complex_emotion.resolve_turn remote failed endpoint={} err={}",
                        self.cfg.endpoint, e
                    )))
                }
            }
        }
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
