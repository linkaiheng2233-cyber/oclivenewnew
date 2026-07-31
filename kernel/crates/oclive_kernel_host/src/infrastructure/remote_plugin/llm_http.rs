//! JSON-RPC：`llm.generate` / `llm.generate_tag`，可选 NDJSON `llm.generate_stream`

use crate::domain::error_helpers::ollama_msg;
use crate::domain::ports::LlmClient;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientAsync;
use async_trait::async_trait;
use oclive_kernel_contracts::{LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use serde_json::json;
use std::sync::Arc;

const METHOD_LLM_GENERATE: &str = "llm.generate";
const METHOD_LLM_GENERATE_TAG: &str = "llm.generate_tag";
pub(crate) const METHOD_LLM_GENERATE_STREAM: &str = "llm.generate_stream";

pub struct RemoteLlmHttp {
    http: RemoteHttpClientAsync,
    native_stream: bool,
}

impl RemoteLlmHttp {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let http = RemoteHttpClientAsync::new(http_client, cfg, high_risk_grants, network_grant_id);
        Self {
            http,
            native_stream: false,
        }
    }

    /// Enables the optional NDJSON stream method when a directory manifest
    /// explicitly declares it. Undeclared/legacy plugins retain full-response
    /// `llm.generate` behavior.
    #[must_use]
    pub fn with_native_stream(mut self, enabled: bool) -> Self {
        self.native_stream = enabled;
        self
    }

    fn text_from_result(v: serde_json::Value, method: &str) -> Result<String> {
        v.get("text")
            .and_then(|x| x.as_str())
            .map(String::from)
            .or_else(|| v.as_str().map(String::from))
            .ok_or_else(|| ollama_msg(method, "missing text"))
    }

    fn outcome_from_result(v: serde_json::Value, method: &str) -> Result<LlmGenerateOutcome> {
        let prompt_eval_ms = v.get("prompt_eval_ms").and_then(serde_json::Value::as_u64);
        let reply = Self::text_from_result(v, method)?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms,
        })
    }
}

#[async_trait]
impl LlmClient for RemoteLlmHttp {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = self.http.call_llm(METHOD_LLM_GENERATE, params).await?;
        Self::text_from_result(v, METHOD_LLM_GENERATE)
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let v = self.http.call_llm(METHOD_LLM_GENERATE_TAG, params).await?;
        Self::text_from_result(v, METHOD_LLM_GENERATE_TAG)
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _ = opts;
        if !self.native_stream {
            let reply = self.generate(model, prompt).await?;
            on_token(reply.as_str());
            return Ok(LlmGenerateOutcome {
                reply,
                prompt_eval_ms: None,
            });
        }
        let params = json!({
            "model": model,
            "prompt": prompt,
        });
        let result = self
            .http
            .call_llm_stream(METHOD_LLM_GENERATE_STREAM, params, on_token)
            .await?;
        Self::outcome_from_result(result, METHOD_LLM_GENERATE_STREAM)
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
        assert_eq!(METHOD_LLM_GENERATE_STREAM, "llm.generate_stream");
    }
}
