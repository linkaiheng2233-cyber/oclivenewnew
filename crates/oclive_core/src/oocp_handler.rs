//! OOCP v0.1 transport-agnostic handler (platform-independent).
//!
//! This module MUST NOT depend on `tauri` or OS-specific crates.

use crate::capabilities::{OOCP_EVENTS, OOCP_METHODS, OOCP_VERSION};
use crate::oocp::{
    OocpCapabilities, OocpError, OocpErrorBody, OocpErrorCode, OocpEvent, OocpLimits, OocpRequest,
    OocpResponse,
};
use serde_json::Value;

// ── helpers ────────────────────────────────────────────────────────────────

fn make_error(id: Value, code: OocpErrorCode, message: impl Into<String>) -> OocpError {
    OocpError {
        msg_type: "error",
        id,
        error: OocpErrorBody {
            code: code.as_str().to_string(),
            message: message.into(),
            data: Value::Null,
        },
    }
}

fn make_response(id: Value, result: Value) -> OocpResponse {
    OocpResponse {
        msg_type: "response",
        id,
        result,
    }
}

#[allow(dead_code)]
fn make_event(event: impl Into<String>, payload: Value) -> OocpEvent {
    OocpEvent {
        msg_type: "event",
        event: event.into(),
        payload,
    }
}

// ── capabilities ───────────────────────────────────────────────────────────

/// Build OOCP capabilities (handshake first frame).
pub fn get_capabilities(
    auth_required: bool,
    max_concurrent_requests: u32,
    max_message_chars: u32,
) -> OocpCapabilities {
    OocpCapabilities {
        msg_type: "capabilities",
        version: OOCP_VERSION,
        methods: OOCP_METHODS.to_vec(),
        events: OOCP_EVENTS.to_vec(),
        limits: OocpLimits {
            max_concurrent_requests,
            max_message_chars,
        },
        auth_required,
    }
}

// ── handled ────────────────────────────────────────────────────────────────

/// Result of handling one OOCP request.
pub enum OocpHandled {
    Response(OocpResponse),
    Error(OocpError),
    Capabilities(OocpCapabilities),
}

// ── dispatch ───────────────────────────────────────────────────────────────

/// Transport-agnostic OOCP request dispatcher.
///
/// `capabilities` is provided by the adapter/transport so that platform-specific
/// auth/limits can be reflected in the handshake.
pub async fn dispatch_oocp_request(
    req: OocpRequest,
    handler: &mut impl OocpMethodHandler,
    capabilities: &OocpCapabilities,
) -> OocpHandled {
    // 1) If client sends empty method, treat as capabilities request.
    if req.method.is_empty() {
        return OocpHandled::Capabilities(capabilities.clone());
    }

    // 2) Whitelist check.
    if !OOCP_METHODS.contains(&req.method.as_str()) {
        return OocpHandled::Error(make_error(
            req.id.clone(),
            OocpErrorCode::UnsupportedMethod,
            format!("方法 '{}' 未在 capabilities 白名单中或尚未实现", req.method),
        ));
    }

    // 3) Dispatch to method impl.
    let id = req.id.clone();
    match handle_method(req, handler).await {
        Ok(result) => OocpHandled::Response(make_response(id, result)),
        Err(e) => OocpHandled::Error(make_error(id, e.code, e.message)),
    }
}

// ── method routing ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct MethodError {
    pub code: OocpErrorCode,
    pub message: String,
}

impl MethodError {
    pub fn new(code: OocpErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub(crate) async fn handle_method(
    req: OocpRequest,
    handler: &mut impl OocpMethodHandler,
) -> Result<Value, MethodError> {
    match req.method.as_str() {
        // session
        "session.create" => {
            let role_id = get_str(&req.params, "role_id")?;
            let session_id = get_str_opt(&req.params, "session_id");
            let scene_id = get_str_opt(&req.params, "scene_id");
            handler
                .session_create(&role_id, session_id.as_deref(), scene_id.as_deref())
                .await
        }
        "session.destroy" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            handler.session_destroy(&session_ns).await
        }
        "session.get_state" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            handler.session_get_state(&session_ns).await
        }
        "session.switch_scene" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let scene_id = get_str(&req.params, "scene_id")?;
            handler.session_switch_scene(&session_ns, &scene_id).await
        }
        "session.switch_interaction_mode" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let mode = get_str(&req.params, "mode")?;
            handler
                .session_switch_interaction_mode(&session_ns, &mode)
                .await
        }
        "session.export_chat_logs" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let format = get_str(&req.params, "format")?;
            let path = get_str_opt(&req.params, "path");
            handler
                .session_export_chat_logs(&session_ns, &format, path.as_deref())
                .await
        }

        // chat
        "chat.send_message" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let user_message = get_str(&req.params, "user_message")?;
            let scene_id = get_str_opt(&req.params, "scene_id");
            handler
                .chat_send_message(&session_ns, &user_message, scene_id.as_deref())
                .await
        }
        "chat.generate_monologue" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let context = get_str_opt(&req.params, "context");
            handler
                .chat_generate_monologue(&session_ns, context.as_deref())
                .await
        }

        // role
        "role.list" => handler.role_list().await,
        "role.get_info" => {
            let role_id = get_str(&req.params, "role_id")?;
            let session_id = get_str_opt(&req.params, "session_id");
            let session_id = session_id.as_deref().map(str::trim).filter(|s| !s.is_empty());
            handler.role_get_info(&role_id, session_id).await
        }
        "role.set_remote_life" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let enabled = get_bool(&req.params, "enabled").unwrap_or(true);
            handler.role_set_remote_life(&session_ns, enabled).await
        }

        // time
        "time.get_state" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            handler.time_get_state(&session_ns).await
        }
        "time.jump" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let target_time_ms = get_i64_opt(&req.params, "target_time_ms");
            let preset = get_str_opt(&req.params, "preset");
            if target_time_ms.is_none()
                && preset
                    .as_ref()
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
            {
                return Err(MethodError {
                    code: OocpErrorCode::InvalidParams,
                    message: "需要 target_time_ms 或 preset（与 Tauri jump_time 一致）".into(),
                });
            }
            handler
                .time_jump(
                    &session_ns,
                    target_time_ms,
                    preset.as_deref().map(str::trim).filter(|s| !s.is_empty()),
                )
                .await
        }

        // agent
        "agent.call_mcp_tool" => {
            let server_id = get_str(&req.params, "server_id")?;
            let tool_name = get_str(&req.params, "tool_name")?;
            let arguments = req.params.get("arguments").cloned().unwrap_or(Value::Null);
            handler
                .agent_call_mcp_tool(&server_id, &tool_name, arguments)
                .await
        }

        other => Err(MethodError {
            code: OocpErrorCode::UnsupportedMethod,
            message: format!("未知方法: {}", other),
        }),
    }
}

// ── params helpers ─────────────────────────────────────────────────────────

fn get_str(params: &Value, key: &str) -> Result<String, MethodError> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| MethodError {
            code: OocpErrorCode::InvalidParams,
            message: format!("缺少必填参数 {}", key),
        })
}

fn get_str_opt(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_bool(params: &Value, key: &str) -> Option<bool> {
    params.get(key).and_then(|v| v.as_bool())
}

fn get_i64_opt(params: &Value, key: &str) -> Option<i64> {
    params.get(key).and_then(|v| v.as_i64())
}

// ── adapter trait ──────────────────────────────────────────────────────────

#[allow(async_fn_in_trait)]
pub trait OocpMethodHandler {
    // session
    async fn session_create(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
        scene_id: Option<&str>,
    ) -> Result<Value, MethodError>;
    async fn session_destroy(&mut self, session_ns: &str) -> Result<Value, MethodError>;
    async fn session_get_state(&mut self, session_ns: &str) -> Result<Value, MethodError>;
    async fn session_switch_scene(
        &mut self,
        session_ns: &str,
        scene_id: &str,
    ) -> Result<Value, MethodError>;
    async fn session_switch_interaction_mode(
        &mut self,
        session_ns: &str,
        mode: &str,
    ) -> Result<Value, MethodError>;
    async fn session_export_chat_logs(
        &mut self,
        session_ns: &str,
        format: &str,
        path: Option<&str>,
    ) -> Result<Value, MethodError>;

    // chat
    async fn chat_send_message(
        &mut self,
        session_ns: &str,
        user_message: &str,
        scene_id: Option<&str>,
    ) -> Result<Value, MethodError>;
    async fn chat_generate_monologue(
        &mut self,
        session_ns: &str,
        context: Option<&str>,
    ) -> Result<Value, MethodError>;

    // role
    async fn role_list(&mut self) -> Result<Value, MethodError>;
    async fn role_get_info(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
    ) -> Result<Value, MethodError>;
    async fn role_set_remote_life(
        &mut self,
        session_ns: &str,
        enabled: bool,
    ) -> Result<Value, MethodError>;

    // time
    async fn time_get_state(&mut self, session_ns: &str) -> Result<Value, MethodError>;
    async fn time_jump(
        &mut self,
        session_ns: &str,
        target_time_ms: Option<i64>,
        preset: Option<&str>,
    ) -> Result<Value, MethodError>;

    // agent
    async fn agent_call_mcp_tool(
        &mut self,
        server_id: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, MethodError>;

    // events
    fn push_event(&mut self, event: OocpEvent);
}
