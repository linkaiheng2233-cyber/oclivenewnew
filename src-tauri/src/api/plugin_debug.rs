//! V1 专业模式：目录插件「开发者调试面板」后端命令。

use crate::api::error::ApiError;
use crate::error::AppError;
use crate::infrastructure::directory_plugins::{OclivePluginManifest, PluginProcessDebugInfo};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub fn spawn_plugin_for_test(
    plugin_id: String,
    config_json: Option<String>,
    state: State<'_, AppState>,
) -> Result<PluginProcessDebugInfo, String> {
    let cfg = config_json.as_deref();
    state
        .directory_plugins
        .spawn_plugin_for_test(plugin_id.trim(), cfg)
}

#[tauri::command]
pub fn kill_plugin_process(plugin_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id = plugin_id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    state.directory_plugins.clear_plugin_process(id);
    Ok(())
}

#[tauri::command]
pub fn list_plugin_processes(state: State<'_, AppState>) -> Vec<PluginProcessDebugInfo> {
    state.directory_plugins.list_managed_plugin_processes()
}

#[tauri::command]
pub fn get_plugin_logs(plugin_id: String, lines: usize, state: State<'_, AppState>) -> Vec<String> {
    state
        .directory_plugins
        .get_plugin_log_tail(plugin_id.trim(), lines.max(1))
}

#[tauri::command]
pub fn clear_plugin_logs(plugin_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id = plugin_id.trim();
    if id.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
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

#[tauri::command]
pub fn test_plugin_method(
    req: TestPluginMethodDto,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let method = req.method.trim();
    if method.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "method required".into(),
        }
        .to_string());
    }
    let url = state
        .directory_plugins
        .ensure_rpc_url_for_debug(pid, None)
        .map_err(|e| e)?;
    invoke_directory_plugin_rpc_blocking(&url, method, req.params, RemoteRpcChannel::Plugin)
        .map_err(|e: AppError| e.to_frontend_error())
}

#[tauri::command]
pub fn discover_plugin_methods(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots.get(pid).cloned().ok_or_else(|| {
            ApiError::PluginNotFound {
                plugin_id: pid.to_string(),
            }
            .to_string()
        })?
    };
    let manifest = OclivePluginManifest::load_from_dir(&root)?;
    let mut out: Vec<String> = manifest
        .rpc_methods
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let url = match state.directory_plugins.ensure_rpc_url_for_debug(pid, None) {
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
