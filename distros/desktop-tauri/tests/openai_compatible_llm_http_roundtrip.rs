//! OpenAI-compatible LLM client E2E against mock `POST /v1/chat/completions`.
//!
//! Covers direct `OpenAiCompatibleLlm::from_env` (K-LLM-01a) and the
//! `BackendRegistry` → `llm_remote_backend` Remote path (K-LLM-01b).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    body::Body,
    extract::Json,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use oclive_kernel_host::domain::ports::LlmClient;
use oclive_kernel_host::infrastructure::backend_registry::BackendRegistry;
use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclive_kernel_host::infrastructure::openai_compatible_llm::OpenAiCompatibleLlm;
use oclive_kernel_host::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_types::models::plugin_backends::LlmBackend;
use oclive_validation::NETWORK_GRANT_REMOTE_LLM;
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;

async fn mock_chat_completions(headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        auth == "Bearer openai-compat-test-token",
        "expected Bearer token, got {auth:?}"
    );
    let prompt = body
        .pointer("/messages/0/content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = if prompt.contains("tag") {
        "neutral"
    } else {
        "openai-compat-ok"
    };
    if body.get("stream").and_then(Value::as_bool).unwrap_or(false) {
        let first = &content[..content.len() / 2];
        let second = &content[content.len() / 2..];
        let sse = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({"choices":[{"delta":{"content": first}}]}),
            json!({"choices":[{"delta":{"content": second}}]}),
        );
        return Response::builder()
            .header("content-type", "text/event-stream")
            .body(Body::from(sse))
            .unwrap();
    }
    Json(json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": "stop"
        }]
    }))
    .into_response()
}

#[tokio::test(flavor = "multi_thread")]
async fn openai_compatible_llm_generate_via_chat_completions() {
    let app = Router::new().route("/v1/chat/completions", post(mock_chat_completions));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Isolate from parallel tests: prefer OCLIVE_* keys; clear OpenAI aliases for this process slice.
    std::env::set_var("OCLIVE_REMOTE_LLM_URL", format!("http://{addr}"));
    std::env::set_var("OCLIVE_REMOTE_LLM_TOKEN", "openai-compat-test-token");
    std::env::set_var("OCLIVE_REMOTE_LLM_TIMEOUT_MS", "5000");
    std::env::remove_var("OPENAI_API_BASE");
    std::env::remove_var("OPENAI_BASE_URL");
    std::env::remove_var("OPENAI_API_KEY");

    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
    grants.grant_network(NETWORK_GRANT_REMOTE_LLM).unwrap();

    let client = OpenAiCompatibleLlm::from_env(reqwest::Client::new(), grants)
        .expect("OpenAiCompatibleLlm::from_env with mock URL");
    assert!(
        client.endpoint().ends_with("/v1/chat/completions"),
        "endpoint should normalize to chat/completions, got {}",
        client.endpoint()
    );

    let reply = client.generate("test-model", "hello prompt").await.unwrap();
    assert_eq!(reply, "openai-compat-ok");
    let tag = client
        .generate_tag("test-model", "tag prompt")
        .await
        .unwrap();
    assert_eq!(tag, "neutral");
    let chunks = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let chunks_for_sink = Arc::clone(&chunks);
    let streamed = client
        .generate_stream(
            "test-model",
            "hello prompt",
            Arc::new(move |token| {
                chunks_for_sink.lock().unwrap().push(token.to_string());
            }),
        )
        .await
        .unwrap();
    assert_eq!(streamed, "openai-compat-ok");
    assert_eq!(chunks.lock().unwrap().concat(), streamed);
    assert!(chunks.lock().unwrap().len() >= 2);

    std::env::remove_var("OCLIVE_REMOTE_LLM_URL");
    std::env::remove_var("OCLIVE_REMOTE_LLM_TOKEN");
    std::env::remove_var("OCLIVE_REMOTE_LLM_TIMEOUT_MS");
    server.abort();
}

/// Registry wiring: fresh [`BackendRegistry`] `llm_for(Remote)` → `llm_remote_backend`
/// (OpenAI-compat when `OCLIVE_LLM_CLOUD_API_STYLE` is not `oclive_jsonrpc`).
#[tokio::test(flavor = "multi_thread")]
async fn openai_compatible_llm_via_registry_remote() {
    let app = Router::new().route("/v1/chat/completions", post(mock_chat_completions));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Isolate process env for this slice (fresh registry avoids OnceLock contamination).
    std::env::set_var("OCLIVE_REMOTE_LLM_URL", format!("http://{addr}"));
    std::env::set_var("OCLIVE_REMOTE_LLM_TOKEN", "openai-compat-test-token");
    std::env::set_var("OCLIVE_REMOTE_LLM_TIMEOUT_MS", "5000");
    std::env::remove_var("OCLIVE_LLM_CLOUD_API_STYLE");
    std::env::remove_var("OPENAI_API_BASE");
    std::env::remove_var("OPENAI_BASE_URL");
    std::env::remove_var("OPENAI_API_KEY");

    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
    grants.grant_network(NETWORK_GRANT_REMOTE_LLM).unwrap();

    let default_llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: "registry-fallback-should-not-win".into(),
    });
    let registry = BackendRegistry::from_runtime(
        default_llm.clone(),
        None,
        dir.path().to_path_buf(),
        grants,
        new_remote_fallback_switch(true),
    );
    let client = registry.llm_for(LlmBackend::Remote);
    assert!(
        !Arc::ptr_eq(&client, &default_llm),
        "Remote path must not return the Ollama/default client when OpenAI-compat URL is set"
    );

    let reply = client.generate("test-model", "hello prompt").await.unwrap();
    assert_eq!(reply, "openai-compat-ok");
    let tag = client
        .generate_tag("test-model", "tag prompt")
        .await
        .unwrap();
    assert_eq!(tag, "neutral");

    std::env::remove_var("OCLIVE_REMOTE_LLM_URL");
    std::env::remove_var("OCLIVE_REMOTE_LLM_TOKEN");
    std::env::remove_var("OCLIVE_REMOTE_LLM_TIMEOUT_MS");
    server.abort();
}
