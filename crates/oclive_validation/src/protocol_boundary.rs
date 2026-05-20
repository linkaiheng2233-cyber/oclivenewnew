//! JSON-RPC 侧车错误与内核 [`KernelErrorBody`] 的分层校验（AB2）。
//!
//! 内核体字段为 `code` / `message` / `hint`（见 `oclive_kernel_runtime`），**不是** `detail`。

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolValidationError {
    NotObject,
    MissingField(&'static str),
    WrongType { field: &'static str, expected: &'static str },
    InvalidKernelCode(String),
    JsonRpcCodeNotInteger,
    LayerOverlap,
}

impl std::fmt::Display for ProtocolValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotObject => write!(f, "expected JSON object"),
            Self::MissingField(k) => write!(f, "missing field `{k}`"),
            Self::WrongType { field, expected } => {
                write!(f, "field `{field}` must be {expected}")
            }
            Self::InvalidKernelCode(c) => write!(f, "invalid kernel code `{c}`"),
            Self::JsonRpcCodeNotInteger => write!(f, "jsonrpc error.code must be integer"),
            Self::LayerOverlap => write!(f, "kernel code must not be JSON-RPC integer form"),
        }
    }
}

/// 校验 JSON-RPC 2.0 错误对象：`error.code` 为 **整数**，`error.message` 为字符串。
pub fn validate_jsonrpc_error_response(value: &Value) -> Result<(), ProtocolValidationError> {
    let obj = value.as_object().ok_or(ProtocolValidationError::NotObject)?;
    let ver = obj.get("jsonrpc").and_then(|v| v.as_str());
    if ver != Some("2.0") {
        return Err(ProtocolValidationError::WrongType {
            field: "jsonrpc",
            expected: "string \"2.0\"",
        });
    }
    let err = obj
        .get("error")
        .ok_or(ProtocolValidationError::MissingField("error"))?;
    let err_obj = err.as_object().ok_or(ProtocolValidationError::WrongType {
        field: "error",
        expected: "object",
    })?;
    let code = err_obj
        .get("code")
        .ok_or(ProtocolValidationError::MissingField("error.code"))?;
    if !code.is_i64() && !code.is_u64() {
        return Err(ProtocolValidationError::JsonRpcCodeNotInteger);
    }
    let msg = err_obj
        .get("message")
        .ok_or(ProtocolValidationError::MissingField("error.message"))?;
    if !msg.is_string() {
        return Err(ProtocolValidationError::WrongType {
            field: "error.message",
            expected: "string",
        });
    }
    Ok(())
}

fn is_screaming_snake(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
        && s.contains('_')
        || s.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
            && s.len() >= 3
            && !s.chars().any(|c| c.is_ascii_lowercase())
}

/// 校验内核 HTTP / Tauri 共用的 `KernelErrorBody` 形状：`code` 为 **SCREAMING_SNAKE_CASE** 字符串。
pub fn validate_kernel_error_body(value: &Value) -> Result<(), ProtocolValidationError> {
    let obj = value.as_object().ok_or(ProtocolValidationError::NotObject)?;
    let code = obj
        .get("code")
        .ok_or(ProtocolValidationError::MissingField("code"))?;
    if code.is_number() {
        return Err(ProtocolValidationError::LayerOverlap);
    }
    let code_str = code.as_str().ok_or(ProtocolValidationError::WrongType {
        field: "code",
        expected: "string (SCREAMING_SNAKE_CASE)",
    })?;
    if !is_screaming_snake(code_str) {
        return Err(ProtocolValidationError::InvalidKernelCode(code_str.to_string()));
    }
    let msg = obj
        .get("message")
        .ok_or(ProtocolValidationError::MissingField("message"))?;
    if !msg.is_string() {
        return Err(ProtocolValidationError::WrongType {
            field: "message",
            expected: "string",
        });
    }
    if let Some(h) = obj.get("hint") {
        if !h.is_string() && !h.is_null() {
            return Err(ProtocolValidationError::WrongType {
                field: "hint",
                expected: "string or omitted",
            });
        }
    }
    Ok(())
}

/// 侧车整数 `code` 与内核字符串 `code` 不得混用同一字段形态。
pub fn assert_layers_do_not_overlap(jsonrpc_err: &Value, kernel_body: &Value) -> Result<(), ProtocolValidationError> {
    validate_jsonrpc_error_response(jsonrpc_err)?;
    validate_kernel_error_body(kernel_body)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn jsonrpc_error_requires_int_code() {
        let ok = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": -32603, "message": "internal" }
        });
        validate_jsonrpc_error_response(&ok).unwrap();
        let bad = json!({
            "jsonrpc": "2.0",
            "error": { "code": "LLM_ERROR", "message": "x" }
        });
        assert!(validate_jsonrpc_error_response(&bad).is_err());
    }

    #[test]
    fn kernel_body_rejects_numeric_code() {
        let bad = json!({ "code": -32603, "message": "x" });
        assert_eq!(
            validate_kernel_error_body(&bad),
            Err(ProtocolValidationError::LayerOverlap)
        );
        let ok = json!({ "code": "LLM_ERROR", "message": "fail" });
        validate_kernel_error_body(&ok).unwrap();
    }
}
