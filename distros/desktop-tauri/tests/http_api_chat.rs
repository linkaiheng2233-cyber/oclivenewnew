//! `--api` HTTP 路由集成测试（`tower::ServiceExt::oneshot`，不监听端口）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclivenewnew_tauri::http_api::api_router;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn response_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .expect("oneshot");
    let status = response.status();
    (status, response_json(response).await)
}

#[tokio::test]
async fn http_api_health_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
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
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let body = json!({
        "role_path": common::roles_dir().join("mumu").to_string_lossy(),
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
    assert_eq!(v["error"]["code"], "EMPTY_MESSAGE");
}

#[tokio::test]
async fn http_api_chat_ok_includes_personality_source_and_reply() {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟回复".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let mumu = common::roles_dir().join("mumu");
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

#[tokio::test]
async fn http_api_chat_with_session_id_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟回复".to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let mumu = common::roles_dir().join("mumu");
    let body = json!({
        "role_path": mumu.to_string_lossy(),
        "message": "你好",
        "session_id": "http-api-session-smoke",
        "scene_id": "vscode",
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
    assert_eq!(v["session_id"], "http-api-session-smoke");
    assert!(v["reply"].as_str().is_some());
}

#[tokio::test]
async fn http_api_adult_stage_five_route_roundtrip() {
    let llm = Arc::new(MockLlmClient {
        reply: r#"{"dialogue":"下一拍","narration":"她轻轻点头。","interaction_state":"active","next_beat_interval_ms":10}"#
            .to_string(),
    });
    let state = Arc::new(
        AppState::new_in_memory_with_llm(llm, common::roles_dir())
            .await
            .expect("state"),
    );
    let app = api_router(state);
    let gates = json!({
        "confirmed_adult": true,
        "global_enabled": true,
        "role_enabled": true,
        "interaction_active": true,
        "action": "continue",
    });
    let chat = json!({
        "role_id": "gentle-landlady",
        "scene_id": "default",
        "session_id": "http-adult-stage",
    });

    let (status, begun) = post_json(
        app.clone(),
        "/chat/adult-stage/begin",
        json!({
            "role_id": chat["role_id"],
            "scene_id": chat["scene_id"],
            "session_id": chat["session_id"],
            "adult": gates,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let generation_id = begun["generation_id"]
        .as_str()
        .expect("generation id")
        .to_string();
    assert_eq!(begun["next_sequence"], 0);

    let ownership = || {
        json!({
            "role_id": chat["role_id"],
            "scene_id": chat["scene_id"],
            "session_id": chat["session_id"],
            "generation_id": generation_id,
        })
    };
    let (status, staged) = post_json(
        app.clone(),
        "/chat/adult-stage/beat",
        json!({
            "role_id": chat["role_id"],
            "scene_id": chat["scene_id"],
            "session_id": chat["session_id"],
            "generation_id": generation_id,
            "sequence": 0,
            "adult": gates,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(staged["generation_id"], generation_id);
    assert_eq!(staged["sequence"], 0);
    assert_eq!(staged["response"]["adult_beat"]["dialogue"], "下一拍");
    assert_eq!(
        staged["response"]["adult_beat"]["interaction_state"],
        "active"
    );

    let (status, listed) = post_json(app.clone(), "/chat/adult-stage/list", ownership()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["active"], true);
    assert_eq!(listed["next_sequence"], 1);
    assert_eq!(listed["beats"].as_array().expect("beats").len(), 1);

    let (status, committed) = post_json(
        app.clone(),
        "/chat/adult-stage/commit",
        json!({
            "role_id": chat["role_id"],
            "scene_id": chat["scene_id"],
            "session_id": chat["session_id"],
            "generation_id": generation_id,
            "sequence": 0,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(committed["assistant_message_id"].as_str().is_some());
    let (status, listed) = post_json(app.clone(), "/chat/adult-stage/list", ownership()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(listed["beats"].as_array().expect("beats").is_empty());

    let (status, replacement) = post_json(
        app.clone(),
        "/chat/adult-stage/begin",
        json!({
            "role_id": chat["role_id"],
            "scene_id": chat["scene_id"],
            "session_id": chat["session_id"],
            "adult": gates,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let replacement_id = replacement["generation_id"]
        .as_str()
        .expect("replacement generation")
        .to_string();
    let replacement_ownership = json!({
        "role_id": chat["role_id"],
        "scene_id": chat["scene_id"],
        "session_id": chat["session_id"],
        "generation_id": replacement_id,
    });
    let (status, cancelled) = post_json(
        app.clone(),
        "/chat/adult-stage/cancel",
        replacement_ownership.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["ok"], true);
    let (status, listed) = post_json(app, "/chat/adult-stage/list", replacement_ownership).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["active"], false);
    assert!(listed["beats"].as_array().expect("beats").is_empty());
}
