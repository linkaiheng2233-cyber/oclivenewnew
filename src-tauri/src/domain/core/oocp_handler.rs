//! OOCP v0.1 传输无关 handler。
//!
//! 该模块不依赖任何传输层（Tauri / WS / HTTP / stdio）。
//! 所有外部依赖通过 traits 或函数参数注入。

use crate::models::oocp::{
    OocpCapabilities, OocpError, OocpErrorBody, OocpErrorCode, OocpEvent, OocpLimits,
    OocpRequest, OocpResponse, OOCP_EVENTS, OOCP_METHODS, OOCP_VERSION,
};
use serde_json::{json, Value};

// ── 错误构造辅助 ──────────────────────────────────────────────────────────

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

fn make_event(event: impl Into<String>, payload: Value) -> OocpEvent {
    OocpEvent {
        msg_type: "event",
        event: event.into(),
        payload,
    }
}

// ── Capabilities ───────────────────────────────────────────────────────────

/// 读取 OOCP 共享令牌（复用 WS 层的相同逻辑）。
fn oocp_api_token() -> Option<String> {
    std::env::var("OOCP_API_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn get_capabilities() -> OocpCapabilities {
    let auth_required = oocp_api_token().is_some();
    OocpCapabilities {
        msg_type: "capabilities",
        version: OOCP_VERSION,
        methods: OOCP_METHODS.to_vec(),
        events: OOCP_EVENTS.to_vec(),
        limits: OocpLimits {
            max_concurrent_requests: 8,
            max_message_chars: 4096,
        },
        auth_required,
    }
}

// ── Handler 结果 ──────────────────────────────────────────────────────────

/// 处理一个 OOCP 请求的可能结果。
pub enum OocpHandled {
    /// 成功响应。
    Response(OocpResponse),
    /// 错误响应。
    Error(OocpError),
    /// 需要客户端提供 capabilities（未做 capabilities 协商时）。
    Capabilities(OocpCapabilities),
}

// ── 核心 dispatch ─────────────────────────────────────────────────────────

/// 传输无关的 OOCP 请求分发器。
///
/// 接收一个已反序列化的 `OocpRequest`，返回一个 JSON-serializable 的响应。
/// **本函数不允许直接读写 Tauri State / AppHandle**。
///
/// 所有需要状态的操作均通过 `handler` 闭包参数完成，由 adapter 层注入。
pub async fn dispatch_oocp_request(
    req: OocpRequest,
    handler: &mut dyn OocpMethodHandler,
) -> OocpHandled {
    // 1) 如果客户端尚未获取 capabilities，要求先协商。
    if req.method.is_empty() {
        // 空方法视为 capabilities 请求。
        return OocpHandled::Capabilities(get_capabilities());
    }

    // 2) 校验方法白名单。
    if !OOCP_METHODS.contains(&req.method.as_str()) {
        return OocpHandled::Error(make_error(
            req.id.clone(),
            OocpErrorCode::UnsupportedMethod,
            format!("方法 '{}' 未在 capabilities 白名单中或尚未实现", req.method),
        ));
    }

    // 3) 分发到具体方法处理。
    let id = req.id.clone();
    match handle_method(req, handler).await {
        Ok(result) => OocpHandled::Response(make_response(id, result)),
        Err(e) => OocpHandled::Error(make_error(
            id,
            e.code,
            e.message,
        )),
    }
}

// ── 方法分发 ──────────────────────────────────────────────────────────────

/// 方法执行错误：由 `OocpMethodHandler` 实现返回，
/// 将被包装为 `OocpError` 响应。
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
    handler: &mut dyn OocpMethodHandler,
) -> Result<Value, MethodError> {
    match req.method.as_str() {
        // ── 会话生命周期 ──
        "session.create" => {
            let role_id = get_str(&req.params, "role_id")?;
            let session_id = get_str_opt(&req.params, "session_id");
            let scene_id = get_str_opt(&req.params, "scene_id");
            handler.session_create(role_id, session_id, scene_id).await
        }
        "session.destroy" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            handler.session_destroy(session_ns).await
        }
        "session.get_state" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            handler.session_get_state(session_ns).await
        }
        "session.switch_scene" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let scene_id = get_str(&req.params, "scene_id")?;
            handler.session_switch_scene(session_ns, scene_id).await
        }
        "session.switch_interaction_mode" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let mode = get_str(&req.params, "mode")?;
            handler.session_switch_interaction_mode(session_ns, mode).await
        }
        "session.export_chat_logs" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let format = get_str(&req.params, "format")?;
            let path = get_str_opt(&req.params, "path");
            handler.session_export_chat_logs(session_ns, format, path).await
        }

        // ── 对话 ──
        "chat.send_message" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let user_message = get_str(&req.params, "user_message")?;
            let scene_id = get_str_opt(&req.params, "scene_id");
            handler.chat_send_message(session_ns, user_message, scene_id).await
        }
        "chat.generate_monologue" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let context = get_str_opt(&req.params, "context");
            handler.chat_generate_monologue(session_ns, context).await
        }

        // ── 角色 ──
        "role.list" => {
            handler.role_list().await
        }
        "role.get_info" => {
            let role_id = get_str(&req.params, "role_id")?;
            handler.role_get_info(role_id).await
        }
        "role.set_remote_life" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let enabled = get_bool(&req.params, "enabled").unwrap_or(true);
            handler.role_set_remote_life(session_ns, enabled).await
        }

        // ── 时间 ──
        "time.get_state" => {
            handler.time_get_state().await
        }
        "time.jump" => {
            let session_ns = get_str(&req.params, "session_ns")?;
            let target_time_ms = get_i64_opt(&req.params, "target_time_ms")
                .ok_or_else(|| MethodError {
                    code: OocpErrorCode::InvalidParams,
                    message: "缺少必填参数 target_time_ms".into(),
                })?;
            handler.time_jump(session_ns, target_time_ms).await
        }

        // ── Agent ──
        "agent.call_mcp_tool" => {
            let server_id = get_str(&req.params, "server_id")?;
            let tool_name = get_str(&req.params, "tool_name")?;
            let arguments = req.params.get("arguments").cloned().unwrap_or(Value::Null);
            handler.agent_call_mcp_tool(server_id, tool_name, arguments).await
        }

        other => Err(MethodError {
            code: OocpErrorCode::UnsupportedMethod,
            message: format!("未知方法: {}", other),
        }),
    }
}

// ── 参数提取辅助 ──────────────────────────────────────────────────────────

fn get_str(params: &Value, key: &str) -> Result<&str, MethodError> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| MethodError {
            code: OocpErrorCode::InvalidParams,
            message: format!("缺少必填参数 {}", key),
        })
}

fn get_str_opt<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

fn get_bool(params: &Value, key: &str) -> Option<bool> {
    params.get(key).and_then(|v| v.as_bool())
}

fn get_i64_opt(params: &Value, key: &str) -> Option<i64> {
    params.get(key).and_then(|v| v.as_i64())
}

// ── 方法处理 trait ────────────────────────────────────────────────────────

/// 传输无关的方法处理接口。
///
/// 每个传输层 adapter 需要实现此 trait，
/// 将 OOCP 方法调用转换为对现有业务逻辑的调用。
///
/// 所有方法返回 `Result<Value, MethodError>`，
/// 其中 `Value` 是 JSON Value（由 `OocpHandler` 包装为 `OocpResponse`）。
#[allow(async_fn_in_trait)] // stable in 1.75+
pub trait OocpMethodHandler {
    // ── 会话 ──
    /// 创建会话，返回 `{ session_ns, role }`。
    async fn session_create(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
        scene_id: Option<&str>,
    ) -> Result<Value, MethodError>;

    /// 销毁会话，返回 `{}`。
    async fn session_destroy(
        &mut self,
        session_ns: &str,
    ) -> Result<Value, MethodError>;

    /// 获取会话状态快照。
    async fn session_get_state(
        &mut self,
        session_ns: &str,
    ) -> Result<Value, MethodError>;

    /// 切换场景。
    async fn session_switch_scene(
        &mut self,
        session_ns: &str,
        scene_id: &str,
    ) -> Result<Value, MethodError>;

    /// 切换交互模式。
    async fn session_switch_interaction_mode(
        &mut self,
        session_ns: &str,
        mode: &str,
    ) -> Result<Value, MethodError>;

    /// 导出聊天日志，返回 `{ path, size_bytes }`。
    async fn session_export_chat_logs(
        &mut self,
        session_ns: &str,
        format: &str,
        path: Option<&str>,
    ) -> Result<Value, MethodError>;

    // ── 对话 ──
    /// 发送消息给角色，返回完整的 `SendMessageResponse`。
    async fn chat_send_message(
        &mut self,
        session_ns: &str,
        user_message: &str,
        scene_id: Option<&str>,
    ) -> Result<Value, MethodError>;

    /// 生成角色独白，返回 `{ monologue }`。
    async fn chat_generate_monologue(
        &mut self,
        session_ns: &str,
        context: Option<&str>,
    ) -> Result<Value, MethodError>;

    // ── 角色 ──
    /// 列出所有已加载角色。
    async fn role_list(&mut self) -> Result<Value, MethodError>;

    /// 获取角色详情。
    async fn role_get_info(
        &mut self,
        role_id: &str,
    ) -> Result<Value, MethodError>;

    /// 开关异地心声。
    async fn role_set_remote_life(
        &mut self,
        session_ns: &str,
        enabled: bool,
    ) -> Result<Value, MethodError>;

    // ── 时间 ──
    /// 获取虚拟时间状态。
    async fn time_get_state(&mut self) -> Result<Value, MethodError>;

    /// 时间跳跃。
    async fn time_jump(
        &mut self,
        session_ns: &str,
        target_time_ms: i64,
    ) -> Result<Value, MethodError>;

    // ── Agent / MCP ──
    /// 调用 MCP 工具，返回 `{ content, is_error }`。
    async fn agent_call_mcp_tool(
        &mut self,
        server_id: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, MethodError>;

    // ── 事件推送 ──
    /// 推送事件；adapter 层实现具体推送方式（WS send / 回调等）。
    /// 返回的事件列表由 handler 在方法执行后调用。
    fn push_event(&mut self, event: OocpEvent);
}