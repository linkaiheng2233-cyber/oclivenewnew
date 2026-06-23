//! Remote Agent JSON-RPC client E2E against mock HTTP sidecar.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{extract::Json, routing::post, Router};
use oclive_kernel_host::domain::agent::{AgentInput, AgentProvider};
use oclive_kernel_host::infrastructure::agent_mcp_bridge::AgentMcpBridge;
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::mcp_client::McpClient;
use oclive_kernel_host::infrastructure::remote_plugin::{AgentRpcProvider, RemotePluginHttpConfig};
use oclive_validation::NETWORK_GRANT_REMOTE_AGENT;
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

async fn mock_agent_handler(Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").cloned().unwrap_or(json!(1));
    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "handled": true,
            "reply": "remote-agent-ok"
        }
    }))
}

#[tokio::test(flavor = "multi_thread")]
async fn remote_agent_http_process_via_jsonrpc() {
    let app = Router::new().route("/", post(mock_agent_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
    grants.grant_network(NETWORK_GRANT_REMOTE_AGENT).unwrap();
    let mcp = Arc::new(McpClient::new(dir.path(), grants.clone()));
    let bridge = Arc::new(AgentMcpBridge::new(mcp));

    let cfg = RemotePluginHttpConfig {
        endpoint: format!("http://{addr}/"),
        timeout: Duration::from_secs(5),
        bearer_token: None,
    };
    let client = AgentRpcProvider::new(
        Arc::new(reqwest::Client::new()),
        cfg,
        Arc::new(AtomicBool::new(false)),
        grants,
        Some(NETWORK_GRANT_REMOTE_AGENT.to_string()),
        bridge,
    );

    let out = client
        .process(AgentInput {
            role_id: "r".into(),
            session_namespace: "r__sess__x".into(),
            message: "book a table".into(),
            model: "m".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(out.handled);
    assert_eq!(out.reply, "remote-agent-ok");

    server.abort();
}
