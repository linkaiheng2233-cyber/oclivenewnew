//! Directory plugin private config: `get_plugin_settings_ui` / `set_plugin_settings_config`.

use crate::api::error::ApiError;
use crate::api::error::CommandError;
use oclive_kernel_host::infrastructure::directory_plugins::OclivePluginManifest;
use oclive_kernel_host::infrastructure::plugin_data::{
    ensure_default_config_for_manifest, read_config_json, write_config_json,
};
use oclive_kernel_host::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use oclive_kernel_host::state::{AppState, SharedAppState};
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

fn plugin_root(state: &AppState, plugin_id: &str) -> Result<PathBuf, CommandError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let roots = state.directory_plugins.plugin_roots.read();
    roots
        .get(pid)
        .map(|entry| entry.root.clone())
        .ok_or_else(|| {
            ApiError::PluginNotFound {
                plugin_id: pid.to_string(),
            }
            .into()
        })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub(crate) fn get_plugin_settings_ui_impl(
    state: &AppState,
    plugin_id: &str,
) -> Result<PluginUiSettingsDto, CommandError> {
    let root = plugin_root(state, plugin_id)?;
    let manifest = OclivePluginManifest::load_from_dir(&root)?;
    ensure_default_config_for_manifest(state, &manifest);
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
    let config = read_config_json(state, plugin_id.trim())?;
    Ok(PluginUiSettingsDto {
        ui_template,
        fields,
        config,
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub(crate) async fn set_plugin_settings_config_impl(
    state: &AppState,
    plugin_id: &str,
    config: &Value,
) -> Result<(), CommandError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let _root = plugin_root(state, pid)?;
    if !config.is_object() {
        return Err(ApiError::InvalidParameter {
            message: "config must be a JSON object".into(),
        }
        .into());
    }
    let transition =
        oclive_kernel_host::service::prepare_directory_plugin_resource_config_transition(
            state, pid, config,
        )
        .await;
    write_config_json(state, pid, config)?;
    let rpc_result = if let Ok(url) = state.directory_plugins.ensure_rpc_url(pid) {
        let timeout_ms = state
            .directory_plugins
            .rpc_timeout_override_ms(pid, "config_updated");
        let mut params = json!({ "config": config });
        if let Some(resource_transition) = transition.rpc_payload() {
            if let Some(object) = params.as_object_mut() {
                object.insert("resource_transition".into(), resource_transition);
            }
        }
        tokio::task::spawn_blocking(move || {
            invoke_directory_plugin_rpc_blocking(
                &url,
                "config_updated",
                params,
                RemoteRpcChannel::Plugin,
                timeout_ms,
            )
        })
        .await
        .ok()
        .and_then(Result::ok)
    } else {
        None
    };
    oclive_kernel_host::service::finalize_directory_plugin_resource_config_transition(
        state,
        pid,
        transition,
        rpc_result.as_ref(),
    );
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_plugin_settings_ui(
    plugin_id: String,
    state: State<'_, SharedAppState>,
) -> Result<PluginUiSettingsDto, CommandError> {
    get_plugin_settings_ui_impl(&state, &plugin_id)
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_plugin_settings_config(
    plugin_id: String,
    config: Value,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    set_plugin_settings_config_impl(&state, &plugin_id, &config).await
}
