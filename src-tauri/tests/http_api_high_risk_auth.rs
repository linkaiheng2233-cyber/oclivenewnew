//! HTTP `/high_risk/*` routes require `OCLIVE_BRIDGE_TOKEN` when set (same as `/bridge/dispatch`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclivenewnew_tauri::http_api::api_router;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::state::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

const TOKEN_HEADER: &str = "x-oclive-bridge-token";

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

async fn test_state() -> Arc<AppState> {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state"),
    )
}

/// Single test avoids parallel `OCLIVE_BRIDGE_TOKEN` env races.
#[tokio::test]
async fn high_risk_routes_bridge_token_auth_matrix() {
    std::env::remove_var("OCLIVE_BRIDGE_TOKEN");
    let app = api_router(test_state().await);
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/high_risk/grants")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK, "open when token unset");

    std::env::set_var("OCLIVE_BRIDGE_TOKEN", "test-secret-token");
    for (method, uri, body) in [
        ("GET", "/high_risk/grants", None),
        (
            "POST",
            "/high_risk/grant",
            Some(json!({ "kind": "process:spawn", "id": "plug.test" })),
        ),
        (
            "POST",
            "/high_risk/revoke",
            Some(json!({ "kind": "process:spawn", "id": "plug.test" })),
        ),
    ] {
        let mut req = Request::builder().method(method).uri(uri);
        let body = if let Some(v) = body {
            req = req.header("content-type", "application/json");
            Body::from(serde_json::to_vec(&v).unwrap())
        } else {
            Body::empty()
        };
        let res = app
            .clone()
            .oneshot(req.body(body).unwrap())
            .await
            .expect("oneshot");
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} without token"
        );
        let v = response_json(res).await;
        assert_eq!(v["error"]["code"], "INVALID_PARAMETER");
    }

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/high_risk/grants")
                .header(TOKEN_HEADER, "test-secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK, "list with valid token");

    let grant_body = json!({ "kind": "process:spawn", "id": "plug.auth_test" });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/high_risk/grant")
                .header("content-type", "application/json")
                .header(TOKEN_HEADER, "test-secret-token")
                .body(Body::from(serde_json::to_vec(&grant_body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(res.status(), StatusCode::OK);
    let v = response_json(res).await;
    assert_eq!(v["ok"], true);

    std::env::remove_var("OCLIVE_BRIDGE_TOKEN");
}
