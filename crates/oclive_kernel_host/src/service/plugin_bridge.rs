//! Directory plugin bridge command dispatch (kernel writer path).

use crate::command_error::{ApiError, CommandError};
use crate::domain::chat_engine::conversation_state_role_id;
use crate::error::AppError;
use crate::models::dto::{
    CreateEventRequest, ExportChatLogsRequest, GetRoleInfoRequest, SendMessageRequest,
};
use crate::models::EventType;
use crate::service::conversation::get_conversation_list_impl;
use crate::service::export::export_chat_logs_impl;
use crate::service::role::{
    delete_role_impl, get_role_info_impl, list_roles_impl, switch_role_impl,
};
use crate::service::settings_bridge::update_settings_impl;
use crate::service::time::get_time_state_impl;
use crate::state::AppState;
use serde_json::{json, Value};

#[inline]
fn bridge_invalid(msg: impl Into<String>) -> CommandError {
    CommandError::from(
        ApiError::InvalidParameter {
            message: msg.into(),
        }
        .to_string(),
    )
}

#[inline]
fn bridge_bad_json(ctx: &str, e: serde_json::Error) -> CommandError {
    CommandError::from(
        ApiError::InvalidParameter {
            message: format!("{}: {}", ctx, e),
        }
        .to_string(),
    )
}

#[inline]
fn bridge_serialize_host(ctx: &str, e: serde_json::Error) -> CommandError {
    CommandError::from(
        ApiError::Io {
            message: format!("host json {}: {}", ctx, e),
        }
        .to_string(),
    )
}

fn parse_event_type(s: &str) -> Result<EventType, CommandError> {
    match s {
        "Quarrel" => Ok(EventType::Quarrel),
        "Apology" => Ok(EventType::Apology),
        "Praise" => Ok(EventType::Praise),
        "Complaint" => Ok(EventType::Complaint),
        "Confession" => Ok(EventType::Confession),
        "Joke" => Ok(EventType::Joke),
        "Ignore" => Ok(EventType::Ignore),
        _ => Err(AppError::InvalidParameter(format!("Invalid event_type: {}", s)).into()),
    }
}

/// # Errors
///
/// Returns [`Err`] when the event type is invalid or persistence fails.
pub async fn create_event_impl(
    state: &AppState,
    req: &CreateEventRequest,
) -> Result<crate::models::dto::CreateEventResponse, CommandError> {
    let event_type = parse_event_type(&req.event_type)?;
    state.db_manager.ensure_role_runtime(&req.role_id).await?;

    let (id, timestamp) = state
        .db_manager
        .insert_manual_event(
            &req.role_id,
            &event_type,
            "manual",
            "manual",
            req.description.as_deref(),
        )
        .await?;

    Ok(crate::models::dto::CreateEventResponse {
        id,
        role_id: req.role_id.clone(),
        event_type: format!("{:?}", event_type),
        timestamp,
        description: req.description.clone(),
    })
}

/// Whether this bridge command must hit the kernel DB writer (attach mode HTTP proxy).
#[must_use]
pub fn bridge_command_needs_kernel_writer(command: &str) -> bool {
    matches!(
        command,
        "get_conversation"
            | "switch_role"
            | "get_roles"
            | "get_current_role"
            | "get_role_info"
            | "list_roles"
            | "get_time_state"
            | "update_memory"
            | "delete_memory"
            | "update_emotion"
            | "update_event"
            | "export_conversation"
            | "delete_role"
            | "update_settings"
            | "get_conversation_list"
    )
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn dispatch_bridge_command(
    state: &AppState,
    command: &str,
    params: Value,
) -> Result<Value, CommandError> {
    match command {
        "get_conversation" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("get_conversation: role_id required"))?;
            let session_id = params
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let limit = params
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50)
                .clamp(1, 500) as usize;
            let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let ns = conversation_state_role_id(role_id, session_id.as_deref());
            let rows = state.db_manager.list_short_term_turns(ns.as_str()).await?;
            let total = rows.len();
            let page: Vec<_> = rows.into_iter().skip(offset).take(limit).collect();
            let items: Vec<Value> = page
                .into_iter()
                .map(|(user, bot, emotion, scene, at)| {
                    json!({
                        "user_input": user,
                        "bot_reply": bot,
                        "emotion": emotion,
                        "scene": scene,
                        "created_at": at,
                    })
                })
                .collect();
            Ok(json!({
                "role_id": role_id,
                "session_namespace": ns,
                "total": total,
                "limit": limit,
                "offset": offset,
                "items": items,
            }))
        }
        "switch_role" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("switch_role: role_id required"))?;
            let info = switch_role_impl(state, role_id).await?;
            serde_json::to_value(info).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_roles" | "list_roles" => {
            let rows = list_roles_impl(state).await?;
            serde_json::to_value(rows).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_current_role" | "get_role_info" => {
            let req: GetRoleInfoRequest = if command == "get_current_role" {
                if let Some(inner) = params.get("req") {
                    serde_json::from_value(inner.clone())
                        .map_err(|e| bridge_bad_json("get_current_role.req", e))?
                } else {
                    serde_json::from_value(params)
                        .map_err(|e| bridge_bad_json("get_current_role", e))?
                }
            } else if params.is_null() {
                return Err(bridge_invalid("get_role_info: missing params"));
            } else if let Some(inner) = params.get("req") {
                serde_json::from_value(inner.clone())
                    .map_err(|e| bridge_bad_json("get_role_info.req", e))?
            } else {
                serde_json::from_value(params).map_err(|e| bridge_bad_json("get_role_info", e))?
            };
            let r = get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?;
            serde_json::to_value(r).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_time_state" => {
            let role_id = params
                .get("roleId")
                .or_else(|| params.get("role_id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("get_time_state: need roleId"))?;
            let t = get_time_state_impl(state, role_id).await?;
            serde_json::to_value(t).map_err(|e| bridge_serialize_host(command, e))
        }
        "update_memory" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_memory: role_id required"))?;
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_memory: content required"))?;
            let importance = params
                .get("importance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            state.db_manager.ensure_role_runtime(role_id).await?;
            let role = state.load_role_cached_async(role_id).await?;
            let threshold = role.pack_memory_config.similarity_threshold;
            let memory_id = state
                .db_manager
                .save_memory_merged(role_id, content, importance, threshold, "default")
                .await?;
            Ok(json!({ "memory_id": memory_id }))
        }
        "delete_memory" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("delete_memory: role_id required"))?;
            let memory_id = params
                .get("memory_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("delete_memory: memory_id required"))?;
            state.db_manager.ensure_role_runtime(role_id).await?;
            let deleted = state
                .db_manager
                .delete_memory_for_role(role_id, memory_id)
                .await?;
            if !deleted {
                return Err(bridge_invalid("delete_memory: not found or wrong role"));
            }
            Ok(json!({ "ok": true }))
        }
        "update_emotion" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_emotion: role_id required"))?;
            let emotion = params
                .get("emotion")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_emotion: emotion required"))?;
            state.db_manager.ensure_role_runtime(role_id).await?;
            state
                .db_manager
                .set_current_emotion(role_id, emotion)
                .await?;
            Ok(json!({ "ok": true }))
        }
        "update_event" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_event: role_id required"))?
                .to_string();
            let event_type = params
                .get("event_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("update_event: event_type required"))?
                .to_string();
            let description = params
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let req = CreateEventRequest {
                role_id,
                event_type,
                description,
            };
            let res = create_event_impl(state, &req).await?;
            serde_json::to_value(res).map_err(|e| bridge_serialize_host(command, e))
        }
        "export_conversation" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("export_conversation: role_id required"))?
                .to_string();
            let fmt = params
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("json");
            let session_id = params
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let req = ExportChatLogsRequest {
                role_id: Some(role_id),
                all_roles: false,
                format: fmt.to_string(),
                include_plugin_resolution_debug: false,
                session_id,
            };
            let res = export_chat_logs_impl(state, &req).await?;
            serde_json::to_value(res).map_err(|e| bridge_serialize_host(command, e))
        }
        "delete_role" => {
            let role_id = params
                .get("role_id")
                .or_else(|| params.get("roleId"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("delete_role: role_id required"))?
                .to_string();
            delete_role_impl(state, role_id).await
        }
        "update_settings" => update_settings_impl(state, &params).await,
        "get_conversation_list" => get_conversation_list_impl(state).await,
        "update_prompt" => Ok(json!({
            "ok": false,
            "error": "not_implemented",
            "message": "dynamic prompt template fragments are not wired in the host yet"
        })),
        _ => Err(ApiError::InvalidParameter {
            message: format!("unsupported bridge command: {}", command),
        }
        .to_string()
        .into()),
    }
}

/// Parse bridge `send_message` params (shared by Tauri attach routing).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn parse_send_message_request(params: &Value) -> Result<SendMessageRequest, CommandError> {
    let v = if let Some(inner) = params.get("req") {
        inner.clone()
    } else {
        params.clone()
    };
    let mut r: SendMessageRequest =
        serde_json::from_value(v).map_err(|e| bridge_bad_json("send_message", e))?;
    if r.user_message.trim().is_empty() {
        if let Some(t) = params.get("text").and_then(|x| x.as_str()) {
            r.user_message = t.to_string();
        }
    }
    if r.role_id.trim().is_empty() {
        return Err(bridge_invalid("send_message: role_id required"));
    }
    if r.user_message.trim().is_empty() {
        return Err(bridge_invalid(
            "send_message: user_message or text required",
        ));
    }
    Ok(r)
}
