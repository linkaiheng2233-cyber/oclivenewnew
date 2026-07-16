//! Shared Remote HTTP client construction and remote-failure → builtin graceful degradation template.

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientAsync;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_COMPLEX_EMOTION_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

/// Decode a JSON-RPC result value into `T`.
pub fn decode_serde_value<T: DeserializeOwned>(v: Value, context: &str) -> Result<T> {
    serde_json::from_value(v).map_err(|e| serde_to_ollama(context, e))
}

/// Blocking JSON-RPC sidecar adapter (shared construction and fallback for memory / emotion / prompt, etc.).
pub struct RemotePluginAdapterBlocking {
    pub http: RemoteHttpClientBlocking,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl RemotePluginAdapterBlocking {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let http =
            RemoteHttpClientBlocking::new(http_client, cfg, high_risk_grants, network_grant_id);
        Self {
            http,
            remote_fallback_allowed,
        }
    }

    #[must_use]
    pub fn from_http(
        http: RemoteHttpClientBlocking,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            http,
            remote_fallback_allowed,
        }
    }

    /// On remote `call_plugin` success, `decode`; on failure with fallback allowed, `builtin`; else `RemoteServiceUnavailable`.
    pub fn call_with_builtin_fallback<T>(
        &self,
        method: &str,
        params: Value,
        decode: impl FnOnce(Value) -> Result<T>,
        builtin: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        match self.http.call_plugin(method, params) {
            Ok(v) => decode(v),
            Err(e) => {
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "{method} remote failed endpoint={} err={}; fallback=builtin",
                        self.http.endpoint(),
                        e
                    );
                    builtin()
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "{method} remote failed endpoint={} err={}",
                        self.http.endpoint(),
                        e
                    )))
                }
            }
        }
    }
}

/// Async JSON-RPC sidecar adapter (mirrors [`RemotePluginAdapterBlocking`] with async `call_plugin`).
pub struct RemotePluginAdapterAsync {
    pub http: RemoteHttpClientAsync,
    remote_fallback_allowed: Arc<AtomicBool>,
}

impl RemotePluginAdapterAsync {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let http = RemoteHttpClientAsync::new(http_client, cfg, high_risk_grants, network_grant_id);
        Self {
            http,
            remote_fallback_allowed,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn from_http(
        http: RemoteHttpClientAsync,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            http,
            remote_fallback_allowed,
        }
    }

    /// On remote `call_plugin` success, `decode`; on failure with fallback allowed, `builtin`; else `RemoteServiceUnavailable`.
    pub async fn call_with_async_builtin_fallback<T, F, Fut>(
        &self,
        method: &str,
        params: Value,
        decode: impl FnOnce(Value) -> Result<T>,
        builtin: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        match self.http.call_plugin(method, params).await {
            Ok(v) => decode(v),
            Err(e) => {
                if matches!(e, AppError::HighRiskCapabilityNotGranted { .. }) {
                    return Err(e);
                }
                if remote_fallback_load(&self.remote_fallback_allowed) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "{method} remote failed endpoint={} err={}; fallback=builtin",
                        self.http.config().endpoint,
                        e
                    );
                    builtin().await
                } else {
                    Err(AppError::RemoteServiceUnavailable(format!(
                        "{method} remote failed endpoint={} err={}",
                        self.http.config().endpoint,
                        e
                    )))
                }
            }
        }
    }
}

/// Shared blocking RPC for `complex_emotion.resolve_turn`.
pub fn invoke_turn_rpc(
    adapter: &RemotePluginAdapterBlocking,
    fallback: &BuiltinKeywordComplexEmotionProvider,
    input: &ComplexEmotionInput,
) -> Result<ComplexEmotionOutput> {
    let params = serde_json::to_value(input)
        .map_err(|e| serde_to_ollama("complex_emotion params json", e))?;
    adapter.call_with_builtin_fallback(
        METHOD_COMPLEX_EMOTION_RESOLVE_TURN,
        params,
        |v| {
            let mut out: ComplexEmotionOutput =
                decode_serde_value(v, "complex_emotion result decode")?;
            out.degraded_to_builtin = false;
            Ok(out)
        },
        || {
            let mut o = fallback.resolve_turn_inner(input);
            o.degraded_to_builtin = true;
            Ok(o)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
    use serde_json::json;
    use std::time::Duration;

    fn adapter_to_dead_endpoint(fallback_allowed: bool) -> RemotePluginAdapterBlocking {
        let cfg = RemotePluginHttpConfig {
            endpoint: "http://127.0.0.1:1/rpc".into(),
            timeout: Duration::from_millis(400),
            bearer_token: None,
        };
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(200))
            .build()
            .expect("reqwest client");
        RemotePluginAdapterBlocking::new(
            Arc::new(client),
            cfg,
            Arc::new(AtomicBool::new(fallback_allowed)),
            HighRiskGrantStore::load(std::env::temp_dir(), false),
            None,
        )
    }

    #[test]
    fn call_with_builtin_fallback_uses_builtin_when_remote_fails() {
        let adapter = adapter_to_dead_endpoint(true);
        let out = adapter
            .call_with_builtin_fallback(
                "test.method",
                json!({}),
                |_| Ok("remote".to_string()),
                || Ok("builtin".to_string()),
            )
            .expect("fallback ok");
        assert_eq!(out, "builtin");
    }

    #[test]
    fn call_with_builtin_fallback_errors_when_fallback_disabled() {
        let adapter = adapter_to_dead_endpoint(false);
        let err = adapter
            .call_with_builtin_fallback(
                "test.method",
                json!({}),
                |_| Ok("remote".to_string()),
                || Ok("builtin".to_string()),
            )
            .expect_err("must fail without fallback");
        assert!(matches!(err, AppError::RemoteServiceUnavailable(_)));
    }
}
