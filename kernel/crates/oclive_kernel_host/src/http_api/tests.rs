use super::{api_error, is_allowed_api_origin, kernel_http_error, validate_api_auth_configuration};
use crate::error::http_chat_codes;
use crate::models::role::PersonalitySource as Ps;
use axum::Json;

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
