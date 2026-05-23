//! 统一 Remote JSON-RPC HTTP 传输：reqwest 客户端构建、网络授权与 `jsonrpc` 调用。

use super::config::RemotePluginHttpConfig;
use super::jsonrpc::{self, RemoteRpcChannel};
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use serde_json::Value;
use std::sync::Arc;

/// 插件侧车（`OCLIVE_REMOTE_PLUGIN_URL`）共用端点 — sync API over async reqwest。
pub struct RemoteHttpClientBlocking {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
    grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteHttpClientBlocking {
    /// # Errors
    ///
    /// Returns [`Err`] when `reqwest` client construction fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()?;
        Ok(Self {
            client,
            cfg,
            grants,
            network_grant_id,
        })
    }

    #[must_use]
    pub fn config(&self) -> &RemotePluginHttpConfig {
        &self.cfg
    }

    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.cfg.endpoint.as_str()
    }

    /// Grant store + id for [`jsonrpc`] network checks (`None` = directory localhost RPC).
    #[must_use]
    pub fn network_grant(&self) -> Option<(&HighRiskGrantStore, &str)> {
        self.network_grant_id
            .as_deref()
            .map(|id| (self.grants.as_ref(), id))
    }

    /// # Errors
    ///
    /// Propagates network grant denial and JSON-RPC transport/parse failures as [`AppError`].
    pub fn call(
        &self,
        channel: RemoteRpcChannel,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        jsonrpc::call_blocking(
            channel,
            &self.client,
            &self.cfg.endpoint,
            method,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
        )
    }

    /// JSON-RPC on plugin channel (`memory.*`, `emotion.*`, …).
    ///
    /// # Errors
    ///
    /// Same as [`Self::call`] with [`RemoteRpcChannel::Plugin`].
    pub fn call_plugin(&self, method: &str, params: Value) -> Result<Value> {
        self.call(RemoteRpcChannel::Plugin, method, params)
    }

    /// 与 [`Self::call_plugin`] 相同，但远端/解析失败时返回 `Ok(None)`（仍传播未授权）。
    ///
    /// # Errors
    ///
    /// Returns [`Err`] only for [`AppError::HighRiskCapabilityNotGranted`].
    pub fn call_plugin_soft(&self, method: &str, params: Value) -> Result<Option<Value>> {
        match self.call_plugin(method, params) {
            Ok(v) => Ok(Some(v)),
            Err(e @ AppError::HighRiskCapabilityNotGranted { .. }) => Err(e),
            Err(_) => Ok(None),
        }
    }
}

/// 主对话 LLM 侧车（`OCLIVE_REMOTE_LLM_URL`）— **async** 客户端。
pub struct RemoteHttpClientAsync {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
    grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteHttpClientAsync {
    #[must_use]
    pub fn config(&self) -> &RemotePluginHttpConfig {
        &self.cfg
    }

    /// # Errors
    ///
    /// Returns [`Err`] when `reqwest` client construction fails.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()?;
        Ok(Self {
            client,
            cfg,
            grants,
            network_grant_id,
        })
    }

    /// Grant store + id for [`jsonrpc`] network checks.
    #[must_use]
    pub fn network_grant(&self) -> Option<(&HighRiskGrantStore, &str)> {
        self.network_grant_id
            .as_deref()
            .map(|id| (self.grants.as_ref(), id))
    }

    /// # Errors
    ///
    /// Propagates network grant denial and JSON-RPC transport/parse failures as [`AppError`].
    pub async fn call(
        &self,
        channel: RemoteRpcChannel,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        jsonrpc::call_async(
            channel,
            &self.client,
            &self.cfg.endpoint,
            method,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
        )
        .await
    }

    /// # Errors
    ///
    /// Same as [`Self::call`] with [`RemoteRpcChannel::Llm`].
    pub async fn call_llm(&self, method: &str, params: Value) -> Result<Value> {
        self.call(RemoteRpcChannel::Llm, method, params).await
    }

    /// # Errors
    ///
    /// Same as [`Self::call`] with [`RemoteRpcChannel::Plugin`].
    pub async fn call_plugin(&self, method: &str, params: Value) -> Result<Value> {
        self.call(RemoteRpcChannel::Plugin, method, params).await
    }
}
