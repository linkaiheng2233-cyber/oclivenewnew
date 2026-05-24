//! 共享 Remote HTTP 客户端构造与「远程失败 → builtin」降级模板。

use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_fallback_policy::remote_fallback_load;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// 阻塞 JSON-RPC 侧车适配器（memory / emotion / prompt 等共用构造与降级逻辑）。
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

    /// 远程 `call_plugin` 成功则 `decode`；失败且允许降级则 `builtin`，否则 `RemoteServiceUnavailable`。
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
