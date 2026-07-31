use super::{api_error, is_allowed_api_origin, kernel_http_error, validate_api_auth_configuration};
use crate::error::http_chat_codes;
use crate::infrastructure::MockLlmClient;
use crate::models::role::PersonalitySource as Ps;
use crate::state::AppState;
use axum::body::{to_bytes, Body};
use axum::http::Request;
use axum::Json;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn personality_source_json_matches_http_contract() {
    let v = serde_json::to_value(Ps::Vector).unwrap();
    let p = serde_json::to_value(Ps::Profile).unwrap();
    assert_eq!(v, "vector");
    assert_eq!(p, "profile");
}

#[test]
fn api_error_serializes_kernel_error_body() {
    let (_, Json(body)) = api_error(
        axum::http::StatusCode::BAD_REQUEST,
        kernel_http_error(
            http_chat_codes::INVALID_ROLE_PATH,
            "role_path is not a directory: /x",
            Some("请传入绝对路径".into()),
        ),
    );
    let v = serde_json::to_value(body).expect("serialize");
    assert_eq!(v["error"]["code"], "INVALID_ROLE_PATH");
    assert_eq!(v["error"]["message"], "role_path is not a directory: /x");
    assert_eq!(v["error"]["hint"], "请传入绝对路径");
}

#[test]
fn cors_origin_accepts_local_tools_and_rejects_public_websites() {
    for origin in [
        "http://localhost:1420",
        "http://127.0.0.1:5173",
        "http://[::1]:5175",
        "tauri://localhost",
        "https://tauri.localhost",
        "https://ocliveplugin.localhost",
        "ocliveplugin://localhost",
    ] {
        let value = origin
            .parse::<axum::http::HeaderValue>()
            .expect("origin header");
        assert!(is_allowed_api_origin(&value), "{origin}");
    }

    for origin in [
        "https://example.com",
        "https://evil.localhost.example.com",
        "file://localhost/tmp",
    ] {
        let value = origin
            .parse::<axum::http::HeaderValue>()
            .expect("origin header");
        assert!(!is_allowed_api_origin(&value), "{origin}");
    }
}

#[test]
fn api_server_auth_configuration_fails_closed() {
    assert!(validate_api_auth_configuration(Some("long-random-token"), false).is_ok());
    assert!(validate_api_auth_configuration(None, true).is_ok());
    assert!(validate_api_auth_configuration(None, false).is_err());
    assert!(validate_api_auth_configuration(Some("   "), false).is_err());
}

#[tokio::test]
async fn resource_transition_route_reaches_authoritative_kernel_state() {
    let roles = tempfile::tempdir().expect("roles tempdir");
    let state = Arc::new(
        AppState::new_in_memory_with_llm(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            roles.path(),
        )
        .await
        .expect("test app state"),
    );
    let request = Request::post("/resources/adapter/transition")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "adapter_id": "builtin.llm.llama_server",
                "operation": "suspend",
                "requested_by_adapter_id": "builtin.voice.cosyvoice2",
                "reason": "test"
            })
            .to_string(),
        ))
        .expect("request");
    let response = super::api_router(state)
        .oneshot(request)
        .await
        .expect("route response");
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("response body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("response json");
    assert_eq!(value["error"]["code"], "INVALID_PARAMETER");
    assert!(value["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("resource_transition_adapter_unregistered")));
}
