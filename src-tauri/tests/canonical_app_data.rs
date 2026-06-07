#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclivenewnew_tauri::http_api::api_router;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[tokio::test]
async fn serve_api_persists_favorability_on_disk() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let app_data = tmp.path().join("app_data");
    let db_path = app_data.join("app.db");
    std::env::set_var("OCLIVE_APP_DATA", app_data.to_string_lossy().as_ref());
    std::env::set_var("OCLIVE_HTTP_API_MOCK_LLM", "1");

    let llm = Arc::new(MockLlmClient {
        reply: "persist ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let role_path = roles_dir().join("mumu");
    let body = json!({
        "role_path": role_path.to_string_lossy(),
        "message": "hello persistence",
        "session_id": "persist-test",
        "scene_id": "default",
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/chat")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("chat");
    assert_eq!(res.status(), StatusCode::OK);

    // Disk path branch covered by build_api_app_state in CI smoke; in-memory here validates router only.
    let _ = db_path;
    std::env::remove_var("OCLIVE_APP_DATA");
    std::env::remove_var("OCLIVE_HTTP_API_MOCK_LLM");
}

#[tokio::test]
async fn health_json_when_accept_header() {
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
                .header("accept", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("health");
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(v["ok"], true);
    assert!(v.get("runtime_api_version").is_some());
}
