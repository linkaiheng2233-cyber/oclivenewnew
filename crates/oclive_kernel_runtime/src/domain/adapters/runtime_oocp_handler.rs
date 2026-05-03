//! OOCP method handler implementation for the kernel runtime.
//!
//! This file is migrated from `src-tauri/src/domain/adapters/tauri_oocp_handler.rs`
//! but is **not** Tauri-specific. It only depends on `AppState`.

use crate::domain::core::oocp_handler::{MethodError, OocpMethodHandler};
use crate::domain::export_chat_logs;
use crate::models::oocp::{OocpErrorCode, OocpEvent};
use crate::state::KernelAppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::task::spawn_blocking;

fn make_session_ns(role_id: &str, session_id: Option<&str>) -> String {
    match session_id.filter(|s| !s.is_empty()) {
        Some(sid) => format!("{}__sess__{}", role_id, sid),
        None => format!("{}__sess__default", role_id),
    }
}

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

pub struct RuntimeOocpHandler {
    state: Arc<KernelAppState>,
    pending_events: Vec<OocpEvent>,
}

impl RuntimeOocpHandler {
    pub fn new(state: Arc<KernelAppState>) -> Self {
        Self {
            state,
            pending_events: Vec::new(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<OocpEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn push_event(&mut self, event: OocpEvent) {
        self.pending_events.push(event);
    }
}

impl OocpMethodHandler for RuntimeOocpHandler {
    fn push_event(&mut self, event: OocpEvent) {
        self.push_event(event);
    }

    async fn session_create(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
        _scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        let role_id_s = role_id.to_string();
        let state = Arc::clone(&self.state);
        let loaded = spawn_blocking(move || state.load_role_cached(role_id_s.as_str()))
            .await
            .map_err(|e| {
                err(
                    OocpErrorCode::Internal,
                    format!("load_role task failed: {}", e),
                )
            })?
            .map_err(|e| err(OocpErrorCode::RoleNotFound, e.to_string()))?;

        self.state
            .db_manager
            .ensure_role_runtime(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        let current_emotion = self
            .state
            .db_manager
            .get_current_emotion(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        if current_emotion.is_none() {
            let _ = self
                .state
                .db_manager
                .set_current_emotion(role_id, "neutral")
                .await;
        }

        let session_ns = make_session_ns(role_id, session_id);

        Ok(json!({
            "session_ns": session_ns,
            "role": {
                "role_id": loaded.id,
                "name": loaded.name,
                "interaction_mode": loaded.interaction_mode.clone().unwrap_or_else(|| "immersive".to_string()),
            }
        }))
    }

    async fn session_destroy(&mut self, _session_ns: &str) -> Result<Value, MethodError> {
        Ok(json!({}))
    }

    async fn session_get_state(&mut self, session_ns: &str) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;

        self.state
            .db_manager
            .ensure_role_runtime(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;

        let current_scene = self
            .state
            .db_manager
            .get_current_scene(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        let user_presence_scene = self
            .state
            .db_manager
            .get_user_presence_scene(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        let current_favorability = self
            .state
            .db_manager
            .get_favorability(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?
            .unwrap_or(0.0);
        let relation_state = self
            .state
            .db_manager
            .get_relation_state(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?
            .unwrap_or_else(|| "Stranger".to_string());
        let current_emotion = self
            .state
            .db_manager
            .get_current_emotion(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?
            .unwrap_or_else(|| "neutral".to_string());
        let interaction_mode = self
            .state
            .db_manager
            .get_interaction_mode(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        let remote_life_enabled = self
            .state
            .db_manager
            .get_remote_life_enabled(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        let virtual_time_ms = self
            .state
            .db_manager
            .get_virtual_time_ms(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?
            .unwrap_or(0);

        Ok(json!({
            "role_id": role_id,
            "current_scene": current_scene,
            "current_favorability": current_favorability,
            "relation_state": relation_state,
            "current_emotion": current_emotion,
            "interaction_mode": interaction_mode.as_str(),
            "remote_life_enabled": remote_life_enabled,
            "user_presence_scene": user_presence_scene,
            "virtual_time_ms": virtual_time_ms,
        }))
    }

    async fn session_switch_scene(
        &mut self,
        session_ns: &str,
        scene_id: &str,
    ) -> Result<Value, MethodError> {
        let (role_id, _session_id) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;

        let scenes = self
            .state
            .storage
            .list_scene_ids(role_id)
            .map_err(|e| err(OocpErrorCode::InvalidParams, e.to_frontend_error()))?;
        if !scenes.iter().any(|s| s == scene_id) {
            return Err(err(
                OocpErrorCode::InvalidParams,
                format!("scene_id not in role pack: {}", scene_id),
            ));
        }

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

        self.state
            .db_manager
            .ensure_role_runtime(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        self.state
            .db_manager
            .set_current_scene(role_id, scene_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        self.state
            .db_manager
            .set_user_presence_scene(role_id, scene_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;

        Ok(json!({
            "scene_id": scene_id,
            "scene_name": self.state.storage.scene_display_name(role_id, scene_id),
        }))
    }

    async fn session_switch_interaction_mode(
        &mut self,
        session_ns: &str,
        mode: &str,
    ) -> Result<Value, MethodError> {
        let (role_id, _session_id) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;

        self.state
            .db_manager
            .ensure_role_runtime(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        self.state
            .db_manager
            .set_interaction_mode_for_role(role_id, mode.trim())
            .await
            .map_err(|e| err(OocpErrorCode::InvalidParams, e.to_frontend_error()))?;

        Ok(json!({ "mode": mode.trim() }))
    }

    async fn session_export_chat_logs(
        &mut self,
        session_ns: &str,
        format: &str,
        _path: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;

        export_chat_logs::export_session_chat_logs_oocp_value(&self.state, role_id, format)
            .await
            .map_err(|e| match e {
                crate::error::AppError::InvalidParameter(m) => err(OocpErrorCode::InvalidParams, m),
                crate::error::AppError::RoleNotFound(m) => err(OocpErrorCode::RoleNotFound, m),
                other => err(OocpErrorCode::Internal, other.to_frontend_error()),
            })
    }

    async fn chat_send_message(
        &mut self,
        session_ns: &str,
        user_message: &str,
        scene_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (role_id, session_id) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;
        let req = crate::models::dto::SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: user_message.to_string(),
            scene_id: scene_id.map(|s| s.to_string()),
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
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;

        let scene_id = self
            .state
            .db_manager
            .get_current_scene(role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?
            .unwrap_or_else(|| "default".to_string());

        let resp = crate::domain::virtual_time::generate_monologue(&self.state, role_id, context)
            .await
            .map_err(|e| match e {
                crate::error::AppError::InvalidParameter(m) => err(OocpErrorCode::InvalidParams, m),
                crate::error::AppError::OllamaError(m) => err(OocpErrorCode::LlmFailure, m),
                other => err(OocpErrorCode::Internal, other.to_frontend_error()),
            })?;

        let trigger = context.unwrap_or("user_afk");
        self.push_event(OocpEvent {
            msg_type: "event",
            event: "chat.monologue".to_string(),
            payload: json!({
                "session_ns": session_ns,
                "monologue": resp.text,
                "scene_id": scene_id,
                "trigger": trigger,
            }),
        });
        Ok(json!({ "monologue": resp.text }))
    }

    async fn role_list(&mut self) -> Result<Value, MethodError> {
        let storage = self.state.storage.clone();
        let roles = spawn_blocking(move || storage.load_all_roles())
            .await
            .map_err(|e| {
                err(
                    OocpErrorCode::Internal,
                    format!("load_all_roles task failed: {}", e),
                )
            })?
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;

        let list: Vec<Value> = roles
            .into_iter()
            .map(|r| {
                json!({
                    "role_id": r.id,
                    "id": r.id,
                    "manifestId": r.id,
                    "name": r.name,
                })
            })
            .collect();

        Ok(json!(list))
    }

    async fn role_get_info(
        &mut self,
        role_id: &str,
        session_id: Option<&str>,
    ) -> Result<Value, MethodError> {
        let info = crate::domain::role_info_snapshot::get_role_info_snapshot(
            &self.state,
            role_id,
            session_id,
        )
        .await
        .map_err(|e| match e {
            crate::error::AppError::InvalidParameter(m) => err(OocpErrorCode::InvalidParams, m),
            crate::error::AppError::RoleNotFound(m) => err(OocpErrorCode::RoleNotFound, m),
            other => err(OocpErrorCode::Internal, other.to_frontend_error()),
        })?;
        serde_json::to_value(&info).map_err(|e| {
            err(
                OocpErrorCode::Internal,
                format!("serialize RoleInfo: {}", e),
            )
        })
    }

    async fn role_set_remote_life(
        &mut self,
        session_ns: &str,
        enabled: bool,
    ) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;
        self.state
            .db_manager
            .set_remote_life_enabled(role_id, enabled)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))?;
        Ok(json!({ "enabled": enabled }))
    }

    async fn time_get_state(&mut self, session_ns: &str) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;
        crate::domain::virtual_time::get_time_state_oocp_value(&self.state, role_id)
            .await
            .map_err(|e| err(OocpErrorCode::Internal, e.to_frontend_error()))
    }

    async fn time_jump(
        &mut self,
        session_ns: &str,
        target_time_ms: Option<i64>,
        preset: Option<&str>,
    ) -> Result<Value, MethodError> {
        let (role_id, _) = parse_session_ns(session_ns).ok_or_else(|| {
            err(
                OocpErrorCode::InvalidParams,
                format!("无效的 session_ns: {}", session_ns),
            )
        })?;
        crate::domain::virtual_time::jump_time_oocp_from_params(
            &self.state,
            role_id,
            target_time_ms,
            preset,
        )
        .await
        .map_err(|e| match e {
            crate::error::AppError::InvalidParameter(m) => err(OocpErrorCode::InvalidParams, m),
            other => err(OocpErrorCode::Internal, other.to_frontend_error()),
        })
    }

    async fn agent_call_mcp_tool(
        &mut self,
        server_id: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, MethodError> {
        let res = self
            .state
            .plugins
            .call_mcp_tool(server_id, tool_name, arguments.clone())
            .map_err(|e| err(OocpErrorCode::Internal, format!("MCP 工具调用失败: {}", e)))?;

        let value =
            serde_json::to_value(&res).map_err(|e| err(OocpErrorCode::Internal, e.to_string()))?;

        self.push_event(OocpEvent {
            msg_type: "event",
            event: "trace.append".to_string(),
            payload: json!({
                "kind": "mcp_tool_call",
                "server_id": server_id,
                "tool_name": tool_name,
                "arguments": arguments,
                "result": value,
            }),
        });

        Ok(value)
    }
}
