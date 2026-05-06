//! `GET /health`、`GET /health?verbose=true`、`GET /health/db`（`kernel-http-api`）。

use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use http_body_util::BodyExt;
use oclive_kernel_runtime::http_api::api_router;
use oclive_kernel_runtime::infrastructure::llm::{LlmClient, MockLlmClient};
use oclive_kernel_runtime::state::KernelAppState;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn mock_llm() -> Arc<dyn LlmClient> {
    Arc::new(MockLlmClient {
        reply: "health_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_plain_is_text_ok() {
    let roles = workspace_roles_dir();
    let state = Arc::new(
        KernelAppState::new_in_memory_with_llm(mock_llm(), &roles)
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
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_verbose_json_has_checks() {
    let roles = workspace_roles_dir();
    let state = Arc::new(
        KernelAppState::new_in_memory_with_llm(mock_llm(), &roles)
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health?verbose=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["checks"]["db"], "ok");
    assert_eq!(v["checks"]["roles"], "ok");
    assert_eq!(v["checks"]["disk_space"], "ok");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_db_plain_ok() {
    let roles = workspace_roles_dir();
    let state = Arc::new(
        KernelAppState::new_in_memory_with_llm(mock_llm(), &roles)
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/health/db")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}
