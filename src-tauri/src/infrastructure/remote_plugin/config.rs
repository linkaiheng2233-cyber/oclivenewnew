//! HTTP 侧车配置：通过环境变量启用，与角色包 `plugin_backends.* = remote` 配合。

use std::time::Duration;

/// 通用 JSON-RPC 单端点（记忆 / 情绪 / 事件 / Prompt 等共用）。
#[derive(Debug, Clone)]
pub struct RemotePluginHttpConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub bearer_token: Option<String>,
}

impl RemotePluginHttpConfig {
    /// `OCLIVE_REMOTE_PLUGIN_URL`：非空则启用远程插件 HTTP 客户端（与 `memory`/`emotion`/`event`/`prompt` 的 remote 配合）。
    pub fn from_env_plugin() -> Option<Self> {
        let endpoint = std::env::var("OCLIVE_REMOTE_PLUGIN_URL").ok()?;
        let t = endpoint.trim();
        if t.is_empty() {
            return None;
        }
        let timeout_ms = std::env::var("OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(8_000);
        let bearer_token = std::env::var("OCLIVE_REMOTE_PLUGIN_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Some(Self {
            endpoint: t.to_string(),
            timeout: Duration::from_millis(timeout_ms.clamp(500, 120_000)),
            bearer_token,
        })
    }

    /// `OCLIVE_REMOTE_LLM_URL`：非空则 `plugin_backends.llm = remote` 时走该端点（JSON-RPC `llm.generate` / `llm.generate_tag`）。
    pub fn from_env_llm() -> Option<Self> {
        let endpoint = std::env::var("OCLIVE_REMOTE_LLM_URL").ok()?;
        let t = endpoint.trim();
        if t.is_empty() {
            return None;
        }
        let timeout_ms = std::env::var("OCLIVE_REMOTE_LLM_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(120_000);
        let bearer_token = std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Some(Self {
            endpoint: t.to_string(),
            timeout: Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
            bearer_token,
        })
    }
}
