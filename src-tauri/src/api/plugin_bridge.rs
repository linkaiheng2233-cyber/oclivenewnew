//! 目录插件页 `OclivePluginBridge` → 受控调用宿主 Tauri 命令（manifest `shell.bridge.invoke` 权限白名单）。
//!
//! - **权限令牌**：部分命令使用 `read:*` 形式（见 `required_permission_token`）；`invoke` 数组中声明 **命令名或对应权限名** 均可通过校验。
//! - **整壳深度集成**：`send_message` / `get_conversation` / `switch_role` / `get_roles` / `get_current_role` /
//!   `export_conversation` / `import_role` 以及写入类命令还要求
//!   `manifest.type == "ocliveplugin"` 且请求来自 **`shell.entry`** 对应 HTML 或 **`shell.vueEntry`** 宿主 Vue 入口（非 `ui_slots` 页）。

use crate::api::conversation::get_conversation_list_impl;
use crate::api::directory_plugin::directory_plugin_bootstrap_dto;
use crate::api::error::ApiError;
use crate::api::event::create_event_impl;
use crate::api::export::export_chat_logs_impl;
use crate::api::role::{
    delete_role_impl, get_role_info_impl, list_roles_impl, switch_role_impl,
};
use crate::api::settings::update_settings_impl;
use crate::api::time::get_time_state_impl;
use crate::domain::chat_engine::{conversation_state_role_id, process_message};
use crate::infrastructure::directory_plugins::{normalize_plugin_rel, OclivePluginManifest};
use crate::infrastructure::import_role_pack;
use crate::models::dto::CreateEventRequest;
use crate::models::dto::{ExportChatLogsRequest, GetRoleInfoRequest, SendMessageRequest};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
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
        "export_conversation" => "export:conversation".to_string(),
        "import_role" => "import:role".to_string(),
        "delete_role" => "delete:role".to_string(),
        "update_settings" => "write:settings".to_string(),
        "get_conversation_list" => "read:conversations".to_string(),
        _ => cmd.to_string(),
    }
}

#[inline]
fn bridge_invalid(msg: impl Into<String>) -> String {
    ApiError::InvalidParameter {
        message: msg.into(),
    }
    .to_string()
}

#[inline]
fn bridge_bad_json(ctx: &str, e: serde_json::Error) -> String {
    ApiError::InvalidParameter {
        message: format!("{}: {}", ctx, e),
    }
    .to_string()
}

#[inline]
fn bridge_serialize_host(ctx: &str, e: serde_json::Error) -> String {
    ApiError::Io {
        message: format!("host json {}: {}", ctx, e),
    }
    .to_string()
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
            | "export_conversation"
            | "import_role"
            | "delete_role"
            | "update_settings"
            | "get_conversation_list"
    )
}

fn validate_shell_ocliveplugin(
    manifest: &OclivePluginManifest,
    asset_rel: &str,
) -> Result<(), String> {
    if manifest.plugin_type.as_deref().map(str::trim) != Some("ocliveplugin") {
        return Err(
            ApiError::PermissionDenied {
                message: "this command requires manifest \"type\": \"ocliveplugin\" and shell.bridge.invoke permission"
                    .into(),
            }
            .to_string(),
        );
    }
    let Some(sh) = &manifest.shell else {
        return Err(
            ApiError::PermissionDenied {
                message: "this command is only allowed for shell plugins".into(),
            }
            .to_string(),
        );
    };
    let rel = normalize_plugin_rel(asset_rel);
    let from_entry = rel == normalize_plugin_rel(&sh.entry);
    let from_vue = sh
        .vue_entry
        .as_ref()
        .map(|v| {
            let t = v.trim();
            !t.is_empty() && rel == normalize_plugin_rel(t)
        })
        .unwrap_or(false);
    if !from_entry && !from_vue {
        return Err(
            ApiError::PermissionDenied {
                message: "this command must be invoked from shell.entry or shell.vueEntry (not ui_slots)"
                    .into(),
            }
            .to_string(),
        );
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
    let root = roots.get(plugin_id).ok_or_else(|| {
        ApiError::PluginNotFound {
            plugin_id: plugin_id.to_string(),
        }
        .to_string()
    })?;
    let manifest = OclivePluginManifest::load_from_dir(root).map_err(|e| {
        ApiError::InvalidManifest {
            message: e,
        }
        .to_string()
    })?;
    let rel = normalize_plugin_rel(asset_rel);
    let Some(b) = manifest.bridge_for_asset_rel(&rel) else {
        return Err(
            ApiError::PermissionDenied {
                message: "asset has no bridge config".into(),
            }
            .to_string(),
        );
    };
    if !invoke_list_allows(&b.invoke, command) {
        let tok = required_permission_token(command);
        return Err(
            ApiError::PermissionDenied {
                message: format!(
                    "bridge.invoke must include command {:?} or permission {:?}",
                    command,
                    tok.as_str()
                ),
            }
            .to_string(),
        );
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
        return Err(bridge_invalid("send_message: user_message or text required"));
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
            serde_json::to_value(res).map_err(|e| bridge_serialize_host(command, e))
        }
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
                .ok_or_else(|| bridge_invalid("switch_role: role_id required"))?;
            let info = switch_role_impl(state, role_id).await?;
            serde_json::to_value(info).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_roles" => {
            let rows = list_roles_impl(state).await?;
            serde_json::to_value(rows).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_current_role" => {
            let req: GetRoleInfoRequest = if let Some(inner) = params.get("req") {
                serde_json::from_value(inner.clone())
                    .map_err(|e| bridge_bad_json("get_current_role.req", e))?
            } else {
                serde_json::from_value(params).map_err(|e| bridge_bad_json("get_current_role", e))?
            };
            let r = get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?;
            serde_json::to_value(r).map_err(|e| bridge_serialize_host(command, e))
        }
        "get_role_info" => {
            let req: GetRoleInfoRequest = if params.is_null() {
                return Err(bridge_invalid("get_role_info: missing params"));
            } else if let Some(inner) = params.get("req") {
                serde_json::from_value(inner.clone()).map_err(|e| bridge_bad_json("get_role_info.req", e))?
            } else {
                serde_json::from_value(params).map_err(|e| bridge_bad_json("get_role_info", e))?
            };
            let r = get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await?;
            serde_json::to_value(r).map_err(|e| bridge_serialize_host(command, e))
        }
        "list_roles" => {
            let rows = list_roles_impl(state).await?;
            serde_json::to_value(rows).map_err(|e| bridge_serialize_host(command, e))
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
        "get_directory_plugin_bootstrap" => {
            let role_id = params
                .get("roleId")
                .or_else(|| params.get("role_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let dto = directory_plugin_bootstrap_dto(state, role_id);
            serde_json::to_value(dto).map_err(|e| bridge_serialize_host(command, e))
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
                .ok_or_else(|| bridge_invalid("delete_memory: role_id required"))?;
            let memory_id = params
                .get("memory_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("delete_memory: memory_id required"))?;
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
        "import_role" => {
            let path = params
                .get("path")
                .or_else(|| params.get("src_path"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| bridge_invalid("import_role: path required"))?;
            let overwrite = params
                .get("overwrite")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let storage = state.storage.clone();
            let path_buf = PathBuf::from(path);
            let role_id = tokio::task::spawn_blocking(move || {
                import_role_pack(&storage, &path_buf, overwrite, |_| {})
            })
            .await
            .map_err(|e| {
                ApiError::Io {
                    message: format!("import_role join: {}", e),
                }
                .to_string()
            })?
            .map_err(|e: crate::error::AppError| e.to_frontend_error())?;
            state.invalidate_personality_cache_for_role(&role_id);
            let role = state
                .storage
                .load_role(&role_id)
                .map_err(|e| e.to_frontend_error())?;
            state
                .role_cache
                .write()
                .insert(role_id.clone(), Arc::new(role));
            Ok(json!({ "role_id": role_id, "ok": true }))
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
        _ => Err(
            ApiError::InvalidParameter {
                message: format!("unsupported bridge command: {}", command),
            }
            .to_string(),
        ),
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
        return Err(
            ApiError::InvalidParameter {
                message: "plugin_id, asset_rel, command required".into(),
            }
            .to_string(),
        );
    }
    validate_bridge(&state, pid, &asset, cmd)?;
    dispatch_bridge_command(&state, cmd, req.params).await
}
