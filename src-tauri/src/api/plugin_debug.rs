//! V1 pro mode: directory plugin developer debug panel backend commands.

use crate::api::error::ApiError;
use crate::infrastructure::directory_plugins::{OclivePluginManifest, PluginProcessDebugInfo};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::state::SharedAppState;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;
use crate::api::error::CommandError;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn spawn_plugin_for_test(
    plugin_id: String,
    config_json: Option<String>,
    state: State<'_, SharedAppState>,
) -> Result<PluginProcessDebugInfo, CommandError> {
    let pid = plugin_id.trim().to_string();
    let cfg = config_json;
    let shared = state.inner().clone();
    tokio::task::spawn_blocking(move || {
        shared
            .directory_plugins
            .spawn_plugin_for_test(&pid, cfg.as_deref())
            .map_err(Into::into)
    })
    .await
    .map_err(|e| crate::error::AppError::Unknown(format!("spawn_plugin_for_test join: {e}")))?
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn kill_plugin_process(plugin_id: String, state: State<'_, SharedAppState>) -> Result<(), CommandError> {
    let id = plugin_id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    state.directory_plugins.clear_plugin_process(id);
    Ok(())
}

#[tauri::command]
#[must_use]
pub fn list_plugin_processes(state: State<'_, SharedAppState>) -> Vec<PluginProcessDebugInfo> {
    state.directory_plugins.list_managed_plugin_processes()
}

#[tauri::command]
#[must_use]
pub fn get_plugin_logs(plugin_id: String, lines: usize, state: State<'_, SharedAppState>) -> Vec<String> {
    state
        .directory_plugins
        .get_plugin_log_tail(plugin_id.trim(), lines.max(1))
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn clear_plugin_logs(plugin_id: String, state: State<'_, SharedAppState>) -> Result<(), CommandError> {
    let id = plugin_id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    state.directory_plugins.clear_plugin_log_buffer(id);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPluginMethodDto {
    pub plugin_id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn test_plugin_method(
    req: TestPluginMethodDto,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let method = req.method.trim();
    if method.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "method required".into(),
        }
        .into());
    }
    let pid_owned = pid.to_string();
    let method_owned = method.to_string();
    let params = req.params;
    let shared = state.inner().clone();
    tokio::task::spawn_blocking(move || {
        let url = shared
            .directory_plugins
            .ensure_rpc_url_for_debug(&pid_owned, None)?;
        invoke_directory_plugin_rpc_blocking(&url, &method_owned, params, RemoteRpcChannel::Plugin)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| crate::error::AppError::Unknown(format!("test_plugin_method join: {e}")))?
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn discover_plugin_methods(
    plugin_id: String,
    state: State<'_, SharedAppState>,
) -> Result<Vec<String>, CommandError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let pid_owned = pid.to_string();
    let shared = state.inner().clone();
    tokio::task::spawn_blocking(move || discover_plugin_methods_blocking(&shared, &pid_owned))
        .await
        .map_err(|e| crate::error::AppError::Unknown(format!("discover_plugin_methods join: {e}")))?
}

fn discover_plugin_methods_blocking(
    state: &crate::state::AppState,
    pid: &str,
) -> Result<Vec<String>, CommandError> {
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .map(|entry| entry.root.clone())
            .ok_or_else(|| ApiError::PluginNotFound {
                plugin_id: pid.to_string(),
            })?
    };
    let manifest = OclivePluginManifest::load_from_dir(&root)?;
    let mut out: Vec<String> = manifest
        .rpc_methods
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let url = match state
        .directory_plugins
        .ensure_rpc_url_for_debug(pid, None)
    {
        Ok(u) => u,
        Err(_) => {
            out.sort_unstable();
            out.dedup();
            return Ok(out);
        }
    };

    if let Ok(v) = invoke_directory_plugin_rpc_blocking(
        &url,
        "rpc.discover",
        json!({}),
        RemoteRpcChannel::Plugin,
    ) {
        merge_discovered_methods(&mut out, &v);
    }

    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn merge_discovered_methods(out: &mut Vec<String>, v: &Value) {
    if let Some(arr) = v.as_array() {
        for x in arr {
            if let Some(s) = x.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
        }
        return;
    }
    if let Some(obj) = v.as_object() {
        if let Some(arr) = obj.get("methods").and_then(|x| x.as_array()) {
            for x in arr {
                if let Some(s) = x.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                }
            }
        }
    }
}
