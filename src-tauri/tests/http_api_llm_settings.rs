//! `--api` HTTP LLM settings route integration tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclivenewnew_tauri::http_api::api_router;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test]
async fn http_api_llm_user_settings_get_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/llm/user_settings?role_id=mumu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert!(v.get("provider").is_some());
    assert!(v.get("effectiveModel").is_some());
    assert!(v.get("ollamaBaseUrl").is_some());
}

#[tokio::test]
async fn http_api_llm_session_model_post_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(Arc::clone(&state));
    let body = json!({
        "roleId": "mumu",
        "model": "llama3.2:latest",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/session_model")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["role_id"], "mumu");
}

#[tokio::test]
async fn http_api_llm_user_settings_post_local_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "roleId": "mumu",
        "provider": "local",
        "ollamaBaseUrl": "http://127.0.0.1:11434",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/llm/user_settings")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["role_id"], "mumu");
}

#[tokio::test]
async fn http_api_llm_ollama_models_get_ok_or_unreachable() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/llm/ollama_models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    // Ollama may be offline in CI; route should still respond (500 if unreachable).
    assert!(
        res.status() == StatusCode::OK || res.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "unexpected status {}",
        res.status()
    );
}
