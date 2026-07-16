//! HTTP sidecar configuration: enabled via environment variables, works with the role pack's `plugin_backends.* = remote`.

use std::time::Duration;

/// Generic single JSON-RPC endpoint (shared by memory / emotion / event / prompt, etc.).
#[derive(Debug, Clone)]
pub struct RemotePluginHttpConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub bearer_token: Option<String>,
}

impl RemotePluginHttpConfig {
    /// Connect-phase timeout: roughly 1/4 of the total timeout, clamped to 500ms–15s, to avoid the TCP handshake hanging for a long time.
    #[must_use]
    pub fn connect_timeout(&self) -> Duration {
        let ms = self.timeout.as_millis() as u64;
        let quarter = ms / 4;
        Duration::from_millis(quarter.clamp(500, 15_000))
    }

    /// `OCLIVE_REMOTE_PLUGIN_URL`: when non-empty, enables the remote plugin HTTP client (works with the remote backends for `memory`/`emotion`/`event`/`prompt`).
    #[must_use]
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

    /// `OCLIVE_REMOTE_LLM_URL`: when non-empty, this endpoint is used when `plugin_backends.llm = remote` (JSON-RPC `llm.generate` / `llm.generate_tag`).
    #[must_use]
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

    /// `OCLIVE_COMPLEX_EMOTION_URL`: complex-emotion Remote sidecar (`complex_emotion.resolve_turn`).
    /// Backward-compatible with the old name `OCLIVE_REMOTE_COMPLEX_EMOTION_URL`.
    #[must_use]
    pub fn from_env_complex_emotion() -> Option<Self> {
        let endpoint = std::env::var("OCLIVE_COMPLEX_EMOTION_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                std::env::var("OCLIVE_REMOTE_COMPLEX_EMOTION_URL")
                    .ok()
                    .filter(|s| !s.trim().is_empty())
            })?;
        let t = endpoint.trim();
        if t.is_empty() {
            return None;
        }
        let timeout_ms = std::env::var("OCLIVE_COMPLEX_EMOTION_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(8_000);
        let bearer_token = std::env::var("OCLIVE_COMPLEX_EMOTION_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Some(Self {
            endpoint: t.to_string(),
            timeout: Duration::from_millis(timeout_ms.clamp(500, 120_000)),
            bearer_token,
        })
    }

    /// JSON-RPC root URL obtained after a directory plugin lazily starts (shares timeout / auth env var name prefix `OCLIVE_DIRECTORY_*` with env sidecars).
    pub fn for_directory_plugin_rpc(endpoint: impl Into<String>, is_llm: bool) -> Self {
        let endpoint = endpoint.into();
        let timeout = Self::directory_plugin_rpc_timeout_for_method("", is_llm);
        let bearer_token = std::env::var("OCLIVE_DIRECTORY_PLUGIN_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Self {
            endpoint,
            timeout,
            bearer_token,
        }
    }

    /// Per-method timeout for local directory-plugin JSON-RPC (generic default; plugins may override via manifest `rpcTimeoutsMs`).
    #[must_use]
    pub fn directory_plugin_rpc_timeout_for_method(_method: &str, is_llm: bool) -> Duration {
        if is_llm {
            let timeout_ms = std::env::var("OCLIVE_DIRECTORY_LLM_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(120_000);
            return Duration::from_millis(timeout_ms.clamp(1_000, 600_000));
        }
        let env_ms = std::env::var("OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(8_000);
        Duration::from_millis(env_ms.clamp(500, 900_000))
    }

    /// `OCLIVE_REMOTE_AGENT_URL`: agent sidecar JSON-RPC (`agent.process`).
    /// When unset, falls back to `OCLIVE_REMOTE_PLUGIN_URL` (shared endpoint, method differs).
    #[must_use]
    pub fn from_env_agent() -> Option<Self> {
        let endpoint = std::env::var("OCLIVE_REMOTE_AGENT_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OCLIVE_REMOTE_PLUGIN_URL").ok())?;
        let t = endpoint.trim();
        if t.is_empty() {
            return None;
        }
        let timeout_ms = std::env::var("OCLIVE_REMOTE_AGENT_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                std::env::var("OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(120_000);
        let bearer_token = std::env::var("OCLIVE_REMOTE_AGENT_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::env::var("OCLIVE_REMOTE_PLUGIN_TOKEN")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            });
        Some(Self {
            endpoint: t.to_string(),
            timeout: Duration::from_millis(timeout_ms.clamp(1_000, 600_000)),
            bearer_token,
        })
    }

    /// Role pack `reply_post_processor.remote.url` endpoint.
    #[must_use]
    pub fn for_reply_post_processor_remote(url: &str, timeout_ms: Option<u32>) -> Option<Self> {
        let t = url.trim();
        if t.is_empty() {
            return None;
        }
        let ms = timeout_ms.unwrap_or(8_000).clamp(500, 120_000) as u64;
        Some(Self {
            endpoint: t.to_string(),
            timeout: Duration::from_millis(ms),
            bearer_token: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn directory_plugin_timeout_defaults_to_eight_seconds() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS");
        let d =
            RemotePluginHttpConfig::directory_plugin_rpc_timeout_for_method("voice.warm", false);
        assert_eq!(d.as_millis(), 8_000);
    }

    #[test]
    fn directory_plugin_timeout_honors_env() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var("OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS", "600000");
        let d =
            RemotePluginHttpConfig::directory_plugin_rpc_timeout_for_method("voice.speak", false);
        assert_eq!(d.as_millis(), 600_000);
        std::env::remove_var("OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS");
    }

    #[test]
    fn plugin_timeout_is_clamped() {
        let _guard = ENV_LOCK.lock().expect("env lock");
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
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var("OCLIVE_REMOTE_LLM_URL", "http://127.0.0.1:9999/rpc");
        std::env::set_var("OCLIVE_REMOTE_LLM_TOKEN", "   ");
        let cfg = RemotePluginHttpConfig::from_env_llm().expect("cfg");
        assert!(cfg.bearer_token.is_none());
        std::env::remove_var("OCLIVE_REMOTE_LLM_URL");
        std::env::remove_var("OCLIVE_REMOTE_LLM_TOKEN");
    }
}
