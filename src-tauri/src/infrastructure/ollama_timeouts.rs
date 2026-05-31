//! Ollama HTTP per-request timeout (seconds); shares the configurable default with [`super::OllamaClient`].

use std::time::Duration;

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(default)
}

/// `OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS` (default `120`): `reqwest` timeout for each `OllamaClient` POST.
#[must_use]
pub fn http_client_timeout() -> Duration {
    Duration::from_secs(parse_env_u64("OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS", 120))
}
