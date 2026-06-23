//! Directory plugins: bootstrap and JSON-RPC passthrough (B2).

use crate::api::error::ApiError;
use crate::api::error::{map_directory_rpc_url_error, CommandError};
use oclive_kernel_host::infrastructure::directory_plugins::{
    bootstrap_dto::{self, collect_subscribed_host_events},
    dependency_report, find_plugin_asset_path, normalize_plugin_rel,
    normalize_ui_slot_appearance_id, parse_manifest_version, plugin_scan_container_roots,
};
use oclive_kernel_host::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use oclive_kernel_host::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use oclive_kernel_host::state::{AppState, SharedAppState};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use tauri::State;

pub use oclive_kernel_host::infrastructure::directory_plugins::bootstrap_dto::{
    directory_plugin_bootstrap_dto, DirectoryPluginBootstrapDto, PluginUiSlotDto,
};
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_directory_plugin_bootstrap(
    role_id: Option<String>,
    state: State<'_, SharedAppState>,
) -> Result<DirectoryPluginBootstrapDto, CommandError> {
    Ok(directory_plugin_bootstrap_dto(&state, role_id))
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Read a text file under the directory plugin root (e.g. host-side `.vue` compile); path must not escape the plugin directory.
#[tauri::command]
pub fn read_plugin_asset_text(
    plugin_id: String,
    rel: String,
    state: State<'_, SharedAppState>,
) -> Result<String, CommandError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let rel = normalize_plugin_rel(rel.trim());
    if rel.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "rel required".into(),
        }
        .into());
    }
    if rel.split('/').any(|p| p == "..") {
        return Err(ApiError::InvalidParameter {
            message: "invalid rel path".into(),
        }
        .into());
    }
    let roots = state.directory_plugins.plugin_roots.read();
    let entry = roots.get(pid).ok_or_else(|| ApiError::PluginNotFound {
        plugin_id: pid.to_string(),
    })?;
    let path_canon = find_plugin_asset_path(entry, &rel).map_err(|e| {
        if e == "path escapes plugin directory" {
            ApiError::PermissionDenied {
                message: "path escapes plugin directory".into(),
            }
        } else {
            ApiError::Io {
                message: format!("read_plugin_asset_text: {}", e),
            }
        }
    })?;
    Ok(
        std::fs::read_to_string(&path_canon).map_err(|e| ApiError::Io {
            message: e.to_string(),
        })?,
    )
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Whether a built-in host event name is subscribed by an enabled plugin for the current role (same as `subscribed_host_events`).
#[tauri::command]
pub fn is_host_event_subscribed(
    event: String,
    role_id: Option<String>,
    state: State<'_, SharedAppState>,
) -> Result<bool, CommandError> {
    let ev = event.trim();
    if ev.is_empty() {
        return Ok(false);
    }
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
    let subs = collect_subscribed_host_events(&state, &role_state.slots);
    Ok(subs.iter().any(|s| s == ev))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginInvokeDto {
    pub plugin_id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn directory_plugin_invoke(
    req: DirectoryPluginInvokeDto,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    let pid = req.plugin_id.trim().to_string();
    if pid.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "plugin_id required".into(),
        }
        .into());
    }
    let method = req.method.trim().to_string();
    let params = req.params;
    let shared = state.inner().clone();
    tokio::task::spawn_blocking(move || {
        let url = shared
            .directory_plugins
            .ensure_rpc_url(&pid)
            .map_err(|e| map_directory_rpc_url_error(&pid, e))?;
        invoke_directory_plugin_rpc_blocking(&url, &method, params, RemoteRpcChannel::Plugin)
            .map_err(Into::into)
    })
    .await
    .map_err(|e| crate::error::AppError::Unknown(format!("directory_plugin_invoke join: {e}")))?
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSlotVariantDto {
    pub slot: String,
    pub appearance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginCatalogEntry {
    pub id: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_type: Option<String>,
    /// When manifest has `uiTemplate` or `uiSchema.fields`, host may edit private `config.json`.
    pub has_ui_settings: bool,
    /// Whether manifest declares `process` (JSON-RPC subprocess can be started from this panel).
    pub has_rpc_process: bool,
    /// Whether manifest declares `rpcMethods` (prefill debug panel; RPC to running instance still works without `process`).
    pub declares_rpc_methods: bool,
    pub is_shell: bool,
    /// Declared UI slot names (e.g. `chat_toolbar`); slot name appears once even with multiple appearances.
    pub ui_slot_names: Vec<String>,
    /// One entry per manifest `ui_slots` embed slot, with `appearance_id` / `label`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_slot_variants: Vec<UiSlotVariantDto>,
    pub provides: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    /// `ok` / `missing` / `mismatch`
    pub dependency_status: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_issues: Vec<String>,
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
    let roots = plugin_scan_container_roots(roles, app_data, host);
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

fn build_directory_plugin_catalog(state: &AppState) -> Vec<DirectoryPluginCatalogEntry> {
    let rt = &state.directory_plugins;
    let roots = rt.plugin_roots.read();
    let mut version_by_id: HashMap<String, Version> = HashMap::new();
    for (pid, entry) in roots.iter() {
        if let Ok(m) = rt.load_manifest_cached(pid, &entry.root) {
            if let Some(v) = parse_manifest_version(&m.version) {
                version_by_id.insert(pid.clone(), v);
            }
        }
    }
    let mut out: Vec<DirectoryPluginCatalogEntry> = roots
        .iter()
        .filter_map(|(pid, entry)| {
            let manifest = rt.load_manifest_cached(pid, &entry.root).ok()?;
            let is_shell = manifest.shell.is_some();
            let has_ui_settings = manifest.ui_template.is_some()
                || manifest
                    .ui_schema
                    .as_ref()
                    .map(|s| !s.fields.is_empty())
                    .unwrap_or(false);
            let has_rpc_process = manifest.process.is_some();
            let declares_rpc_methods = !manifest.rpc_methods.is_empty();
            let mut ui_slot_names: Vec<String> = Vec::new();
            let mut seen_slot: HashSet<String> = HashSet::new();
            let mut ui_slot_variants: Vec<UiSlotVariantDto> = Vec::new();
            for u in &manifest.ui_slots {
                if !bootstrap_dto::EMBEDDED_UI_SLOT_NAMES.contains(&u.slot.as_str()) {
                    continue;
                }
                ui_slot_variants.push(UiSlotVariantDto {
                    slot: u.slot.clone(),
                    appearance_id: normalize_ui_slot_appearance_id(&u.appearance_id),
                    label: u.label.clone(),
                });
                if seen_slot.insert(u.slot.clone()) {
                    ui_slot_names.push(u.slot.clone());
                }
            }
            let (dependency_status, dependency_issues) =
                dependency_report(&manifest, &version_by_id);
            Some(DirectoryPluginCatalogEntry {
                id: pid.clone(),
                version: manifest.version.clone(),
                plugin_type: manifest.plugin_type.clone(),
                has_ui_settings,
                has_rpc_process,
                declares_rpc_methods,
                is_shell,
                ui_slot_names,
                ui_slot_variants,
                provides: manifest.provides.clone(),
                description: manifest.description.clone(),
                author: manifest.author.clone(),
                permissions: manifest.permissions.clone(),
                dependency_status,
                dependency_issues,
            })
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Same logic as [`get_directory_plugin_catalog`]; for integration tests without `State` wrapper.
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn get_directory_plugin_catalog_impl(
    state: &AppState,
) -> Result<Vec<DirectoryPluginCatalogEntry>, CommandError> {
    let fp = plugin_catalog_fingerprint(state).map_err(|e| ApiError::Io {
        message: e.to_string(),
    })?;
    {
        let lock = PLUGIN_CATALOG_CACHE.lock();
        if let Some(cached) = lock.as_ref() {
            if cached.fingerprint == fp && cached.stored_at.elapsed() < Duration::from_secs(5) {
                return Ok(cached.entries.clone());
            }
        }
    }
    let out = build_directory_plugin_catalog(state);
    *PLUGIN_CATALOG_CACHE.lock() = Some(PluginCatalogCacheValue {
        fingerprint: fp,
        stored_at: Instant::now(),
        entries: out.clone(),
    });
    Ok(out)
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_directory_plugin_catalog(
    state: State<'_, SharedAppState>,
) -> Result<Vec<DirectoryPluginCatalogEntry>, CommandError> {
    get_directory_plugin_catalog_impl(&state)
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
    /// Per-role state saved alone in `plugin_state.json` (not merged with global default).
    pub role: RolePluginStateDto,
    /// Global default (plugin manager "global default"); merged with `role` to drive actual embeds and full-shell.
    pub global_defaults: RolePluginStateDto,
}

/// Same logic as [`get_plugin_state`]; for integration tests without `State` wrapper.
///
/// # Errors
///
/// Returns `Err(String)` when directory plugin runtime state cannot be read.
pub fn get_plugin_state_impl(
    role_id: &str,
    state: &AppState,
) -> Result<PluginStateGetResponse, CommandError> {
    let rt = &state.directory_plugins;
    let rid = role_id.trim();
    Ok(PluginStateGetResponse {
        role: rt.role_plugin_state_stored_for(rid).into(),
        global_defaults: rt.global_plugin_state().into(),
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_plugin_state(
    role_id: String,
    state: State<'_, SharedAppState>,
) -> Result<PluginStateGetResponse, CommandError> {
    get_plugin_state_impl(&role_id, &state)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn save_plugin_state(
    role_id: String,
    state: RolePluginStateDto,
    app: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    app.directory_plugins
        .save_role_plugin_state(role_id.trim(), state.into())
        .map_err(Into::into)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn save_global_plugin_state(
    state: RolePluginStateDto,
    app: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    app.directory_plugins
        .save_global_plugin_state(state.into())
        .map_err(Into::into)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn reset_plugin_state_to_role_default(
    role_id: String,
    app: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    let role = app.storage.load_role(role_id.trim())?;
    let ui = role.plugin_state_ui_baseline();
    app.directory_plugins
        .reset_role_plugin_state_from_ui(role_id.trim(), ui)
        .map_err(Into::into)
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
