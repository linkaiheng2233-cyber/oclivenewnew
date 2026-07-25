//! Unified Remote JSON-RPC HTTP transport: reqwest client construction, network authorization, and `jsonrpc` calls.

use super::config::RemotePluginHttpConfig;
use super::jsonrpc::{self, RemoteRpcChannel};
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_contracts::LlmTokenSink;
use serde_json::Value;
use std::sync::Arc;

fn client_for_endpoint(
    client: Arc<reqwest::Client>,
    cfg: &RemotePluginHttpConfig,
) -> Arc<reqwest::Client> {
    if !cfg.is_loopback_endpoint() {
        return client;
    }
    let local = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap_or_else(|_| client.as_ref().clone());
    Arc::new(local)
}

/// Plugin sidecar (`OCLIVE_REMOTE_PLUGIN_URL`) shared endpoint — sync API over async reqwest.
pub struct RemoteHttpClientBlocking {
    client: Arc<reqwest::Client>,
    cfg: RemotePluginHttpConfig,
    grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteHttpClientBlocking {
    #[must_use]
    pub fn new(
        client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let client = client_for_endpoint(client, &cfg);
        Self {
            client,
            cfg,
            grants,
            network_grant_id,
        }
    }

    /// One-off RPC (e.g. for directory plugins): independent connection pool, does not use the [`BackendRegistry`](crate::domain::plugin_host::BackendRegistry) shared client.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] when `reqwest` client construction fails.
    pub fn new_standalone(
        cfg: RemotePluginHttpConfig,
        grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let mut builder = reqwest::Client::builder().connect_timeout(cfg.connect_timeout());
        if cfg.is_loopback_endpoint() {
            builder = builder.no_proxy();
        }
        let client = builder.build()?;
        Ok(Self::new(Arc::new(client), cfg, grants, network_grant_id))
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
    pub fn call(&self, channel: RemoteRpcChannel, method: &str, params: Value) -> Result<Value> {
        jsonrpc::call_blocking(
            channel,
            self.client.as_ref(),
            &self.cfg.endpoint,
            method,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
            self.cfg.timeout,
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

    /// Same as [`Self::call_plugin`], but returns `Ok(None)` on remote/parse failure (still propagates "not authorized").
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

/// Main-conversation LLM sidecar (`OCLIVE_REMOTE_LLM_URL`) — **async** client.
pub struct RemoteHttpClientAsync {
    client: Arc<reqwest::Client>,
    cfg: RemotePluginHttpConfig,
    grants: Arc<HighRiskGrantStore>,
    network_grant_id: Option<String>,
}

impl RemoteHttpClientAsync {
    #[must_use]
    pub fn config(&self) -> &RemotePluginHttpConfig {
        &self.cfg
    }

    #[must_use]
    pub fn new(
        client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
    ) -> Self {
        let client = client_for_endpoint(client, &cfg);
        Self {
            client,
            cfg,
            grants,
            network_grant_id,
        }
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
            self.client.as_ref(),
            &self.cfg.endpoint,
            method,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
            self.cfg.timeout,
        )
        .await
    }

    /// # Errors
    ///
    /// Same as [`Self::call`] with [`RemoteRpcChannel::Llm`].
    pub async fn call_llm(&self, method: &str, params: Value) -> Result<Value> {
        self.call(RemoteRpcChannel::Llm, method, params).await
    }

    /// Incremental JSON-RPC-over-NDJSON call on the LLM channel.
    ///
    /// # Errors
    ///
    /// Propagates authorization, transport, framing, and remote JSON-RPC errors.
    pub async fn call_llm_stream(
        &self,
        method: &str,
        params: Value,
        on_token: LlmTokenSink,
    ) -> Result<Value> {
        jsonrpc::call_async_stream(
            RemoteRpcChannel::Llm,
            self.client.as_ref(),
            &self.cfg.endpoint,
            method,
            params,
            self.cfg.bearer_token.as_deref(),
            self.network_grant(),
            self.cfg.timeout,
            on_token.as_ref(),
        )
        .await
    }

    /// # Errors
    ///
    /// Same as [`Self::call`] with [`RemoteRpcChannel::Plugin`].
    pub async fn call_plugin(&self, method: &str, params: Value) -> Result<Value> {
        self.call(RemoteRpcChannel::Plugin, method, params).await
    }
}
