//! Directory plugin page `OclivePluginBridge` → controlled host Tauri commands (manifest `shell.bridge.invoke` permission whitelist).
//!
//! - **Permission tokens**: some commands use `read:*` form (see `required_permission_token`); declaring **command name or matching permission** in the `invoke` array passes validation.
//! - **Full-shell deep integration**: `send_message` / `get_conversation` / `switch_role` / `get_roles` / `get_current_role` /
//!   `export_conversation` / `import_role` and write commands also require
//!   `manifest.type == "ocliveplugin"` and the request from **`shell.entry`** HTML or **`shell.vueEntry`** host Vue entry (not `ui_slots` pages).

use crate::api::chat_backend::ChatBackend;
use crate::api::directory_plugin::directory_plugin_bootstrap_dto;
use crate::api::error::{map_directory_rpc_url_error, ApiError, CommandError};
use crate::api::plugin_config::{get_plugin_settings_ui_impl, set_plugin_settings_config_impl};
use crate::kernel_attach::{role_dir_for_id, KernelHttpClient};
use oclive_kernel_host::infrastructure::directory_plugins::{
    normalize_plugin_rel, OclivePluginManifest,
};
use oclive_kernel_host::infrastructure::import_role_pack;
use oclive_kernel_host::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use oclive_kernel_host::infrastructure::role_pack::validate_bridge_import_role_source;
use oclive_kernel_host::service::{
    bridge_command_needs_kernel_writer, dispatch_bridge_command, parse_send_message_request,
};
use oclive_kernel_host::state::{AppState, SharedAppState};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
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

fn validate_plugin_rpc_method(
    state: &AppState,
    plugin_id: &str,
    method: &str,
) -> Result<(), CommandError> {
    let pid = plugin_id.trim();
    let method = method.trim();
    if pid.is_empty() || method.is_empty() {
        return Err(bridge_invalid(
            "plugin_rpc_invoke: plugin_id and method required",
        ));
    }
    let roots = state.directory_plugins.plugin_roots.read();
    let entry = roots.get(pid).ok_or_else(|| ApiError::PluginNotFound {
        plugin_id: pid.to_string(),
    })?;
    let manifest = OclivePluginManifest::load_from_dir(&entry.root)
        .map_err(|e| ApiError::InvalidManifest { message: e })?;
    validate_rpc_method_for_manifest(&manifest, method)
}

fn validate_rpc_method_for_manifest(
    manifest: &OclivePluginManifest,
    method: &str,
) -> Result<(), CommandError> {
    if manifest.process.is_none() {
        return Err(ApiError::PermissionDenied {
            message: "plugin has no process block for RPC".into(),
        }
        .into());
    }
    if !manifest.rpc_methods.iter().any(|m| m.trim() == method) {
        return Err(ApiError::PermissionDenied {
            message: format!("method {:?} not declared in manifest rpcMethods", method),
        }
        .into());
    }
    Ok(())
}

async fn dispatch_plugin_rpc_invoke(
    shared: SharedAppState,
    plugin_id: &str,
    params: Value,
) -> Result<Value, CommandError> {
    let method = params
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if method.is_empty() {
        return Err(bridge_invalid("plugin_rpc_invoke: method required"));
    }
    let rpc_params = params.get("params").cloned().unwrap_or(Value::Null);
    validate_plugin_rpc_method(shared.as_ref(), plugin_id, &method)?;
    let pid = plugin_id.trim().to_string();
    tokio::task::spawn_blocking(move || {
        let url = shared
            .directory_plugins
            .ensure_rpc_url(&pid)
            .map_err(|e| map_directory_rpc_url_error(&pid, e))?;
        let timeout_ms = shared
            .directory_plugins
            .rpc_timeout_override_ms(&pid, &method);
        invoke_directory_plugin_rpc_blocking(
            &url,
            &method,
            rpc_params,
            RemoteRpcChannel::Plugin,
            timeout_ms,
        )
        .map_err(Into::into)
    })
    .await
    .map_err(|e| {
        CommandError::from(
            ApiError::Io {
                message: format!("plugin_rpc_invoke join: {e}"),
            }
            .to_string(),
        )
    })?
}

async fn dispatch_local_bridge_command(
    state: &AppState,
    backend: &ChatBackend,
    command: &str,
    params: Value,
    _bridge_plugin_id: &str,
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

    if command == "get_plugin_settings_ui" {
        let plugin_id = params
            .get("pluginId")
            .or_else(|| params.get("plugin_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| bridge_invalid("get_plugin_settings_ui: pluginId required"))?;
        let dto = get_plugin_settings_ui_impl(state, plugin_id)?;
        return serde_json::to_value(dto).map_err(|e| {
            CommandError::from(
                ApiError::Io {
                    message: format!("host json get_plugin_settings_ui: {e}"),
                }
                .to_string(),
            )
        });
    }

    if command == "set_plugin_settings_config" {
        let plugin_id = params
            .get("pluginId")
            .or_else(|| params.get("plugin_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| bridge_invalid("set_plugin_settings_config: pluginId required"))?;
        let config = params.get("config").cloned().unwrap_or(Value::Null);
        set_plugin_settings_config_impl(state, plugin_id, &config)?;
        return Ok(json!({ "ok": true }));
    }

    if command == "get_role_pack_path" {
        let role_id = params
            .get("roleId")
            .or_else(|| params.get("role_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| bridge_invalid("get_role_pack_path: roleId required"))?;
        let path = role_dir_for_id(state, role_id.trim());
        return Ok(json!({ "role_path": path.to_string_lossy() }));
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
        let app_data_dir = oclive_kernel_runtime::find_app_data_dir_for_host();
        let path_buf = validate_bridge_import_role_source(&storage, &app_data_dir, Path::new(path))
            .map_err(|e| e.to_string())?;
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
    state: &SharedAppState,
    backend: &ChatBackend,
    command: &str,
    params: Value,
    bridge_plugin_id: &str,
) -> Result<Value, CommandError> {
    if command == "plugin_rpc_invoke" {
        return dispatch_plugin_rpc_invoke(Arc::clone(state), bridge_plugin_id, params).await;
    }
    if let ChatBackend::Http(conn) = backend {
        if bridge_command_needs_kernel_writer(command) {
            return KernelHttpClient::bridge_dispatch_via_http(conn, command, params)
                .await
                .map_err(CommandError::from);
        }
    }
    dispatch_local_bridge_command(state.as_ref(), backend, command, params, bridge_plugin_id).await
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
    dispatch_bridge_command_routed(state.inner(), &backend, cmd, req.params, pid).await
}

#[cfg(test)]
mod rpc_validation_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_manifest(root: &std::path::Path, rpc_methods: &[&str], with_process: bool) {
        let methods = rpc_methods
            .iter()
            .map(|m| format!("\"{m}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let process = if with_process {
            r#""process": { "command": "node", "args": ["rpc_server.mjs"] },"#
        } else {
            ""
        };
        let json = format!(
            r#"{{
  "schema_version": 1,
  "id": "com.test.voice",
  "version": "1.0.0",
  {process}
  "rpcMethods": [{methods}]
}}"#
        );
        fs::write(root.join("manifest.json"), json).expect("write manifest");
    }

    #[test]
    fn declared_rpc_method_passes_manifest_whitelist() {
        let tmp = TempDir::new().expect("temp");
        write_manifest(
            tmp.path(),
            &[
                "voice.probe",
                "voice.transcribe",
                "voice.speak",
                "voice.build_directive",
                "voice.import_tts_adapter",
                "voice.list_tts_adapters",
            ],
            true,
        );
        let manifest = OclivePluginManifest::load_from_dir(tmp.path()).expect("load manifest");
        assert!(validate_rpc_method_for_manifest(&manifest, "voice.transcribe").is_ok());
        assert!(validate_rpc_method_for_manifest(&manifest, "voice.build_directive").is_ok());
        assert!(validate_rpc_method_for_manifest(&manifest, "voice.import_tts_adapter").is_ok());
        assert!(validate_rpc_method_for_manifest(&manifest, "voice.list_tts_adapters").is_ok());
    }

    #[test]
    fn voice_asr_manifest_declares_tts_adapter_rpc() {
        let repo = path_from_manifest_dir();
        let manifest =
            OclivePluginManifest::load_from_dir(&repo).expect("load com.oclive.voice.asr manifest");
        for method in ["voice.import_tts_adapter", "voice.list_tts_adapters"] {
            assert!(
                manifest.rpc_methods.iter().any(|m| m == method),
                "missing rpcMethods {method}"
            );
            assert!(
                validate_rpc_method_for_manifest(&manifest, method).is_ok(),
                "whitelist rejected {method}"
            );
        }
    }

    fn path_from_manifest_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../chat-pro/plugins/com.oclive.voice.asr")
            .canonicalize()
            .expect("voice.asr plugin dir")
    }

    #[test]
    fn undeclared_rpc_method_rejected() {
        let tmp = TempDir::new().expect("temp");
        write_manifest(tmp.path(), &["voice.probe"], true);
        let manifest = OclivePluginManifest::load_from_dir(tmp.path()).expect("load manifest");
        let err = validate_rpc_method_for_manifest(&manifest, "voice.speak")
            .expect_err("undeclared method");
        assert!(err.to_string().contains("not declared"));
    }

    #[test]
    fn manifest_without_process_rejects_rpc() {
        let tmp = TempDir::new().expect("temp");
        write_manifest(tmp.path(), &["voice.probe"], false);
        let manifest = OclivePluginManifest::load_from_dir(tmp.path()).expect("load manifest");
        let err =
            validate_rpc_method_for_manifest(&manifest, "voice.probe").expect_err("no process");
        assert!(err.to_string().contains("no process"));
    }
}
