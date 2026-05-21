//! AB2：侧车 JSON-RPC 错误与 `KernelErrorBody` 分层边界。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_runtime::{AppError, KernelErrorBody};
use oclive_validation::{
    assert_layers_do_not_overlap, validate_jsonrpc_error_response, validate_kernel_error_body,
};

#[test]
fn sidecar_jsonrpc_error_uses_integer_code() {
    let rpc = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32603, "message": "internal_error" }
    });
    validate_jsonrpc_error_response(&rpc).expect("valid jsonrpc error");
}

#[test]
fn kernel_error_body_from_app_error_is_string_code() {
    let app = AppError::OllamaError(
        "jsonrpc error method=emotion.analyze code=-32603(internal_error) message=fail".into(),
    );
    let body = app.kernel_error_body();
    validate_kernel_error_body(&serde_json::to_value(&body).unwrap()).expect("kernel shape");
    assert_eq!(body.code, "LLM_ERROR");
    assert!(
        body.code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_'),
        "kernel code must be SCREAMING_SNAKE_CASE, got {}",
        body.code
    );
}

#[test]
fn kernel_http_wrapper_never_uses_integer_code_field() {
    let body = KernelErrorBody {
        code: "REMOTE_SERVICE_UNAVAILABLE".into(),
        message: "sidecar down".into(),
        hint: None,
    };
    let v = serde_json::to_value(&body).unwrap();
    assert!(v.get("code").unwrap().is_string());
    assert!(v.get("code").unwrap().as_i64().is_none());
}

#[test]
fn layers_do_not_overlap_in_same_payload_shapes() {
    let rpc = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32010, "message": "plugin_timeout" }
    });
    let kernel = serde_json::json!({
        "code": "LLM_ERROR",
        "message": "wrapped sidecar failure"
    });
    assert_layers_do_not_overlap(&rpc, &kernel).expect("distinct layers");
}
