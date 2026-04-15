//! 目录插件页 `OclivePluginBridge` → 受控调用宿主 Tauri 命令（manifest `shell.bridge.invoke` 权限白名单）。
//!
//! - **权限令牌**：部分命令使用 `read:*` 形式（见 `required_permission_token`）；`invoke` 数组中声明 **命令名或对应权限名** 均可通过校验。
//! - **整壳深度集成**：`send_message` / `get_conversation` / `switch_role` / `get_roles` / `get_current_role` 还要求
//!   `manifest.type == "ocliveplugin"` 且请求来自 **`shell.entry`** 对应 HTML（非 `ui_slots` 页）。

use crate::api::directory_plugin::directory_plugin_bootstrap_dto;
use crate::api::event::create_event_impl;
use crate::api::role::{get_role_info_impl, list_roles_impl, switch_role_impl};
use crate::api::time::get_time_state_impl;
use crate::models::dto::CreateEventRequest;
use crate::domain::chat_engine::{conversation_state_role_id, process_message};
use crate::infrastructure::directory_plugins::{normalize_plugin_rel, OclivePluginManifest};
use crate::models::dto::{GetRoleInfoRequest, SendMessageRequest};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginBridgeInvokeRequest {
    pub plugin_id: String,
    pub asset_rel: String,
    pub command: String,
    #[serde(default)]
    pub params: Value,
}

/// 桥接命令名 → manifest `bridge.invoke` 中需声明的权限串（与命令名不同则二者任一命中即可）。
fn required_permission_token(cmd: &str) -> String {
    match cmd {
        "get_conversation" => "read:conversation".to_string(),
        "get_roles" => "read:roles".to_string(),
        "get_current_role" => "read:current_role".to_string(),
        "update_memory" | "delete_memory" => "write:memory".to_string(),
        "update_emotion" => "write:emotion".to_string(),
        "update_event" => "write:event".to_string(),
        "update_prompt" => "write:prompt".to_string(),
        _ => cmd.to_string(),
    }
}

fn invoke_list_allows(invoke: &[String], cmd: &str) -> bool {
    let need = required_permission_token(cmd);
    invoke.iter().any(|x| {
        let t = x.trim();
        t == cmd || t == need.as_str()
    })
}

/// 仅允许自 **`type: "ocliveplugin"`** 的整壳 **`shell.entry`** 调用的敏感命令。
fn requires_typed_shell(cmd: &str) -> bool {
    matches!(
        cmd,
        "send_message"
            | "get_conversation"
            | "switch_role"
            | "get_roles"
            | "get_current_role"
            | "update_memory"
            | "delete_memory"
            | "update_emotion"
            | "update_event"
            | "update_prompt"
    )
}

fn validate_shell_ocliveplugin(manifest: &OclivePluginManifest, asset_rel: &str) -> Result<(), String> {
    if manifest.plugin_type.as_deref().map(str::trim) != Some("ocliveplugin") {
        return Err(
            "this command requires manifest \"type\": \"ocliveplugin\" and shell.bridge.invoke permission"
                .to_string(),
        );
    }
    let Some(sh) = &manifest.shell else {
        return Err("this command is only allowed for shell plugins".to_string());
    };
    if normalize_plugin_rel(asset_rel) != normalize_plugin_rel(&sh.entry) {
        return Err("this command must be invoked from shell.entry HTML (not ui_slots)".to_string());
    }
    Ok(())
}

fn validate_bridge(
    state: &AppState,
    plugin_id: &str,
    asset_rel: &str,
    command: &str,
) -> Result<(), String> {
    let roots = state.directory_plugins.plugin_roots.read();
    let root = roots
        .get(plugin_id)
        .ok_or_else(|| format!("unknown plugin_id={}", plugin_id))?;
    let manifest = OclivePluginManifest::load_from_dir(root)?;
    let rel = normalize_plugin_rel(asset_rel);
    let Some(b) = manifest.bridge_for_asset_rel(&rel) else {
        return Err("asset has no bridge config".to_string());
    };
    if !invoke_list_allows(&b.invoke, command) {
        let tok = required_permission_token(command);
        return Err(format!(
            "bridge.invoke must include command {:?} or permission {:?}",
            command, tok.as_str()
        ));
    }
    if requires_typed_shell(command) {
        validate_shell_ocliveplugin(&manifest, &rel)?;
    }
    Ok(())
}

fn parse_send_message_request(params: &Value) -> Result<SendMessageRequest, String> {
    let v = if let Some(inner) = params.get("req") {
        inner.clone()
    } else {
        params.clone()
    };
    let mut r: SendMessageRequest = serde_json::from_value(v).map_err(|e| e.to_string())?;
    if r.user_message.trim().is_empty() {
        if let Some(t) = params.get("text").and_then(|x| x.as_str()) {
            r.user_message = t.to_string();
        }
    }
    if r.role_id.trim().is_empty() {
        return Err("send_message: role_id required".to_string());
    }
    if r.user_message.trim().is_empty() {
        return Err("send_message: user_message or text required".to_string());
    }
    Ok(r)
}

async fn dispatch_bridge_command(
    state: &AppState,
    command: &str,
    params: Value,
) -> Result<Value, String> {
    match command {
        "send_message" => {
            let req = parse_send_message_request(&params)?;
            let res = process_message(state, &req)
                .await
                .map_err(|e: crate::error::AppError| e.to_frontend_error())?;
            serde_json::to_value(res).map_err(|e| e.to_string())
        }
        "get_conversation" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "get_conversation: role_id required".to_string())?;
            let session_id = params
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let limit = params
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50)
                .clamp(1, 500) as usize;
            let offset = params
                .get("offset")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let ns = conversation_state_role_id(role_id, session_id.as_deref());
            let rows = state
                .db_manager
                .list_short_term_turns(ns.as_str())
                .await
                .map_err(|e| e.to_frontend_error())?;
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
                .ok_or_else(|| "switch_role: role_id required".to_string())?;
            let info = switch_role_impl(state, role_id).await?;
            serde_json::to_value(info).map_err(|e| e.to_string())
        }
        "get_roles" => {
            let rows = list_roles_impl(state).await?;
            serde_json::to_value(rows).map_err(|e| e.to_string())
        }
        "get_current_role" => {
            let req: GetRoleInfoRequest = if let Some(inner) = params.get("req") {
                serde_json::from_value(inner.clone()).map_err(|e| e.to_string())?
            } else {
                serde_json::from_value(params).map_err(|e| e.to_string())?
            };
            let r = get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }
        "get_role_info" => {
            let req: GetRoleInfoRequest = if params.is_null() {
                return Err("get_role_info: missing params".to_string());
            } else if let Some(inner) = params.get("req") {
                serde_json::from_value(inner.clone()).map_err(|e| e.to_string())?
            } else {
                serde_json::from_value(params).map_err(|e| e.to_string())?
            };
            let r = get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?;
            serde_json::to_value(r).map_err(|e| e.to_string())
        }
        "list_roles" => {
            let rows = list_roles_impl(state).await?;
            serde_json::to_value(rows).map_err(|e| e.to_string())
        }
        "get_time_state" => {
            let role_id = params
                .get("roleId")
                .or_else(|| params.get("role_id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| "get_time_state: need roleId".to_string())?;
            let t = get_time_state_impl(state, role_id).await?;
            serde_json::to_value(t).map_err(|e| e.to_string())
        }
        "get_directory_plugin_bootstrap" => {
            let role_id = params
                .get("roleId")
                .or_else(|| params.get("role_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let dto = directory_plugin_bootstrap_dto(state, role_id);
            serde_json::to_value(dto).map_err(|e| e.to_string())
        }
        "update_memory" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_memory: role_id required".to_string())?;
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_memory: content required".to_string())?;
            let importance = params
                .get("importance")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5)
                .clamp(0.0, 1.0);
            state
                .db_manager
                .ensure_role_runtime(role_id)
                .await
                .map_err(|e| e.to_frontend_error())?;
            let memory_id = state
                .memory_repo
                .save_memory(role_id, content, importance)
                .await
                .map_err(|e| e.to_frontend_error())?;
            Ok(json!({ "memory_id": memory_id }))
        }
        "delete_memory" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "delete_memory: role_id required".to_string())?;
            let memory_id = params
                .get("memory_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "delete_memory: memory_id required".to_string())?;
            state
                .db_manager
                .ensure_role_runtime(role_id)
                .await
                .map_err(|e| e.to_frontend_error())?;
            let deleted = state
                .db_manager
                .delete_memory_for_role(role_id, memory_id)
                .await
                .map_err(|e| e.to_frontend_error())?;
            if !deleted {
                return Err("delete_memory: not found or wrong role".to_string());
            }
            Ok(json!({ "ok": true }))
        }
        "update_emotion" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_emotion: role_id required".to_string())?;
            let emotion = params
                .get("emotion")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_emotion: emotion required".to_string())?;
            state
                .db_manager
                .ensure_role_runtime(role_id)
                .await
                .map_err(|e| e.to_frontend_error())?;
            state
                .db_manager
                .set_current_emotion(role_id, emotion)
                .await
                .map_err(|e| e.to_frontend_error())?;
            Ok(json!({ "ok": true }))
        }
        "update_event" => {
            let role_id = params
                .get("role_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_event: role_id required".to_string())?
                .to_string();
            let event_type = params
                .get("event_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "update_event: event_type required".to_string())?
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
            serde_json::to_value(res).map_err(|e| e.to_string())
        }
        "update_prompt" => Ok(json!({
            "ok": false,
            "error": "not_implemented",
            "message": "dynamic prompt template fragments are not wired in the host yet"
        })),
        _ => Err(format!("unsupported bridge command: {}", command)),
    }
}

#[tauri::command]
pub async fn plugin_bridge_invoke(
    req: PluginBridgeInvokeRequest,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let pid = req.plugin_id.trim();
    let asset = normalize_plugin_rel(req.asset_rel.trim());
    let cmd = req.command.trim();
    if pid.is_empty() || asset.is_empty() || cmd.is_empty() {
        return Err("plugin_id, asset_rel, command required".to_string());
    }
    validate_bridge(&state, pid, &asset, cmd)?;
    dispatch_bridge_command(&state, cmd, req.params).await
}
