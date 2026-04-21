//! 目录插件私有配置：`get_plugin_settings_ui` / `set_plugin_settings_config`。

use crate::api::error::ApiError;
use crate::infrastructure::directory_plugins::OclivePluginManifest;
use crate::infrastructure::plugin_data::{ensure_default_config_for_manifest, read_config_json, write_config_json};
use crate::infrastructure::remote_plugin::{invoke_directory_plugin_rpc_blocking, RemoteRpcChannel};
use crate::state::AppState;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSchemaFieldDto {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUiSettingsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_template: Option<String>,
    #[serde(default)]
    pub fields: Vec<UiSchemaFieldDto>,
    pub config: Value,
}

fn plugin_root(state: &AppState, plugin_id: &str) -> Result<PathBuf, String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let roots = state.directory_plugins.plugin_roots.read();
    roots
        .get(pid)
        .cloned()
        .ok_or_else(|| ApiError::PluginNotFound { plugin_id: pid.to_string() }.to_string())
}

#[tauri::command]
pub fn get_plugin_settings_ui(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<PluginUiSettingsDto, String> {
    let root = plugin_root(&state, &plugin_id)?;
    let manifest = OclivePluginManifest::load_from_dir(&root).map_err(|e| e.to_string())?;
    ensure_default_config_for_manifest(&state, &manifest);
    let ui_template = manifest.ui_template.clone();
    let fields: Vec<UiSchemaFieldDto> = manifest
        .ui_schema
        .as_ref()
        .map(|s| {
            s.fields
                .iter()
                .map(|f| UiSchemaFieldDto {
                    key: f.key.clone(),
                    label: f.label.clone(),
                    field_type: f.field_type.clone(),
                    required: f.required,
                    default: f.default.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let config = read_config_json(&state, plugin_id.trim()).map_err(|e| e)?;
    Ok(PluginUiSettingsDto {
        ui_template,
        fields,
        config,
    })
}

#[tauri::command]
pub fn set_plugin_settings_config(
    plugin_id: String,
    config: Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let _root = plugin_root(&state, pid)?;
    if !config.is_object() {
        return Err(ApiError::InvalidParameter {
            message: "config must be a JSON object".into(),
        }
        .to_string());
    }
    write_config_json(&state, pid, &config).map_err(|e| e)?;
    if let Ok(url) = state.directory_plugins.ensure_rpc_url(pid) {
        let _ = invoke_directory_plugin_rpc_blocking(
            &url,
            "config_updated",
            json!({ "config": config }),
            RemoteRpcChannel::Plugin,
        );
    }
    Ok(())
}
