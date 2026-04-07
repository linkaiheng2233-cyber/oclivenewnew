//! JSON-RPC：`llm.generate` / `llm.generate_tag`

use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc;
use async_trait::async_trait;
use serde_json::json;

pub struct RemoteLlmHttp {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
}

impl RemoteLlmHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest async client");
        Self { client, cfg }
    }
}

#[async_trait]
impl LlmClient for RemoteLlmHttp {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = jsonrpc::call_async(
            &self.client,
            &self.cfg.endpoint,
            "llm.generate",
            params,
            self.cfg.bearer_token.as_deref(),
        )
        .await?;
        let text = v
            .get("text")
            .and_then(|x| x.as_str())
            .map(String::from)
            .or_else(|| v.as_str().map(String::from))
            .ok_or_else(|| {
                crate::error::AppError::OllamaError("llm.generate: missing text".to_string())
            })?;
        Ok(text)
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = jsonrpc::call_async(
            &self.client,
            &self.cfg.endpoint,
            "llm.generate_tag",
            params,
            self.cfg.bearer_token.as_deref(),
        )
        .await?;
        let text = v
            .get("text")
            .and_then(|x| x.as_str())
            .map(String::from)
            .or_else(|| v.as_str().map(String::from))
            .ok_or_else(|| {
                crate::error::AppError::OllamaError("llm.generate_tag: missing text".to_string())
            })?;
        Ok(text)
    }
}
