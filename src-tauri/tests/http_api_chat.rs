//! `--api` HTTP 路由集成测试（`tower::ServiceExt::oneshot`，不监听端口）。

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclivenewnew_tauri::http_api::api_router;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::state::AppState;
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
async fn http_api_health_ok() {
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
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    assert_eq!(bytes.as_ref(), b"ok");
}

#[tokio::test]
async fn http_api_chat_empty_message_400() {
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
        "role_path": roles_dir().join("mumu").to_string_lossy(),
        "message": "   ",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let v = response_json(res).await;
    assert_eq!(v["error"]["code"], "empty_message");
}

#[tokio::test]
async fn http_api_chat_ok_includes_personality_source_and_reply() {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟回复".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let mumu = roles_dir().join("mumu");
    let body = json!({
        "role_path": mumu.to_string_lossy(),
        "message": "你好",
    });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["personality_source"], "vector");
    assert!(v["reply"].as_str().is_some());
}
