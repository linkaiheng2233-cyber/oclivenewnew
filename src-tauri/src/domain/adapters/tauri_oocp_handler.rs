//! OOCP `OocpMethodHandler` 的 Tauri adapter 实现。
//!
//! 将 OOCP 传输无关方法调用桥接到现有 Tauri 应用逻辑
//! （通过 `AppState` 访问所有业务 engine）。
//!
//! 命名空间规则：
//! - `session.create` 返回 `role_id` + `session_id` 组合的 `session_ns`。
//! - 后续请求通过 `session_ns` 反查 `role_id` + 可选 `session_id`。

use crate::domain::core::oocp_handler::{MethodError, OocpMethodHandler};
use crate::models::oocp::{OocpErrorCode, OocpEvent};
use crate::state::AppState;
use serde_json::{json, Value};

use std::sync::Arc;

/// 将 `role_id` 和可选 `session_id` 编码为 `session_ns`。
fn make_session_ns(role_id: &str, session_id: Option<&str>) -> String {
    match session_id.filter(|s| !s.is_empty()) {
        Some(sid) => format!("{}__sess__{}", role_id, sid),
        None => format!("{}__sess__default", role_id),
    }
}

/// 从 `session_ns` 解构出 `(role_id, Option<session_id>)`。
fn parse_session_ns(session_ns: &str) -> Option<(&str, Option<&str>)> {
    let (role_id, rest) = session_ns.split_once("__sess__")?;
    let session_id = match rest {
        "default" => None,
        other => Some(other),
    };
    Some((role_id, session_id))
}

fn err(code: OocpErrorCode, msg: impl Into<String>) -> MethodError {
    MethodError::new(code, msg)
}

/// OOCP v0.1 的 Tauri adapter。
///
/// 持有 `AppState`（通过 Arc）以便调用所有业务 engine。
pub struct TauriOocpHandler {
    state: Arc<AppState>,
    /// 待推送的事件缓冲（方法执行后由 transport 层消费）。
    pending_events: Vec<OocpEvent>,
}

impl TauriOocpHandler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            pending_events: Vec::new(),
        }
    }

    /// 消费所有已缓冲的事件（由 transport 层调用以发送事件）。
    pub fn drain_events(&mut self) -> Vec<OocpEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

impl OocpMethodHandler for TauriOocpHandler {
    // ── 会话 ──────────────────────────────────────────────────────────────

    async fn session_create(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
        _scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        let session_ns = make_session_ns(role_id, session_id);

        // TODO P0-C: 委托到 role_manager 加载角色。
        // 当前作为最小实现，返回 session_ns 与空角色信息。

        Ok(json!({
            "session_ns": session_ns,
            "role": {
                "name": role_id,
                "scenes": ["default"],
                "interaction_mode": "chat",
            }
        }))
    }

    async fn session_destroy(
        &mut self,
        _session_ns: &str,
    ) -> Result<Value, MethodError> {
        // v0.1 最小实现：不清理内存/DB。
        Ok(json!({}))
    }

    async fn session_get_state(
        &mut self,
        session_ns: &str,
    ) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 role_manager / DB 查询角色运行时状态。
        // 当前为占位实现。
        Ok(json!({
            "role_id": role_id,
            "current_scene": "default",
            "current_favorability": 50,
            "relation_state": "neutral",
            "current_emotion": "neutral",
            "interaction_mode": "chat",
            "remote_life_enabled": false,
            "user_presence_scene": "default",
            "virtual_time_ms": 0,
        }))
    }

    async fn session_switch_scene(
        &mut self,
        session_ns: &str,
        scene_id: &str,
    ) -> Result<Value, MethodError> {
        let (_role_id, _session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 scene_store / role_manager 加载场景配置。
        // 当前为占位实现。

        self.push_event(OocpEvent {
            msg_type: "event",
            event: "chat.monologue".to_string(),
            payload: json!({
                "session_ns": session_ns,
                "monologue": format!("（场景切换: {}）", scene_id),
                "scene_id": scene_id,
                "trigger": "scene_change",
            }),
        });

        Ok(json!({
            "scene_id": scene_id,
            "scene_name": scene_id,
        }))
    }

    async fn session_switch_interaction_mode(
        &mut self,
        session_ns: &str,
        mode: &str,
    ) -> Result<Value, MethodError> {
        let (_role_id, _session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 role_runtime DB 更新。
        // 当前为占位实现。
        Ok(json!({ "mode": mode }))
    }

    async fn session_export_chat_logs(
        &mut self,
        session_ns: &str,
        format: &str,
        _path: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (_role_id, _) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 export 模块。
        // 当前为占位实现。
        Ok(json!({
            "path": format!("export_{}.{}", session_ns, format),
            "size_bytes": 0,
        }))
    }

    // ── 对话 ──────────────────────────────────────────────────────────────

    async fn chat_send_message(
        &mut self,
        session_ns: &str,
        user_message: &str,
        _scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (role_id, session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // 委托到现有的 chat_engine。
        let req = crate::models::dto::SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: user_message.to_string(),
            scene_id: _scene_id.map(|s| s.to_string()),
            session_id: session_id.map(|s| s.to_string()),
        };

        let response = crate::domain::chat_engine::process_message(&self.state, &req)
            .await
            .map_err(|e| err(OocpErrorCode::LlmFailure, format!("对话处理失败: {}", e)))?;

        serde_json::to_value(&response)
            .map_err(|e| err(OocpErrorCode::Internal, format!("序列化响应失败: {}", e)))
    }

    async fn chat_generate_monologue(
        &mut self,
        session_ns: &str,
        context: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (_role_id, _session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 monologue engine。
        // 当前为占位实现。
        let trigger = context.unwrap_or("user_afk");
        let monologue_text = format!("（独白 - {} - {}）", session_ns, trigger);

        self.push_event(OocpEvent {
            msg_type: "event",
            event: "chat.monologue".to_string(),
            payload: json!({
                "session_ns": session_ns,
                "monologue": monologue_text,
                "scene_id": "default",
                "trigger": trigger,
            }),
        });

        Ok(json!({ "monologue": monologue_text }))
    }

    // ── 角色 ──────────────────────────────────────────────────────────────

    async fn role_list(
        &mut self,
    ) -> Result<Value, MethodError> {
        // TODO P0-C: 委托到 role_manager 获取已加载角色列表。
        // 当前为占位实现。
        Ok(json!([]))
    }

    async fn role_get_info(
        &mut self,
        role_id: &str,
    ) -> Result<Value, MethodError> {
        // TODO P0-C: 委托到 role_manager。
        // 当前为占位实现。
        Ok(json!({
            "role_id": role_id,
            "role_name": role_id,
            "scenes": ["default"],
            "interaction_mode": "chat",
        }))
    }

    async fn role_set_remote_life(
        &mut self,
        session_ns: &str,
        enabled: bool,
    ) -> Result<Value, MethodError> {
        let (_role_id, _session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 role_manager。
        // 当前为占位实现。
        Ok(json!({ "enabled": enabled }))
    }

    // ── 时间 ──────────────────────────────────────────────────────────────

    async fn time_get_state(
        &mut self,
    ) -> Result<Value, MethodError> {
        // TODO P0-C: 委托到 time engine。
        // 当前为占位实现。
        Ok(json!({
            "virtual_time_ms": 0,
            "virtual_time_label": "2024-01-01 00:00:00",
            "time_speed_multiplier": 1.0,
        }))
    }

    async fn time_jump(
        &mut self,
        session_ns: &str,
        target_time_ms: i64,
    ) -> Result<Value, MethodError> {
        let (_role_id, _session_id) = parse_session_ns(session_ns)
            .ok_or_else(|| err(OocpErrorCode::InvalidParams, format!("无效的 session_ns: {}", session_ns)))?;

        // TODO P0-C: 委托到 time engine。
        // 当前为占位实现。
        Ok(json!({
            "virtual_time_ms": target_time_ms,
        }))
    }

    // ── Agent / MCP ───────────────────────────────────────────────────────

    async fn agent_call_mcp_tool(
        &mut self,
        server_id: &str,
        tool_name: &str,
        _arguments: Value,
    ) -> Result<Value, MethodError> {
        // TODO P0-C: 委托到 MCP client。
        // 当前为占位实现。
        Ok(json!({
            "content": format!("[placeholder] mcp call {}::{}", server_id, tool_name),
            "is_error": false,
        }))
    }

    // ── 事件推送 ──────────────────────────────────────────────────────────

    fn push_event(&mut self, event: OocpEvent) {
        self.pending_events.push(event);
    }
}