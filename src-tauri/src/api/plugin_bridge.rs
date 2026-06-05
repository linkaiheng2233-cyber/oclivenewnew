//! Directory plugin page `OclivePluginBridge` → controlled host Tauri commands (manifest `shell.bridge.invoke` permission whitelist).
//!
//! - **Permission tokens**: some commands use `read:*` form (see `required_permission_token`); declaring **command name or matching permission** in the `invoke` array passes validation.
//! - **Full-shell deep integration**: `send_message` / `get_conversation` / `switch_role` / `get_roles` / `get_current_role` /
//!   `export_conversation` / `import_role` and write commands also require
//!   `manifest.type == "ocliveplugin"` and the request from **`shell.entry`** HTML or **`shell.vueEntry`** host Vue entry (not `ui_slots` pages).

use crate::api::chat_backend::ChatBackend;
use crate::api::directory_plugin::directory_plugin_bootstrap_dto;
use crate::api::error::ApiError;
use crate::api::error::CommandError;
use crate::infrastructure::directory_plugins::{normalize_plugin_rel, OclivePluginManifest};
use crate::infrastructure::import_role_pack;
use crate::kernel_attach::{role_dir_for_id, KernelHttpClient};
use crate::state::{AppState, SharedAppState};
use oclive_kernel_host::service::{
    bridge_command_needs_kernel_writer, dispatch_bridge_command, parse_send_message_request,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginBridgeInvokeRequest {
    pub plugin_id: String,
    pub asset_rel: String,
    pub command: String,
    #[serde(default)]
    pub params: Value,
}

/// Bridge command name → permission string required in manifest `bridge.invoke` (when different from command name, either match suffices).
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
fn bridge_invalid(msg: impl Into<String>) -> CommandError {
    CommandError::from(
        ApiError::InvalidParameter {
            message: msg.into(),
        }
        .to_string(),
    )
}

fn invoke_list_allows(invoke: &[String], cmd: &str) -> bool {
    let need = required_permission_token(cmd);
    invoke.iter().any(|x| {
        let t = x.trim();
        t == cmd || t == need.as_str()
    })
}

/// Sensitive commands allowed only from full-shell **`type: "ocliveplugin"`** **`shell.entry`**.
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
) -> Result<(), CommandError> {
    if manifest.plugin_type.as_deref().map(str::trim) != Some("ocliveplugin") {
        return Err(
            ApiError::PermissionDenied {
                message: "this command requires manifest \"type\": \"ocliveplugin\" and shell.bridge.invoke permission"
                    .into(),
            }
            .into(),
        );
    }
    let Some(sh) = &manifest.shell else {
        return Err(ApiError::PermissionDenied {
            message: "this command is only allowed for shell plugins".into(),
        }
        .into());
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
        return Err(ApiError::PermissionDenied {
            message:
                "this command must be invoked from shell.entry or shell.vueEntry (not ui_slots)"
                    .into(),
        }
        .into());
    }
    Ok(())
}

fn validate_bridge(
    state: &AppState,
    plugin_id: &str,
    asset_rel: &str,
    command: &str,
) -> Result<(), CommandError> {
    let roots = state.directory_plugins.plugin_roots.read();
    let root = roots
        .get(plugin_id)
        .map(|entry| &entry.root)
        .ok_or_else(|| ApiError::PluginNotFound {
            plugin_id: plugin_id.to_string(),
        })?;
    let manifest = OclivePluginManifest::load_from_dir(root)
        .map_err(|e| ApiError::InvalidManifest { message: e })?;
    let rel = normalize_plugin_rel(asset_rel);
    let Some(b) = manifest.bridge_for_asset_rel(&rel) else {
        return Err(ApiError::PermissionDenied {
            message: "asset has no bridge config".into(),
        }
        .into());
    };
    if !invoke_list_allows(&b.invoke, command) {
        let tok = required_permission_token(command);
        return Err(ApiError::PermissionDenied {
            message: format!(
                "bridge.invoke must include command {:?} or permission {:?}",
                command,
                tok.as_str()
            ),
        }
        .into());
    }
    if requires_typed_shell(command) {
        validate_shell_ocliveplugin(&manifest, &rel)?;
    }
    Ok(())
}

async fn dispatch_local_bridge_command(
    state: &AppState,
    backend: &ChatBackend,
    command: &str,
    params: Value,
) -> Result<Value, CommandError> {
    if command == "send_message" {
        let req = parse_send_message_request(&params)?;
        let role_path = role_dir_for_id(state, &req.role_id);
        let res = backend
            .send_message(&role_path, &req)
            .await
            .map_err(CommandError::from)?;
        return serde_json::to_value(res).map_err(|e| {
            CommandError::from(
                ApiError::Io {
                    message: format!("host json send_message: {e}"),
                }
                .to_string(),
            )
        });
    }

    if command == "get_directory_plugin_bootstrap" {
        let role_id = params
            .get("roleId")
            .or_else(|| params.get("role_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let dto = directory_plugin_bootstrap_dto(state, role_id);
        return serde_json::to_value(dto).map_err(|e| {
            CommandError::from(
                ApiError::Io {
                    message: format!("host json get_directory_plugin_bootstrap: {e}"),
                }
                .to_string(),
            )
        });
    }

    if command == "import_role" {
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
        })??;
        state.invalidate_personality_cache_for_role(&role_id);
        let role = state.storage.load_role(&role_id)?;
        state
            .role_cache
            .write()
            .insert(role_id.clone(), Arc::new(role));
        return Ok(json!({ "role_id": role_id, "ok": true }));
    }

    dispatch_bridge_command(state, command, params).await
}

async fn dispatch_bridge_command_routed(
    state: &AppState,
    backend: &ChatBackend,
    command: &str,
    params: Value,
) -> Result<Value, CommandError> {
    if let ChatBackend::Http(conn) = backend {
        if bridge_command_needs_kernel_writer(command) {
            return KernelHttpClient::bridge_dispatch_via_http(conn, command, params)
                .await
                .map_err(CommandError::from);
        }
    }
    dispatch_local_bridge_command(state, backend, command, params).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn plugin_bridge_invoke(
    req: PluginBridgeInvokeRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    let pid = req.plugin_id.trim();
    let asset = normalize_plugin_rel(req.asset_rel.trim());
    let cmd = req.command.trim();
    if pid.is_empty() || asset.is_empty() || cmd.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id, asset_rel, command required".into(),
        }
        .into());
    }
    validate_bridge(&state, pid, &asset, cmd)?;
    let backend = ChatBackend::from_app(&app, state.inner().clone());
    dispatch_bridge_command_routed(state.as_ref(), &backend, cmd, req.params).await
}
