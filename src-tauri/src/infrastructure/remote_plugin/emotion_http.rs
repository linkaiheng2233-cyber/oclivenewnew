//! JSON-RPC：`emotion.analyze`

use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::error_helpers::serde_to_ollama;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::domain::BuiltinUserEmotionAnalyzer;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_EMOTION_ANALYZE: &str = "emotion.analyze";

pub struct RemoteUserEmotionAnalyzerHttp {
    http: RemoteHttpClientBlocking,
    fallback: BuiltinUserEmotionAnalyzer,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl RemoteUserEmotionAnalyzerHttp {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let http = RemoteHttpClientBlocking::new(
            http_client,
            cfg,
            high_risk_grants,
            network_grant_id,
        );
        Self {
            http,
            fallback: BuiltinUserEmotionAnalyzer,
            remote_fallback_allowed,
        }
    }
}

impl UserEmotionAnalyzer for RemoteUserEmotionAnalyzerHttp {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        let params = json!({ "text": text });
        match self.http.call_plugin(METHOD_EMOTION_ANALYZE, params) {
            Ok(v) => {
                let r: EmotionResult =
                    serde_json::from_value(v).map_err(|e| serde_to_ollama("emotion.analyze decode", e))?;
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
                        self.http.endpoint(),
                        e
                    );
                    self.fallback.analyze(text)
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "emotion.analyze remote failed endpoint={} err={}",
                        self.http.endpoint(),
                        e
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
