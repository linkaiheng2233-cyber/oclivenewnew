//! JSON-RPC：`emotion.analyze`

use crate::domain::emotion_analyzer::EmotionResult;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::domain::BuiltinUserEmotionAnalyzer;
use crate::error::Result;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc;
use serde_json::json;

pub struct RemoteUserEmotionAnalyzerHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinUserEmotionAnalyzer,
}

impl RemoteUserEmotionAnalyzerHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest blocking client");
        Self {
            client,
            cfg,
            fallback: BuiltinUserEmotionAnalyzer,
        }
    }
}

impl UserEmotionAnalyzer for RemoteUserEmotionAnalyzerHttp {
    fn analyze(&self, text: &str) -> Result<EmotionResult> {
        let params = json!({ "text": text });
        match jsonrpc::call_blocking(
            &self.client,
            &self.cfg.endpoint,
            "emotion.analyze",
            params,
            self.cfg.bearer_token.as_deref(),
        ) {
            Ok(v) => {
                let r: EmotionResult = serde_json::from_value(v).map_err(|e| {
                    crate::error::AppError::OllamaError(format!("emotion.analyze decode: {}", e))
                })?;
                Ok(r)
            }
            Err(e) => {
                log::warn!(
                    target: "oclive_plugin",
                    "emotion.analyze remote failed: {}; builtin fallback",
                    e
                );
                self.fallback.analyze(text)
            }
        }
    }
}
