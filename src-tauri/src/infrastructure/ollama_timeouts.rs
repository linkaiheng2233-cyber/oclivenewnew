//! Ollama HTTP 单次请求超时（秒），与 [`super::OllamaClient`] 共用可配置默认值。

use std::time::Duration;

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

/// `OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS`（默认 `120`）：`OllamaClient` 每次 POST 的 `reqwest` 超时。
#[must_use]
pub fn http_client_timeout() -> Duration {
    Duration::from_secs(parse_env_u64("OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS", 120))
}
