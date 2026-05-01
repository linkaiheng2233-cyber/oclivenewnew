//! Ollama HTTP 单次请求超时（秒），与可选「直连辅助」路径共用同一套可配置默认值。
//!
//! - 主路径 [`super::OllamaClient`] 使用 [`http_client_timeout`].
//! - `utils::ollama::ollama_generate` 使用 [`legacy_utils_call_timeout`]（默认更短，避免辅助脚本拖死）。

use std::time::Duration;

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

/// `OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS`（默认 `120`）：`OllamaClient` 每次 POST 的 `reqwest` 超时。
pub fn http_client_timeout() -> Duration {
    Duration::from_secs(parse_env_u64("OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS", 120))
}

/// `OCLIVE_OLLAMA_LEGACY_UTILS_TIMEOUT_SECS`（默认 `30`）：`utils/ollama.rs` 单次 `generate` 外包 `tokio::timeout`。
pub fn legacy_utils_call_timeout() -> Duration {
    Duration::from_secs(parse_env_u64("OCLIVE_OLLAMA_LEGACY_UTILS_TIMEOUT_SECS", 30))
}
