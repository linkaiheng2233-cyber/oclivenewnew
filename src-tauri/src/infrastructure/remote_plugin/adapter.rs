//! Shared Remote HTTP client construction and remote-failure → builtin graceful degradation template.

use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
        let http = RemoteHttpClientBlocking::new(
            http_client,
            cfg,
            high_risk_grants,
            network_grant_id,
        );
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
