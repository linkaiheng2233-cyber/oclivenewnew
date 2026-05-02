//! Cloud LLM: OpenAI-compatible HTTP client.
//!
//! This is intended for "cloud API" usage without requiring a JSON-RPC sidecar.

use crate::error::{AppError, Result};
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::llm_params;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CloudLlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
    pub default_model: Option<String>,
}

impl CloudLlmConfig {
    /// `OCLIVE_CLOUD_LLM_BASE_URL` + `OCLIVE_CLOUD_LLM_API_KEY`
    /// Optional: `OCLIVE_CLOUD_LLM_MODEL`, `OCLIVE_CLOUD_LLM_TIMEOUT_MS`
    pub fn from_env_openai_compat() -> Option<Self> {
        let base_url = std::env::var("OCLIVE_CLOUD_LLM_BASE_URL").ok()?;
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return None;
        }
        let api_key = std::env::var("OCLIVE_CLOUD_LLM_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if api_key.is_empty() {
            return None;
        }
        let timeout_ms = std::env::var("OCLIVE_CLOUD_LLM_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(120_000);
        let default_model = std::env::var("OCLIVE_CLOUD_LLM_MODEL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Some(Self {
            base_url,
            api_key,
            timeout: Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
            default_model,
        })
    }
}

/// 与 `app_settings.host_cloud_llm_json` 及保存 DTO 一致（camelCase JSON）。
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostCloudLlmJson {
    pub base_url: String,
    pub api_key: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

pub fn cloud_config_from_persist_json(s: &str) -> Option<CloudLlmConfig> {
    let j: HostCloudLlmJson = serde_json::from_str(s.trim()).ok()?;
    let base_url = j.base_url.trim().trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return None;
    }
    let api_key = j.api_key.trim().to_string();
    if api_key.is_empty() {
        return None;
    }
    let timeout_ms = j.timeout_ms.unwrap_or(120_000).clamp(1_000, 600_000);
    let default_model = j
        .model
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty());
    Some(CloudLlmConfig {
        base_url,
        api_key,
        timeout: Duration::from_millis(timeout_ms),
        default_model,
    })
}

/// 应用内覆盖优先于环境变量（与 [`CloudLlmConfig::from_env_openai_compat`] 合并）。
pub fn effective_cloud_llm_config(store: &RwLock<Option<CloudLlmConfig>>) -> Option<CloudLlmConfig> {
    let g = store.read();
    if let Some(c) = g.as_ref() {
        if !c.base_url.trim().is_empty() && !c.api_key.trim().is_empty() {
            return Some(c.clone());
        }
    }
    drop(g);
    CloudLlmConfig::from_env_openai_compat()
}

pub const HOST_CLOUD_LLM_JSON_KEY: &str = "host_cloud_llm_json";
pub const HOST_CHAT_MODEL_KEY: &str = "host_chat_model";

#[derive(Debug, Clone)]
pub struct OpenAiCompatLlmClient {
    client: reqwest::Client,
    cfg: CloudLlmConfig,
}

impl OpenAiCompatLlmClient {
    pub fn new(cfg: CloudLlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest client");
        Self { client, cfg }
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.cfg.base_url)
    }

    fn pick_model<'a>(&'a self, model: &'a str) -> &'a str {
        let t = model.trim();
        if !t.is_empty() {
            return t;
        }
        self.cfg
            .default_model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("gpt-4o-mini")
    }

    async fn call(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<String> {
        let req = OpenAiChatCompletionsRequest {
            model: self.pick_model(model).to_string(),
            messages: vec![OpenAiChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            top_p,
        };
        let resp = self
            .client
            .post(self.endpoint())
            .bearer_auth(self.cfg.api_key.as_str())
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::OllamaError(format!("cloud llm request failed: {}", e)))?;
        let status = resp.status();
        let raw = resp
            .text()
            .await
            .unwrap_or_else(|_| "<read body failed>".to_string());
        if !status.is_success() {
            return Err(AppError::OllamaError(format!(
                "cloud llm http {}: {}",
                status.as_u16(),
                raw
            )));
        }
        let parsed: OpenAiChatCompletionsResponse = serde_json::from_str(&raw).map_err(|e| {
            AppError::OllamaError(format!("cloud llm parse failed: {} raw={}", e, raw))
        })?;
        let text = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::OllamaError("cloud llm: empty response".to_string()))?;
        Ok(text)
    }
}

#[async_trait]
impl LlmClient for OpenAiCompatLlmClient {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::main_chat_options();
        self.call(model, prompt, t, p).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let (t, p) = llm_params::tag_task_options();
        self.call(model, prompt, t, p).await
    }
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatCompletionsRequest {
    model: String,
    messages: Vec<OpenAiChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatCompletionsResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChoiceMessage {
    content: Option<String>,
}
