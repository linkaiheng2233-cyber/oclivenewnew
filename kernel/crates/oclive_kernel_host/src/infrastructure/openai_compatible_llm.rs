//! OpenAI-compatible `POST /v1/chat/completions` client (DeepSeek, relay endpoints, OpenAI, etc.).

use crate::domain::ports::LlmClient;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm_params;
use async_trait::async_trait;
use futures_util::StreamExt;
use oclive_kernel_contracts::{LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
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

/// Normalize user/base URL to `…/v1/models` (OpenAI-compatible model listing).
#[must_use]
pub fn models_list_url(base: &str) -> String {
    let b = base.trim().trim_end_matches('/').to_string();
    if b.is_empty() {
        return String::new();
    }
    if b.ends_with("/models") {
        return b;
    }
    if b.ends_with("/chat/completions") {
        let root = b.trim_end_matches("/chat/completions");
        if root.ends_with("/v1") {
            return format!("{root}/models");
        }
    }
    if b.ends_with("/v1") {
        return format!("{b}/models");
    }
    if b.contains("/v1/") {
        return b;
    }
    format!("{b}/v1/models")
}

/// Clone the caller's client for normal remote traffic, but construct an explicit
/// no-proxy client for loopback endpoints. `reqwest::Client::clone` is cheap and
/// preserves connection pooling for the common cloud case.
fn client_for_endpoint(client: &Client, endpoint: &str) -> Client {
    if !crate::infrastructure::is_loopback_endpoint(endpoint) {
        return client.clone();
    }
    Client::builder()
        .no_proxy()
        .build()
        .unwrap_or_else(|_| client.clone())
}

pub struct OpenAiCompatibleLlm {
    chat_url: String,
    bearer_token: Option<String>,
    client: Client,
    timeout: Duration,
    grants: Option<Arc<HighRiskGrantStore>>,
    network_grant_id: String,
}

impl OpenAiCompatibleLlm {
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.chat_url
    }

    pub fn from_env(http: Client, grants: Arc<HighRiskGrantStore>) -> Option<Self> {
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
        let client = client_for_endpoint(&http, &chat_url);
        Some(Self {
            chat_url,
            bearer_token,
            client,
            timeout: Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
            grants: Some(grants),
            network_grant_id: NETWORK_GRANT_REMOTE_LLM.to_string(),
        })
    }

    /// Construct a loopback-only OpenAI-compatible client for the distro's managed
    /// llama-server runtime. Builtin local inference follows the same trust boundary as
    /// Ollama and therefore does not use the remote-plugin network grant.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::InvalidParameter`] when `base` is not a loopback HTTP endpoint.
    pub fn for_local_runtime(base: &str, timeout: Duration) -> Result<Self> {
        let chat_url = chat_completions_url(base);
        if chat_url.is_empty() || !crate::infrastructure::is_loopback_endpoint(&chat_url) {
            return Err(AppError::InvalidParameter(
                "local LLM runtime endpoint must use localhost/127.0.0.1/[::1]".into(),
            ));
        }
        let client = Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_secs(2))
            .pool_max_idle_per_host(4)
            .build()
            .map_err(|e| AppError::InvalidParameter(format!("local LLM HTTP client: {e}")))?;
        Ok(Self {
            chat_url,
            bearer_token: None,
            client,
            timeout: timeout.clamp(Duration::from_secs(1), Duration::from_secs(600)),
            grants: None,
            network_grant_id: String::new(),
        })
    }

    fn ensure_network_grant(&self) -> Result<()> {
        if let Some(ref grants) = self.grants {
            grants.require_network(&self.network_grant_id)?;
        }
        Ok(())
    }

    async fn chat(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        top_p: f32,
    ) -> Result<String> {
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
        let response =
            req.json(&body).send().await.map_err(|e| {
                AppError::RemoteServiceUnavailable(format!("OpenAI API request: {e}"))
            })?;
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

    async fn chat_stream(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        top_p: f32,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        self.ensure_network_grant()?;
        let body = serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": prompt }],
            "temperature": temperature,
            "top_p": top_p,
            "stream": true,
        });
        let mut req = self
            .client
            .post(&self.chat_url)
            .timeout(self.timeout)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream");
        if let Some(ref token) = self.bearer_token {
            req = req.bearer_auth(token);
        }
        let response = req.json(&body).send().await.map_err(|e| {
            AppError::RemoteServiceUnavailable(format!("OpenAI stream request: {e}"))
        })?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::RemoteServiceUnavailable(format!(
                "OpenAI stream HTTP {status}: {}",
                text.chars().take(600).collect::<String>()
            )));
        }

        let mut stream = response.bytes_stream();
        let mut pending = Vec::<u8>::new();
        let mut reply = String::new();
        let mut done = false;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| {
                AppError::RemoteServiceUnavailable(format!("OpenAI stream body: {e}"))
            })?;
            pending.extend_from_slice(&bytes);
            while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
                let mut line: Vec<u8> = pending.drain(..=newline).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                let line = std::str::from_utf8(&line).map_err(|e| {
                    AppError::RemoteServiceUnavailable(format!("OpenAI stream UTF-8: {e}"))
                })?;
                if consume_sse_line(line, &mut reply, on_token.as_ref())? {
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
        }
        if !done && !pending.is_empty() {
            if pending.last() == Some(&b'\r') {
                pending.pop();
            }
            let line = std::str::from_utf8(&pending).map_err(|e| {
                AppError::RemoteServiceUnavailable(format!("OpenAI stream UTF-8: {e}"))
            })?;
            if !line.trim().is_empty() {
                let _ = consume_sse_line(line, &mut reply, on_token.as_ref())?;
            }
        }
        if reply.is_empty() {
            return Err(AppError::RemoteServiceUnavailable(
                "OpenAI stream ended without assistant content".into(),
            ));
        }
        Ok(reply)
    }
}

fn consume_sse_line(
    line: &str,
    reply: &mut String,
    on_token: &(dyn Fn(&str) + Send + Sync),
) -> Result<bool> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return Ok(false);
    }
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(false);
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(true);
    }
    let chunk: ChatCompletionChunk = serde_json::from_str(data).map_err(|e| {
        AppError::RemoteServiceUnavailable(format!("OpenAI stream JSON parse: {e}"))
    })?;
    for choice in chunk.choices.unwrap_or_default() {
        if let Some(content) = choice.delta.and_then(|delta| delta.content) {
            if !content.is_empty() {
                reply.push_str(&content);
                on_token(&content);
            }
        }
    }
    Ok(false)
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    choices: Option<Vec<ChatChunkChoice>>,
}

#[derive(Debug, Deserialize)]
struct ChatChunkChoice {
    delta: Option<ChatDelta>,
}

#[derive(Debug, Deserialize)]
struct ChatDelta {
    content: Option<String>,
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
    let parsed: ChatCompletionResponse = serde_json::from_str(body)
        .map_err(|e| AppError::RemoteServiceUnavailable(format!("OpenAI API JSON parse: {e}")))?;
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

#[derive(Debug, Deserialize)]
struct ModelsListResponse {
    data: Option<Vec<ModelEntry>>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: Option<String>,
}

/// List model ids from an OpenAI-compatible `GET /v1/models` endpoint.
///
/// # Errors
///
/// Returns `AppError` when network is not granted, the request fails, or the response has no model ids.
pub async fn list_openai_compatible_models(
    client: &Client,
    base_url: &str,
    bearer_token: Option<&str>,
    timeout: Duration,
    grants: &HighRiskGrantStore,
) -> Result<Vec<String>> {
    grants.require_network(NETWORK_GRANT_REMOTE_LLM)?;
    let models_url = models_list_url(base_url);
    if models_url.is_empty() {
        return Err(AppError::InvalidParameter("云端 Base URL 为空".into()));
    }
    let client = client_for_endpoint(client, &models_url);
    let mut req = client
        .get(&models_url)
        .timeout(timeout)
        .header("Accept", "application/json");
    if let Some(token) = bearer_token.filter(|s| !s.trim().is_empty()) {
        req = req.bearer_auth(token.trim());
    }
    let response = req.send().await.map_err(|e| {
        AppError::RemoteServiceUnavailable(format!("OpenAI models list request: {e}"))
    })?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| AppError::RemoteServiceUnavailable(format!("OpenAI models list body: {e}")))?;
    if !status.is_success() {
        return Err(AppError::RemoteServiceUnavailable(format!(
            "OpenAI models list HTTP {status}: {}",
            text.chars().take(600).collect::<String>()
        )));
    }
    parse_models_list_response(&text)
}

fn parse_models_list_response(body: &str) -> Result<Vec<String>> {
    let parsed: ModelsListResponse = serde_json::from_str(body).map_err(|e| {
        AppError::RemoteServiceUnavailable(format!("OpenAI models list JSON parse: {e}"))
    })?;
    let mut ids: Vec<String> = parsed
        .data
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| entry.id.map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect();
    ids.sort_by_key(|a| a.to_ascii_lowercase());
    ids.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    if ids.is_empty() {
        return Err(AppError::RemoteServiceUnavailable(
            "OpenAI models list response contained no model ids".into(),
        ));
    }
    Ok(ids)
}

#[async_trait]
impl LlmClient for OpenAiCompatibleLlm {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        self.chat(model, prompt, t.unwrap_or(0.8), p.unwrap_or(0.9))
            .await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        self.chat(model, prompt, t.unwrap_or(0.28), p.unwrap_or(0.85))
            .await
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        _opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let reply = self.generate(model, prompt).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        self.chat_stream(model, prompt, t.unwrap_or(0.8), p.unwrap_or(0.9), on_token)
            .await
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        _opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let reply = self.generate_stream(model, prompt, on_token).await?;
        Ok(LlmGenerateOutcome {
            reply,
            prompt_eval_ms: None,
        })
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_completions_url_normalizes_base() {
        assert_eq!(
            chat_completions_url("https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn models_list_url_normalizes_base() {
        assert_eq!(
            models_list_url("https://api.openai.com"),
            "https://api.openai.com/v1/models"
        );
        assert_eq!(
            models_list_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/models"
        );
        assert_eq!(
            models_list_url("https://api.openai.com/v1/chat/completions"),
            "https://api.openai.com/v1/models"
        );
    }

    #[test]
    fn loopback_endpoint_detection_is_strict() {
        assert!(crate::infrastructure::is_loopback_endpoint(
            "http://localhost:1234/v1/chat/completions"
        ));
        assert!(crate::infrastructure::is_loopback_endpoint(
            "http://127.0.0.1:1234/v1/chat/completions"
        ));
        assert!(crate::infrastructure::is_loopback_endpoint(
            "http://[::1]:1234/v1/chat/completions"
        ));
        assert!(!crate::infrastructure::is_loopback_endpoint(
            "https://api.openai.com/v1/chat/completions"
        ));
        assert!(!crate::infrastructure::is_loopback_endpoint("not a URL"));
    }

    #[test]
    fn parse_models_list_response_extracts_ids() {
        let body = r#"{"data":[{"id":"gpt-4o-mini"},{"id":"gpt-4o"}]}"#;
        let ids = parse_models_list_response(body).expect("parse");
        assert_eq!(ids, vec!["gpt-4o".to_string(), "gpt-4o-mini".to_string()]);
    }

    #[test]
    fn sse_line_emits_incremental_content() {
        let tokens = std::sync::Mutex::new(Vec::<String>::new());
        let mut reply = String::new();
        let done = consume_sse_line(
            r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#,
            &mut reply,
            &|token| tokens.lock().unwrap().push(token.to_string()),
        )
        .unwrap();
        assert!(!done);
        assert_eq!(reply, "hello");
        assert_eq!(*tokens.lock().unwrap(), vec!["hello"]);
        assert!(consume_sse_line("data: [DONE]", &mut reply, &|_| {}).unwrap());
    }
}
