use super::{api_error, kernel_http_error};
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
