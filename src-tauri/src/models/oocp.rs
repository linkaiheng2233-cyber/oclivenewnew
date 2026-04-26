//! OOCP (OClive Open Control Protocol) v0.1 消息类型。
//!
//! 传输无关的序列化类型；所有传输层（WS / HTTP / stdio）共用这些结构体。
//! 字段命名与 OOCP spec 一致（JSON camelCase）。

use serde::{Deserialize, Serialize};

// ── 顶层消息 ──────────────────────────────────────────────────────────────

/// 客户端发送的请求。
#[derive(Debug, Clone, Deserialize)]
pub struct OocpRequest {
    #[serde(rename = "type")]
    pub msg_type: String, // "request"
    /// 由客户端生成的唯一 id；u64 或 string 均可。
    pub id: serde_json::Value,
    pub method: String,
    pub params: serde_json::Value,
}

/// 服务端返回的成功响应。
#[derive(Debug, Clone, Serialize)]
pub struct OocpResponse {
    #[serde(rename = "type")]
    pub msg_type: &'static str, // "response"
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

/// 服务端主动推送的事件（无对应 request）。
#[derive(Debug, Clone, Serialize)]
pub struct OocpEvent {
    #[serde(rename = "type")]
    pub msg_type: &'static str, // "event"
    pub event: String,
    pub payload: serde_json::Value,
}

/// 服务端返回的错误（替代 response）。
#[derive(Debug, Clone, Serialize)]
pub struct OocpError {
    #[serde(rename = "type")]
    pub msg_type: &'static str, // "error"
    pub id: serde_json::Value,
    pub error: OocpErrorBody,
}

#[derive(Debug, Clone, Serialize)]
pub struct OocpErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub data: serde_json::Value,
}

/// 连接建立后服务端发送的首条消息。
#[derive(Debug, Clone, Serialize)]
pub struct OocpCapabilities {
    #[serde(rename = "type")]
    pub msg_type: &'static str, // "capabilities"
    pub version: &'static str,
    pub methods: Vec<&'static str>,
    pub events: Vec<&'static str>,
    pub limits: OocpLimits,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OocpLimits {
    pub max_concurrent_requests: u32,
    pub max_message_chars: u32,
}

// ── 错误码 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OocpErrorCode {
    UnsupportedMethod,
    InvalidParams,
    SessionNotFound,
    RoleNotFound,
    LlmFailure,
    Internal,
    AuthRequired,
    AuthFailed,
    RateLimited,
}

impl OocpErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedMethod => "UNSUPPORTED_METHOD",
            Self::InvalidParams => "INVALID_PARAMS",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::RoleNotFound => "ROLE_NOT_FOUND",
            Self::LlmFailure => "LLM_FAILURE",
            Self::Internal => "INTERNAL",
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::AuthFailed => "AUTH_FAILED",
            Self::RateLimited => "RATE_LIMITED",
        }
    }
}

// ── 当前支持的 v0.1 方法 ──────────────────────────────────────────────────

// Re-exported from oclive_core for single source of truth.
pub use oclive_core::capabilities::{OOCP_EVENTS, OOCP_METHODS, OOCP_VERSION};
