//! Remote LLM JSON-RPC client E2E against mock HTTP sidecar.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{extract::Json, routing::post, Router};
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use oclive_kernel_host::domain::ports::LlmClient;
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::remote_plugin::{RemoteLlmHttp, RemotePluginHttpConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

async fn mock_llm_handler(Json(body): Json<Value>) -> Json<Value> {
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = body.get("id").cloned().unwrap_or(json!(1));
    let text = if method == "llm.generate_tag" {
        "neutral"
    } else {
        "remote-llm-ok"
    };
    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "text": text }
    }))
}

#[tokio::test(flavor = "multi_thread")]
async fn remote_llm_http_generate_and_tag_via_jsonrpc() {
    let app = Router::new().route("/", post(mock_llm_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
    grants.grant_network(NETWORK_GRANT_REMOTE_LLM).unwrap();

    let cfg = RemotePluginHttpConfig {
        endpoint: format!("http://{addr}/"),
        timeout: Duration::from_secs(5),
        bearer_token: Some("test-token".into()),
    };
    let client = RemoteLlmHttp::new(
        Arc::new(reqwest::Client::new()),
        cfg,
        grants,
        Some(NETWORK_GRANT_REMOTE_LLM.to_string()),
    );

    let reply = client.generate("test-model", "hello prompt").await.unwrap();
    assert_eq!(reply, "remote-llm-ok");
    let tag = client.generate_tag("test-model", "hello prompt").await.unwrap();
    assert_eq!(tag, "neutral");

    server.abort();
}
