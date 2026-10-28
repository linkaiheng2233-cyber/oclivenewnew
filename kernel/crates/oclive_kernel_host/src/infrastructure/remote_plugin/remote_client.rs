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
) -> std::result::Result<Arc<reqwest::Client>, String> {
    if !cfg.is_loopback_endpoint() {
        return Ok(client);
    }
    let local = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("local RPC client: {error}"))?;
    Ok(Arc::new(local))
}

/// Plugin sidecar (`OCLIVE_REMOTE_PLUGIN_URL`) shared endpoint — sync API over async reqwest.
pub struct RemoteHttpClientBlocking {
    client: std::result::Result<Arc<reqwest::Client>, String>,
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
            builder = builder
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none());
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
        let client = self
            .client
            .as_ref()
            .map_err(|e| AppError::RemoteServiceUnavailable(e.clone()))?;
        jsonrpc::call_blocking(
            channel,
            client.as_ref(),
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
    client: std::result::Result<Arc<reqwest::Client>, String>,
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
        let client = self
            .client
            .as_ref()
            .map_err(|e| AppError::RemoteServiceUnavailable(e.clone()))?;
        jsonrpc::call_async(
            channel,
            client.as_ref(),
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
        let client = self
            .client
            .as_ref()
            .map_err(|e| AppError::RemoteServiceUnavailable(e.clone()))?;
        jsonrpc::call_async_stream(
            RemoteRpcChannel::Llm,
            client.as_ref(),
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

#[cfg(test)]
mod transport_boundary_tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn local_client_bypasses_proxy_and_does_not_follow_redirects() {
        let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_url = format!("http://{}/rpc", origin.local_addr().unwrap());
        let location = format!("http://{}/elsewhere", destination.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = origin.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut chunk = [0; 1024];
                let read = socket.read(&mut chunk).await.unwrap();
                assert!(read > 0, "request ended before its headers");
                request.extend_from_slice(&chunk[..read]);
                assert!(request.len() <= 16384);
            }
            socket.write_all(format!("HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
        });
        let inherited = reqwest::Client::builder()
            .proxy(reqwest::Proxy::all("http://127.0.0.1:9").unwrap())
            .build()
            .unwrap();
        let cfg = RemotePluginHttpConfig {
            endpoint: origin_url.clone(),
            timeout: Duration::from_secs(2),
            bearer_token: None,
        };
        let client = client_for_endpoint(Arc::new(inherited), &cfg).unwrap();
        let response = client
            .get(origin_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::FOUND);
        server.await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(50), destination.accept())
                .await
                .is_err()
        );
    }

    #[test]
    fn local_client_setup_failure_never_uses_the_inherited_client() {
        let fixture = tempfile::tempdir().unwrap();
        let client = RemoteHttpClientBlocking {
            client: Err("fixture setup failure".into()),
            cfg: RemotePluginHttpConfig {
                endpoint: "http://127.0.0.1:9/rpc".into(),
                timeout: Duration::from_millis(20),
                bearer_token: None,
            },
            grants: HighRiskGrantStore::load(fixture.path().to_path_buf(), false),
            network_grant_id: None,
        };
        assert!(matches!(
            client.call_plugin("test", serde_json::json!({})),
            Err(AppError::RemoteServiceUnavailable(_))
        ));
    }
}
