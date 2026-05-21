//! JSON-RPC：`llm.generate` / `llm.generate_tag`

use crate::domain::error_helpers::ollama_msg;
use crate::domain::ports::LlmClient;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientAsync;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

const METHOD_LLM_GENERATE: &str = "llm.generate";
const METHOD_LLM_GENERATE_TAG: &str = "llm.generate_tag";

pub struct RemoteLlmHttp {
    http: RemoteHttpClientAsync,
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
        let http = RemoteHttpClientAsync::new(cfg, high_risk_grants, network_grant_id)?;
        Ok(Self { http })
    }

    fn text_from_result(v: serde_json::Value, method: &str) -> Result<String> {
        v.get("text")
            .and_then(|x| x.as_str())
            .map(String::from)
            .or_else(|| v.as_str().map(String::from))
            .ok_or_else(|| ollama_msg(method, "missing text"))
    }
}

#[async_trait]
impl LlmClient for RemoteLlmHttp {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = self
            .http
            .call_llm(METHOD_LLM_GENERATE, params)
            .await?;
        Self::text_from_result(v, METHOD_LLM_GENERATE)
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = self
            .http
            .call_llm(METHOD_LLM_GENERATE_TAG, params)
            .await?;
        Self::text_from_result(v, METHOD_LLM_GENERATE_TAG)
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
