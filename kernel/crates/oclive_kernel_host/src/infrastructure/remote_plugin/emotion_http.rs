//! JSON-RPC：`emotion.analyze`

use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::error_helpers::serde_to_ollama;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::domain::BuiltinUserEmotionAnalyzer;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use serde_json::json;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_EMOTION_ANALYZE: &str = "emotion.analyze";

pub struct RemoteUserEmotionAnalyzerHttp {
    adapter: RemotePluginAdapterBlocking,
    fallback: BuiltinUserEmotionAnalyzer,
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
        Self {
            adapter: RemotePluginAdapterBlocking::new(
                http_client,
                cfg,
                remote_fallback_allowed,
                high_risk_grants,
                network_grant_id,
            ),
            fallback: BuiltinUserEmotionAnalyzer,
        }
    }
}

impl UserEmotionAnalyzer for RemoteUserEmotionAnalyzerHttp {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        let params = json!({ "text": text });
        self.adapter.call_with_builtin_fallback(
            METHOD_EMOTION_ANALYZE,
            params,
            |v| serde_json::from_value(v).map_err(|e| serde_to_ollama("emotion.analyze decode", e)),
            || self.fallback.analyze(text),
        )
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
