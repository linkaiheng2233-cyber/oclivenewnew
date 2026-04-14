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
    /// 连接阶段超时：约为总超时的 1/4，夹在 500ms～15s，避免 TCP 握手长时间挂死。
    pub fn connect_timeout(&self) -> Duration {
        let ms = self.timeout.as_millis() as u64;
        let quarter = ms / 4;
        Duration::from_millis(quarter.max(500).min(15_000))
    }

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

    /// 目录插件懒启动后得到的 JSON-RPC 根 URL（与 env 侧车共用超时 / 鉴权环境变量名前缀 `OCLIVE_DIRECTORY_*`）。
    pub fn for_directory_plugin_rpc(endpoint: impl Into<String>, is_llm: bool) -> Self {
        let endpoint = endpoint.into();
        let timeout_ms = if is_llm {
            std::env::var("OCLIVE_DIRECTORY_LLM_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(120_000)
        } else {
            std::env::var("OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(8_000)
        };
        let (lo, hi) = if is_llm {
            (1_000u64, 600_000u64)
        } else {
            (500u64, 120_000u64)
        };
        let bearer_token = std::env::var("OCLIVE_DIRECTORY_PLUGIN_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Self {
            endpoint,
            timeout: Duration::from_millis(timeout_ms.clamp(lo, hi)),
            bearer_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_timeout_is_clamped() {
        std::env::set_var("OCLIVE_REMOTE_PLUGIN_URL", "http://127.0.0.1:9999/rpc");
        std::env::set_var("OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS", "1");
        let cfg = RemotePluginHttpConfig::from_env_plugin().expect("cfg");
        assert_eq!(cfg.timeout.as_millis(), 500);
        assert_eq!(cfg.connect_timeout().as_millis(), 500);
        std::env::remove_var("OCLIVE_REMOTE_PLUGIN_URL");
        std::env::remove_var("OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS");
    }

    #[test]
    fn connect_timeout_caps_at_15s() {
        let cfg = RemotePluginHttpConfig {
            endpoint: "http://127.0.0.1:9/rpc".into(),
            timeout: Duration::from_millis(120_000),
            bearer_token: None,
        };
        assert_eq!(cfg.connect_timeout().as_millis(), 15_000);
    }

    #[test]
    fn connect_timeout_floors_when_total_timeout_small() {
        let cfg = RemotePluginHttpConfig {
            endpoint: "http://127.0.0.1:9/rpc".into(),
            timeout: Duration::from_millis(500),
            bearer_token: None,
        };
        assert_eq!(cfg.connect_timeout().as_millis(), 500);
    }

    #[test]
    fn llm_token_trims_empty_to_none() {
        std::env::set_var("OCLIVE_REMOTE_LLM_URL", "http://127.0.0.1:9999/rpc");
        std::env::set_var("OCLIVE_REMOTE_LLM_TOKEN", "   ");
        let cfg = RemotePluginHttpConfig::from_env_llm().expect("cfg");
        assert!(cfg.bearer_token.is_none());
        std::env::remove_var("OCLIVE_REMOTE_LLM_URL");
        std::env::remove_var("OCLIVE_REMOTE_LLM_TOKEN");
    }
}
