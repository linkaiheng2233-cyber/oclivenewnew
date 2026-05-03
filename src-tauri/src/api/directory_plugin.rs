//! 目录式插件：启动引导与 JSON-RPC 透传（B2）。

use crate::api::error::ApiError;
use crate::domain::directory_plugin_commands;
use crate::error::AppError;
use crate::infrastructure::directory_plugins::{
    build_directory_plugin_catalog as kernel_build_directory_plugin_catalog,
    directory_plugin_bootstrap_dto as kernel_directory_plugin_bootstrap_dto,
    is_host_event_subscribed as kernel_is_host_event_subscribed,
    plugin_scan_container_roots,
    read_plugin_asset_text_under_root,
    DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
};
use crate::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::models::dto::{DirectoryPluginBootstrapDto, DirectoryPluginCatalogEntry};
use crate::state::AppState;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use tauri::State;

/// 供 `get_directory_plugin_bootstrap` 与 `plugin_bridge_invoke` 共用。
/// `role_id`：当前角色；省略时尝试 `oclive_last_role_id.txt`，再回退旧版全局插件状态。
pub fn directory_plugin_bootstrap_dto(
    state: &AppState,
    role_id: Option<String>,
) -> DirectoryPluginBootstrapDto {
    kernel_directory_plugin_bootstrap_dto(
        &state.directory_plugins,
        role_id,
        DEFAULT_DIRECTORY_PLUGIN_ASSET_BASE_URL,
    )
}

#[tauri::command]
pub fn get_directory_plugin_bootstrap(
    role_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<DirectoryPluginBootstrapDto, String> {
    Ok(directory_plugin_bootstrap_dto(&state, role_id))
}

/// 读取目录插件根下文本文件（用于宿主侧编译 `.vue` 等）；路径不得越出插件目录。
#[tauri::command]
pub fn read_plugin_asset_text(
    plugin_id: String,
    rel: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let roots = state.directory_plugins.plugin_roots.read();
    let root = roots.get(pid).ok_or_else(|| {
        ApiError::PluginNotFound {
            plugin_id: pid.to_string(),
        }
        .to_string()
    })?;
    read_plugin_asset_text_under_root(root, rel.trim()).map_err(map_read_plugin_asset_err)
}

fn map_read_plugin_asset_err(e: AppError) -> String {
    match e {
        AppError::InvalidParameter(m) => ApiError::InvalidParameter { message: m }.to_string(),
        AppError::PermissionDenied(m) => ApiError::PermissionDenied { message: m }.to_string(),
        AppError::IoError(io) => ApiError::Io {
            message: io.to_string(),
        }
        .to_string(),
        other => ApiError::Io {
            message: other.to_string(),
        }
        .to_string(),
    }
}

/// 查询某宿主内置事件名是否被当前角色下已启用插件订阅（与 `subscribed_host_events` 一致）。
#[tauri::command]
pub fn is_host_event_subscribed(
    event: String,
    role_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let rt = &state.directory_plugins;
    let rid = role_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| rt.read_last_role_id_from_disk());
    let Some(rid) = rid else {
        return Ok(false);
    };
    let role_state = rt.role_plugin_state_for(rid.trim());
    let roots = rt.plugin_roots.read();
    Ok(kernel_is_host_event_subscribed(
        &roots,
        &role_state.slots,
        event.as_str(),
    ))
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
pub async fn directory_plugin_invoke(
    req: DirectoryPluginInvokeDto,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .to_string());
    }
    let perm = "rpc:invoke";
    let ok = state
        .db_manager
        .is_plugin_permission_granted(pid, perm)
        .await
        .unwrap_or(false);
    if !ok {
        let _ = state
            .db_manager
            .insert_plugin_audit_log(pid, "rpc.invoke", Some(perm), false, "{}")
            .await;
        return Err(ApiError::PluginPermissionNotGranted {
            message: format!("permission {:?} not granted for plugin {}", perm, pid),
        }
        .to_string());
    }
    let pid_s = pid.to_string();
    let method = req.method.trim().to_string();
    let params = req.params;
    let dir = state.directory_plugins.clone();
    let out = tokio::task::spawn_blocking(move || {
        let url = dir
            .ensure_rpc_url(pid_s.as_str())
            .map_err(|e| crate::api::error::map_directory_rpc_url_error(pid_s.as_str(), e))?;
        invoke_directory_plugin_rpc_blocking(
            &url,
            method.as_str(),
            params,
            RemoteRpcChannel::Plugin,
        )
        .map_err(|e: AppError| e.to_frontend_error())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = state
        .db_manager
        .insert_plugin_audit_log(pid, "rpc.invoke", Some(perm), true, "{}")
        .await;
    Ok(out)
}

struct PluginCatalogCacheValue {
    fingerprint: u64,
    stored_at: Instant,
    entries: Vec<DirectoryPluginCatalogEntry>,
}

static PLUGIN_CATALOG_CACHE: Lazy<Mutex<Option<PluginCatalogCacheValue>>> =
    Lazy::new(|| Mutex::new(None));

fn plugin_catalog_fingerprint(state: &AppState) -> std::io::Result<u64> {
    let roles = state.storage.roles_dir();
    let app_data = state.directory_plugins.app_data_dir();
    let host = state.directory_plugins.host();
    let roots = plugin_scan_container_roots(roles, app_data, &host);
    let mut h = DefaultHasher::new();
    state
        .directory_plugins
        .catalog_cache_invalidation_gen()
        .hash(&mut h);
    for r in roots {
        r.hash(&mut h);
        if let Ok(meta) = std::fs::metadata(&r) {
            if let Ok(t) = meta.modified() {
                if let Ok(d) = t.duration_since(std::time::UNIX_EPOCH) {
                    d.as_secs().hash(&mut h);
                    d.subsec_nanos().hash(&mut h);
                }
            }
        }
    }
    Ok(h.finish())
}

fn build_directory_plugin_catalog_local(state: &AppState) -> Vec<DirectoryPluginCatalogEntry> {
    let rt = &state.directory_plugins;
    rt.ensure_scanned();
    let roots = rt.plugin_roots.read();
    kernel_build_directory_plugin_catalog(&roots)
}

#[tauri::command]
pub fn get_directory_plugin_catalog(
    state: State<'_, AppState>,
) -> Result<Vec<DirectoryPluginCatalogEntry>, String> {
    let fp = plugin_catalog_fingerprint(&state).map_err(|e| {
        ApiError::Io {
            message: e.to_string(),
        }
        .to_string()
    })?;
    {
        let lock = PLUGIN_CATALOG_CACHE.lock();
        if let Some(cached) = lock.as_ref() {
            if cached.fingerprint == fp && cached.stored_at.elapsed() < Duration::from_secs(5) {
                return Ok(cached.entries.clone());
            }
        }
    }
    let out = build_directory_plugin_catalog_local(&state);
    *PLUGIN_CATALOG_CACHE.lock() = Some(PluginCatalogCacheValue {
        fingerprint: fp,
        stored_at: Instant::now(),
        entries: out.clone(),
    });
    Ok(out)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePluginStateDto {
    #[serde(default)]
    pub shell_plugin_id: String,
    #[serde(flatten)]
    pub slots: PluginStateFile,
}

impl From<RolePluginState> for RolePluginStateDto {
    fn from(r: RolePluginState) -> Self {
        Self {
            shell_plugin_id: r.shell_plugin_id,
            slots: r.slots,
        }
    }
}

impl From<RolePluginStateDto> for RolePluginState {
    fn from(d: RolePluginStateDto) -> Self {
        Self {
            shell_plugin_id: d.shell_plugin_id,
            slots: d.slots,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStateGetResponse {
    pub role: RolePluginStateDto,
    pub global_defaults: RolePluginStateDto,
}

#[tauri::command]
pub fn get_plugin_state(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<PluginStateGetResponse, String> {
    let rt = &state.directory_plugins;
    let rid = role_id.trim();
    Ok(PluginStateGetResponse {
        role: rt.role_plugin_state_stored_for(rid).into(),
        global_defaults: rt.global_plugin_state().into(),
    })
}

#[tauri::command]
pub fn save_plugin_state(
    role_id: String,
    state: RolePluginStateDto,
    app: State<'_, AppState>,
) -> Result<(), String> {
    app.directory_plugins
        .save_role_plugin_state(role_id.trim(), state.into())
}

#[tauri::command]
pub fn save_global_plugin_state(
    state: RolePluginStateDto,
    app: State<'_, AppState>,
) -> Result<(), String> {
    app.directory_plugins.save_global_plugin_state(state.into())
}

#[tauri::command]
pub fn reset_plugin_state_to_role_default(
    role_id: String,
    app: State<'_, AppState>,
) -> Result<(), String> {
    directory_plugin_commands::reset_role_plugin_state_to_pack_default(&app, role_id.trim())
        .map_err(|e| e.to_frontend_error())
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
