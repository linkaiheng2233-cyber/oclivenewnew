//! Cloud LLM: OpenAI-compatible HTTP client.
//!
//! This is intended for "cloud API" usage without requiring a JSON-RPC sidecar.

use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::llm_params;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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

    pub fn normalize_base_url(raw: &str) -> String {
        raw.trim().trim_end_matches('/').to_string()
    }

    pub fn validate_base_url_for_ui(raw: &str) -> Result<String> {
        let s = Self::normalize_base_url(raw);
        if s.is_empty() {
            return Err(AppError::InvalidParameter(
                "cloud_llm: base_url empty".to_string(),
            ));
        }
        let lower = s.to_ascii_lowercase();
        if !(lower.starts_with("https://") || lower.starts_with("http://")) {
            return Err(AppError::InvalidParameter(
                "cloud_llm: base_url must start with http:// or https://".to_string(),
            ));
        }
        Ok(s)
    }
}

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

// --- App settings + runtime (OpenAI-compatible cloud, UI / env) ---

pub const CLOUD_LLM_APP_KEY_BASE_URL: &str = "cloud_llm.base_url";
pub const CLOUD_LLM_APP_KEY_API_KEY: &str = "cloud_llm.api_key";
pub const CLOUD_LLM_APP_KEY_MODEL: &str = "cloud_llm.model";
pub const CLOUD_LLM_APP_KEY_TIMEOUT_MS: &str = "cloud_llm.timeout_ms";
/// `"1"`：禁止 OpenAI 兼容云端（环境变量与 UI 配置均不生效）；侧车 `OCLIVE_REMOTE_LLM_URL` 不受影响。
pub const CLOUD_LLM_APP_KEY_OPENAI_BLOCKED: &str = "cloud_llm.openai_blocked";
/// `"0"`：关闭「Ollama 自动升为 Remote」；缺省或 `"1"` 为开启。
pub const CLOUD_LLM_APP_KEY_AUTO_REMOTE_LLM: &str = "cloud_llm.auto_remote_llm";
/// `"1"`：用户已确认出站网络风险说明（首次保存云端配置前由前端写入）。
pub const CLOUD_LLM_APP_KEY_NETWORK_ACK: &str = "cloud_llm.network_ack";

pub async fn load_user_cloud_llm_from_db(db: &DbManager) -> Result<Option<CloudLlmConfig>> {
    let base_raw = db
        .get_app_setting(CLOUD_LLM_APP_KEY_BASE_URL)
        .await?
        .unwrap_or_default();
    let base_url = CloudLlmConfig::normalize_base_url(&base_raw);
    if base_url.is_empty() {
        return Ok(None);
    }
    let api_key = db
        .get_app_setting(CLOUD_LLM_APP_KEY_API_KEY)
        .await?
        .unwrap_or_default();
    if api_key.trim().is_empty() {
        return Ok(None);
    }
    let default_model = db
        .get_app_setting(CLOUD_LLM_APP_KEY_MODEL)
        .await?
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let timeout_ms = db
        .get_app_setting(CLOUD_LLM_APP_KEY_TIMEOUT_MS)
        .await?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120_000)
        .clamp(1_000, 600_000);
    Ok(Some(CloudLlmConfig {
        base_url,
        api_key,
        timeout: Duration::from_millis(timeout_ms),
        default_model,
    }))
}

pub async fn persist_user_cloud_llm_to_db(db: &DbManager, cfg: Option<&CloudLlmConfig>) -> Result<()> {
    if let Some(c) = cfg {
        db.upsert_app_setting(CLOUD_LLM_APP_KEY_BASE_URL, c.base_url.as_str())
            .await?;
        db.upsert_app_setting(CLOUD_LLM_APP_KEY_API_KEY, c.api_key.as_str())
            .await?;
        db.upsert_app_setting(
            CLOUD_LLM_APP_KEY_MODEL,
            c.default_model.as_deref().unwrap_or(""),
        )
        .await?;
        db.upsert_app_setting(
            CLOUD_LLM_APP_KEY_TIMEOUT_MS,
            &c.timeout.as_millis().to_string(),
        )
        .await?;
    } else {
        for k in [
            CLOUD_LLM_APP_KEY_BASE_URL,
            CLOUD_LLM_APP_KEY_API_KEY,
            CLOUD_LLM_APP_KEY_MODEL,
            CLOUD_LLM_APP_KEY_TIMEOUT_MS,
        ] {
            db.delete_app_setting(k).await?;
        }
    }
    Ok(())
}

async fn load_bool_setting(db: &DbManager, key: &str, default_on: bool) -> Result<bool> {
    let v = db.get_app_setting(key).await?.unwrap_or_default();
    let t = v.trim();
    if t.is_empty() {
        return Ok(default_on);
    }
    Ok(!(t == "0" || t.eq_ignore_ascii_case("false") || t.eq_ignore_ascii_case("off")))
}

pub async fn load_openai_blocked_from_db(db: &DbManager) -> Result<bool> {
    let v = db.get_app_setting(CLOUD_LLM_APP_KEY_OPENAI_BLOCKED).await?;
    Ok(matches!(
        v.as_deref().map(str::trim),
        Some("1" | "true" | "yes" | "on")
    ))
}

pub async fn load_auto_remote_llm_from_db(db: &DbManager) -> Result<bool> {
    load_bool_setting(db, CLOUD_LLM_APP_KEY_AUTO_REMOTE_LLM, true).await
}

pub async fn load_network_ack_from_db(db: &DbManager) -> Result<bool> {
    let v = db.get_app_setting(CLOUD_LLM_APP_KEY_NETWORK_ACK).await?;
    Ok(matches!(
        v.as_deref().map(str::trim),
        Some("1" | "true" | "yes" | "on")
    ))
}

/// 环境变量优先；`openai_blocked` 时禁用 OpenAI 兼容路径（含 `OCLIVE_CLOUD_LLM_*`）。
pub fn resolve_cloud_llm_config(runtime: &CloudLlmRuntime) -> Option<CloudLlmConfig> {
    if runtime.openai_hard_blocked() {
        return None;
    }
    if let Some(c) = CloudLlmConfig::from_env_openai_compat() {
        return Some(c);
    }
    runtime.user_config_snapshot()
}

#[derive(Debug)]
pub struct CloudLlmRuntime {
    user: RwLock<Option<CloudLlmConfig>>,
    openai_hard_blocked: RwLock<bool>,
    auto_remote_llm: RwLock<bool>,
    network_ack: RwLock<bool>,
}

impl CloudLlmRuntime {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            user: RwLock::new(None),
            openai_hard_blocked: RwLock::new(false),
            auto_remote_llm: RwLock::new(true),
            network_ack: RwLock::new(false),
        })
    }

    pub async fn bootstrap_from_db(db: &DbManager) -> Result<Arc<Self>> {
        let u = load_user_cloud_llm_from_db(db).await?;
        let blocked = load_openai_blocked_from_db(db).await?;
        let auto = load_auto_remote_llm_from_db(db).await?;
        let ack = load_network_ack_from_db(db).await?;
        Ok(Arc::new(Self {
            user: RwLock::new(u),
            openai_hard_blocked: RwLock::new(blocked),
            auto_remote_llm: RwLock::new(auto),
            network_ack: RwLock::new(ack),
        }))
    }

    #[must_use]
    pub fn user_config_snapshot(&self) -> Option<CloudLlmConfig> {
        self.user.read().clone()
    }

    pub fn set_user_config(&self, cfg: Option<CloudLlmConfig>) {
        *self.user.write() = cfg;
    }

    #[must_use]
    pub fn openai_hard_blocked(&self) -> bool {
        *self.openai_hard_blocked.read()
    }

    pub fn set_openai_hard_blocked(&self, v: bool) {
        *self.openai_hard_blocked.write() = v;
    }

    #[must_use]
    pub fn auto_remote_llm_enabled(&self) -> bool {
        *self.auto_remote_llm.read()
    }

    pub fn set_auto_remote_llm_enabled(&self, v: bool) {
        *self.auto_remote_llm.write() = v;
    }

    #[must_use]
    pub fn network_acknowledged(&self) -> bool {
        *self.network_ack.read()
    }

    pub fn set_network_acknowledged(&self, v: bool) {
        *self.network_ack.write() = v;
    }
}

impl CloudLlmConfig {
    pub async fn probe_chat_minimal(&self) -> Result<()> {
        let c = OpenAiCompatLlmClient::new(self.clone());
        let _ = c.generate("", "ok").await?;
        Ok(())
    }
}
