//! Directory plugins: bootstrap and JSON-RPC passthrough (B2).

use crate::api::error::ApiError;
use crate::infrastructure::directory_plugins::{
    dependency_report, normalize_plugin_rel, normalize_ui_slot_appearance_id,
    parse_manifest_version, plugin_scan_container_roots, resolve_plugin_asset_path,
    HostPluginsFile, OclivePluginManifest, UiSlotDecl,
};
use crate::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::state::{AppState, SharedAppState};
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
use crate::api::error::{map_directory_rpc_url_error, CommandError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUiSlotDto {
    pub plugin_id: String,
    /// Official semantic slot name (see `EMBEDDED_UI_SLOT_NAMES`).
    pub slot: String,
    /// Matches manifest `ui_slots[].appearance_id`; empty string means default variant.
    pub appearance_id: String,
    /// Display label (from manifest, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Relative to plugin root; matches manifest `ui_slots[].entry` (iframe and `plugin_bridge` validation).
    pub entry: String,
    /// Optional `.vue` path relative to plugin root (`manifest.vueComponent`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vue_component: Option<String>,
    pub url: String,
}

/// Embedded UI slots that non-shell plugins may declare (aligned with frontend consumption).
const EMBEDDED_UI_SLOT_NAMES: &[&str] = &[
    "chat_toolbar",
    "settings.panel",
    "role.detail",
    "sidebar",
    "chat.header",
    "settings.plugins",
    "settings.advanced",
    "overlay.floating",
    "launcher.palette",
    "debug.dock",
];

fn pick_ui_slot_decl<'a>(
    decls: &[&'a UiSlotDecl],
    selected: Option<&str>,
) -> Option<&'a UiSlotDecl> {
    if decls.is_empty() {
        return None;
    }
    let want = selected
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_ui_slot_appearance_id);
    if let Some(ref w) = want {
        for d in decls {
            if normalize_ui_slot_appearance_id(&d.appearance_id) == *w {
                return Some(*d);
            }
        }
    }
    for d in decls {
        if normalize_ui_slot_appearance_id(&d.appearance_id).is_empty() {
            return Some(*d);
        }
    }
    Some(decls[0])
}

fn plugin_ui_slot_dto_from_decl(pid: &str, decl: &UiSlotDecl) -> Option<PluginUiSlotDto> {
    let entry = decl.entry.trim().trim_start_matches(['/', '\\']);
    if entry.is_empty() {
        return None;
    }
    let entry_norm = entry.replace('\\', "/");
    let vue_component = decl
        .vue_component
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('\\', "/"));
    let url = format!("https://ocliveplugin.localhost/{}/{}", pid, entry_norm);
    Some(PluginUiSlotDto {
        plugin_id: pid.to_string(),
        slot: decl.slot.clone(),
        appearance_id: normalize_ui_slot_appearance_id(&decl.appearance_id),
        label: decl.label.clone(),
        entry: entry_norm,
        vue_component,
        url,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPluginBootstrapDto {
    pub shell_url: Option<String>,
    pub shell_plugin_id: Option<String>,
    /// Full-shell `manifest.shell.vueEntry` (relative to plugin root); with `force_iframe_mode`, decides host Vue vs iframe entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_vue_entry: Option<String>,
    /// From `plugin_state`: when true, full-shell and slots use iframe only; Vue entry is ignored.
    pub force_iframe_mode: bool,
    pub plugin_ids: Vec<String>,
    pub developer_mode: bool,
    /// Host event names declared in enabled plugins' manifest `bridge.events` for the current role (deduped, sorted).
    pub subscribed_host_events: Vec<String>,
    /// Embedded UI declared in non-shell plugins' `manifest.ui_slots` (consumed by main UI).
    pub ui_slots: Vec<PluginUiSlotDto>,
}

/// Merge `events` from manifest `shell.bridge` / `ui_slots[].bridge` into a set (same semantics as `is_host_event_subscribed`).
fn merge_manifest_bridge_events(manifest: &OclivePluginManifest, set: &mut HashSet<String>) {
    if let Some(sh) = &manifest.shell {
        if let Some(b) = &sh.bridge {
            for e in &b.events {
                let t = e.trim();
                if !t.is_empty() {
                    set.insert(t.to_string());
                }
            }
        }
    }
    for us in &manifest.ui_slots {
        if let Some(b) = &us.bridge {
            for e in &b.events {
                let t = e.trim();
                if !t.is_empty() {
                    set.insert(t.to_string());
                }
            }
        }
    }
}

fn subscribed_events_sorted_vec(set: HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort_unstable();
    v
}

/// Collect `events` from plugins not globally disabled in `shell.bridge` / `ui_slots[].bridge`.
fn collect_subscribed_host_events(state: &AppState, pst: &PluginStateFile) -> Vec<String> {
    let mut set = HashSet::new();
    let roots = state.directory_plugins.plugin_roots.read();
    for (pid, entry) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = state.directory_plugins.load_manifest_cached(pid, &entry.root) else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut set);
    }
    subscribed_events_sorted_vec(set)
}

/// Sort entries for the **same slot** by `plugin_state.slot_order[slot]`.
fn order_plugin_slots(mut slots: Vec<PluginUiSlotDto>, order: &[String]) -> Vec<PluginUiSlotDto> {
    let mut by_id: HashMap<String, PluginUiSlotDto> =
        slots.drain(..).map(|s| (s.plugin_id.clone(), s)).collect();
    let mut out = Vec::new();
    for id in order {
        if let Some(s) = by_id.remove(id.as_str()) {
            out.push(s);
        }
    }
    let mut rest: Vec<_> = by_id.into_values().collect();
    rest.sort_by(|a, b| a.plugin_id.cmp(&b.plugin_id));
    out.extend(rest);
    out
}

/// Shared by `get_directory_plugin_bootstrap` and `plugin_bridge_invoke`.
/// `role_id`: current role; when omitted, try `oclive_last_role_id.txt`, then fall back to legacy global plugin state.
pub fn directory_plugin_bootstrap_dto(
    state: &AppState,
    role_id: Option<String>,
) -> DirectoryPluginBootstrapDto {
    let rt = &state.directory_plugins;
    let host = rt.host();
    let rid = role_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| rt.read_last_role_id_from_disk());
    let role_state = if let Some(ref id) = rid {
        let mut s = rt.role_plugin_state_for(id);
        rt.sanitize_role_shell(&mut s);
        s
    } else {
        let mut s = RolePluginState {
            shell_plugin_id: String::new(),
            slots: rt.effective_slots(),
        };
        rt.sanitize_role_shell(&mut s);
        s
    };
    let pst = &role_state.slots;
    let mut plugin_ids_sorted: Vec<String> = rt.plugin_roots.read().keys().cloned().collect();
    plugin_ids_sorted.retain(|id| !pst.is_plugin_disabled(id));
    plugin_ids_sorted.sort_unstable();
    let shell_plugin_id_raw = shell_plugin_id_resolved(host, Some(&role_state));
    let shell_plugin_id = shell_plugin_id_raw.filter(|id| !pst.is_plugin_disabled(id));
    let shell_manifest = shell_plugin_id.as_ref().and_then(|pid| {
        let roots = rt.plugin_roots.read();
        let entry = roots.get(pid)?;
        rt.load_manifest_cached(pid, &entry.root).ok()
    });
    let shell_url = shell_manifest.as_ref().and_then(|manifest| {
        let pid = shell_plugin_id.as_ref()?;
        let sh = manifest.shell.as_ref()?;
        rt.shell_url_for(pid, sh.entry.as_str())
    });
    let shell_vue_entry = shell_manifest.as_ref().and_then(|manifest| {
        let sh = manifest.shell.as_ref()?;
        let ve = sh.vue_entry.as_ref()?.trim();
        if ve.is_empty() {
            None
        } else {
            Some(ve.replace('\\', "/"))
        }
    });

    let mut ui_slots = Vec::new();
    let mut subscribed_set = HashSet::new();
    let roots = rt.plugin_roots.read();
    for (pid, entry) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = rt.load_manifest_cached(pid, &entry.root) else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut subscribed_set);
        if manifest.shell.is_some() {
            continue;
        }
        let appearance_for = pst.slot_appearance.get(pid);
        let mut by_slot: HashMap<String, Vec<&UiSlotDecl>> = HashMap::new();
        for decl in &manifest.ui_slots {
            if !EMBEDDED_UI_SLOT_NAMES.contains(&decl.slot.as_str()) {
                continue;
            }
            by_slot.entry(decl.slot.clone()).or_default().push(decl);
        }
        for (slot_name, decls) in by_slot {
            if pst.is_slot_contribution_disabled(&slot_name, pid) {
                continue;
            }
            let sel = appearance_for
                .and_then(|m| m.get(&slot_name))
                .map(|s| s.as_str());
            let Some(picked) = pick_ui_slot_decl(&decls, sel) else {
                continue;
            };
            let Some(dto) = plugin_ui_slot_dto_from_decl(pid, picked) else {
                continue;
            };
            ui_slots.push(dto);
        }
    }
    let mut ui_slots_ordered = Vec::new();
    for slot_name in EMBEDDED_UI_SLOT_NAMES {
        let mut bucket: Vec<_> = ui_slots
            .iter()
            .filter(|s| s.slot == *slot_name)
            .cloned()
            .collect();
        let order = pst
            .slot_order
            .get(*slot_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        bucket = order_plugin_slots(bucket, order);
        ui_slots_ordered.extend(bucket);
    }
    let ui_slots = ui_slots_ordered;

    let subscribed_host_events = subscribed_events_sorted_vec(subscribed_set);

    DirectoryPluginBootstrapDto {
        shell_url,
        shell_plugin_id,
        shell_vue_entry,
        force_iframe_mode: pst.force_iframe_mode,
        plugin_ids: plugin_ids_sorted,
        developer_mode: host.developer_effective(),
        subscribed_host_events,
        ui_slots,
    }
}

fn shell_plugin_id_resolved(
    host: &HostPluginsFile,
    role: Option<&RolePluginState>,
) -> Option<String> {
    if let Ok(v) = std::env::var("OCLIVE_SHELL_PLUGIN_ID") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    if let Some(rs) = role {
        let t = rs.shell_plugin_id.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    host.shell_plugin_id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
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
    let entry = roots.get(pid).ok_or_else(|| {
        ApiError::PluginNotFound {
            plugin_id: pid.to_string(),
        }
    })?;
    let path_canon = resolve_plugin_asset_path(entry, &rel).map_err(|e| {
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
    Ok(std::fs::read_to_string(&path_canon).map_err(|e| {
        ApiError::Io {
            message: e.to_string(),
        }
    })?)
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
                if !EMBEDDED_UI_SLOT_NAMES.contains(&u.slot.as_str()) {
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
    let fp = plugin_catalog_fingerprint(state).map_err(|e| {
        ApiError::Io {
            message: e.to_string(),
        }
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
    let role = app
        .storage
        .load_role(role_id.trim())
        ?;
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
