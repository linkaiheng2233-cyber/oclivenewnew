//! 目录式插件：启动引导与 JSON-RPC 透传（B2）。

use crate::error::AppError;
use crate::infrastructure::directory_plugins::{HostPluginsFile, OclivePluginManifest};
use crate::infrastructure::remote_plugin::{invoke_directory_plugin_rpc_blocking, RemoteRpcChannel};
use crate::state::AppState;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginBootstrapDto {
    pub shell_url: Option<String>,
    pub shell_plugin_id: Option<String>,
    pub plugin_ids: Vec<String>,
    pub developer_mode: bool,
}

fn shell_plugin_id_from_host(host: &HostPluginsFile) -> Option<String> {
    if let Ok(v) = std::env::var("OCLIVE_SHELL_PLUGIN_ID") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    host.shell_plugin_id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[tauri::command]
pub fn get_directory_plugin_bootstrap(state: State<'_, AppState>) -> Result<DirectoryPluginBootstrapDto, String> {
    let rt = &state.directory_plugins;
    let host = rt.host();
    let mut plugin_ids_sorted: Vec<String> = rt.plugin_roots.read().keys().cloned().collect();
    plugin_ids_sorted.sort_unstable();
    let shell_plugin_id = shell_plugin_id_from_host(host);
    let shell_url = shell_plugin_id.as_ref().and_then(|pid| {
        let roots = rt.plugin_roots.read();
        let root = roots.get(pid)?;
        let manifest = OclivePluginManifest::load_from_dir(root).ok()?;
        let entry = manifest.shell.as_ref()?.entry.as_str();
        rt.shell_url_for(pid, entry)
    });
    Ok(DirectoryPluginBootstrapDto {
        shell_url,
        shell_plugin_id,
        plugin_ids: plugin_ids_sorted,
        developer_mode: host.developer_effective(),
    })
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginInvokeDto {
    pub plugin_id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[tauri::command]
pub fn directory_plugin_invoke(
    req: DirectoryPluginInvokeDto,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id empty".to_string());
    }
    let url = state.directory_plugins.ensure_rpc_url(pid)?;
    invoke_directory_plugin_rpc_blocking(
        &url,
        req.method.trim(),
        req.params,
        RemoteRpcChannel::Plugin,
    )
    .map_err(|e: AppError| e.to_frontend_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn invoke_dto_defaults_params() {
        let raw = json!({"pluginId": "p1", "method": "x"});
        let v: DirectoryPluginInvokeDto = serde_json::from_value(raw).expect("parse");
        assert_eq!(v.params, Value::Null);
    }
}
