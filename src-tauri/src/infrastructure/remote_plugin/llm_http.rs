//! JSON-RPC：`llm.generate` / `llm.generate_tag`

use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

const METHOD_LLM_GENERATE: &str = "llm.generate";
const METHOD_LLM_GENERATE_TAG: &str = "llm.generate_tag";

pub struct RemoteLlmHttp {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
    high_risk_grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteLlmHttp {
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()?;
        Ok(Self {
            client,
            cfg,
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

#[async_trait]
impl LlmClient for RemoteLlmHttp {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = jsonrpc::call_async(
            RemoteRpcChannel::Llm,
            &self.client,
            &self.cfg.endpoint,
            METHOD_LLM_GENERATE,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
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
            RemoteRpcChannel::Llm,
            &self.client,
            &self.cfg.endpoint,
            METHOD_LLM_GENERATE_TAG,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
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

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_match_remote_protocol() {
        assert_eq!(METHOD_LLM_GENERATE, "llm.generate");
        assert_eq!(METHOD_LLM_GENERATE_TAG, "llm.generate_tag");
    }
}
