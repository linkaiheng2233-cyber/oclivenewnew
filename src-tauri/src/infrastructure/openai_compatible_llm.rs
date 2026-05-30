//! OpenAI-compatible `POST /v1/chat/completions` client (DeepSeek, relay endpoints, OpenAI, etc.).

use crate::domain::ports::LlmClient;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm_params;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

/// Normalize user/base URL to `…/v1/chat/completions`.
#[must_use]
pub fn chat_completions_url(base: &str) -> String {
    let b = base.trim().trim_end_matches('/').to_string();
    if b.is_empty() {
        return String::new();
    }
    if b.ends_with("/chat/completions") {
        return b;
    }
    if b.ends_with("/v1") {
        return format!("{b}/chat/completions");
    }
    if b.contains("/v1/") {
        return b;
    }
    format!("{b}/v1/chat/completions")
}

pub struct OpenAiCompatibleLlm {
    chat_url: String,
    bearer_token: Option<String>,
    client: Client,
    timeout: Duration,
    grants: Arc<HighRiskGrantStore>,
    network_grant_id: String,
}

impl OpenAiCompatibleLlm {
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.chat_url
    }

    pub fn from_env(
        http: Client,
        grants: Arc<HighRiskGrantStore>,
    ) -> Option<Self> {
        let base = std::env::var("OCLIVE_REMOTE_LLM_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OPENAI_API_BASE").ok())
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())?;
        let chat_url = chat_completions_url(base.trim());
        if chat_url.is_empty() {
            return None;
        }
        let bearer_token = std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
            .ok()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .or_else(super::user_llm_secrets::cached_remote_llm_token)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let timeout_ms = std::env::var("OCLIVE_REMOTE_LLM_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(120_000);
        Some(Self {
            chat_url,
            bearer_token,
            client: http,
            timeout: Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
            grants,
            network_grant_id: NETWORK_GRANT_REMOTE_LLM.to_string(),
        })
    }

    fn ensure_network_grant(&self) -> Result<()> {
        self.grants.require_network(&self.network_grant_id)
    }

    async fn chat(&self, model: &str, prompt: &str, temperature: f32, top_p: f32) -> Result<String> {
        self.ensure_network_grant()?;
        let body = serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": prompt }],
            "temperature": temperature,
            "top_p": top_p,
            "stream": false,
        });
        let mut req = self
            .client
            .post(&self.chat_url)
            .timeout(self.timeout)
            .header("Content-Type", "application/json");
        if let Some(ref token) = self.bearer_token {
            req = req.bearer_auth(token);
        }
        let response = req
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::RemoteServiceUnavailable(format!("OpenAI API request: {e}")))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AppError::RemoteServiceUnavailable(format!("OpenAI API body: {e}")))?;
        if !status.is_success() {
            return Err(AppError::RemoteServiceUnavailable(format!(
                "OpenAI API HTTP {status}: {}",
                text.chars().take(600).collect::<String>()
            )));
        }
        parse_chat_response(&text)
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Option<Vec<ChatChoice>>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: Option<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

fn parse_chat_response(body: &str) -> Result<String> {
    let parsed: ChatCompletionResponse = serde_json::from_str(body).map_err(|e| {
        AppError::RemoteServiceUnavailable(format!("OpenAI API JSON parse: {e}"))
    })?;
    parsed
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            AppError::RemoteServiceUnavailable(
                "OpenAI API response missing choices[0].message.content".into(),
            )
        })
}

#[async_trait]
impl LlmClient for OpenAiCompatibleLlm {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        self.chat(
            model,
            prompt,
            t.unwrap_or(0.8),
            p.unwrap_or(0.9),
        )
        .await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        self.chat(
            model,
            prompt,
            t.unwrap_or(0.28),
            p.unwrap_or(0.85),
        )
        .await
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}
