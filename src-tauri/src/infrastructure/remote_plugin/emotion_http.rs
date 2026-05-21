//! JSON-RPC：`emotion.analyze`

use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::domain::BuiltinUserEmotionAnalyzer;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_EMOTION_ANALYZE: &str = "emotion.analyze";

pub struct RemoteUserEmotionAnalyzerHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinUserEmotionAnalyzer,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteUserEmotionAnalyzerHttp {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let client = reqwest::blocking::Client::builder()
            .timeout(cfg.timeout)
            .build()?;
        Ok(Self {
            client,
            cfg,
            fallback: BuiltinUserEmotionAnalyzer,
            remote_fallback_allowed,
            high_risk_grants,
            network_grant_id,
        })
    }

    fn network_grant(&self) -> Option<(&HighRiskGrantStore, &str)> {
        self.network_grant_id
            .as_deref()
            .map(|id| (self.high_risk_grants.as_ref(), id))
    }
}

impl UserEmotionAnalyzer for RemoteUserEmotionAnalyzerHttp {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        let params = json!({ "text": text });
        match jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_EMOTION_ANALYZE,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
        ) {
            Ok(v) => {
                let r: EmotionResult = serde_json::from_value(v).map_err(|e| {
                    crate::error::AppError::OllamaError(format!("emotion.analyze decode: {}", e))
                })?;
                Ok(r)
            }
            Err(e) => {
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "emotion.analyze remote failed endpoint={} err={}; fallback=builtin",
                        self.cfg.endpoint,
                        e
                    );
                    self.fallback.analyze(text)
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "emotion.analyze remote failed endpoint={} err={}",
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
    fn method_name_matches_remote_protocol() {
        assert_eq!(METHOD_EMOTION_ANALYZE, "emotion.analyze");
    }
}
