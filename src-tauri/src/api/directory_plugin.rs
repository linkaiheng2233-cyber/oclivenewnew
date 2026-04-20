//! 目录式插件：启动引导与 JSON-RPC 透传（B2）。

use crate::api::error::ApiError;
use crate::error::AppError;
use crate::infrastructure::directory_plugins::{
    dependency_report, normalize_plugin_rel, normalize_ui_slot_appearance_id, parse_manifest_version,
    plugin_scan_container_roots, HostPluginsFile, OclivePluginManifest, UiSlotDecl,
};
use crate::infrastructure::plugin_state::{PluginStateFile, RolePluginState};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};
use crate::state::AppState;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUiSlotDto {
    pub plugin_id: String,
    /// 官方语义插槽名（见 `EMBEDDED_UI_SLOT_NAMES`）。
    pub slot: String,
    /// 与 manifest `ui_slots[].appearance_id` 一致；空字符串表示默认变体。
    pub appearance_id: String,
    /// 展示用标签（来自 manifest，可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 相对插件根，与 manifest `ui_slots[].entry` 一致（iframe 与 `plugin_bridge` 校验）。
    pub entry: String,
    /// 可选：相对插件根的 `.vue` 路径（`manifest.vueComponent`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vue_component: Option<String>,
    pub url: String,
}

/// 非整壳插件可声明的嵌入 UI 插槽（与前端消费一致）。
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
    /// 整壳 `manifest.shell.vueEntry`（相对插件根）；与 `force_iframe_mode` 共同决定是否走宿主 Vue 入口。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_vue_entry: Option<String>,
    /// 来自 `plugin_state`：为真时整壳与插槽均仅用 iframe，忽略 Vue 入口。
    pub force_iframe_mode: bool,
    pub plugin_ids: Vec<String>,
    pub developer_mode: bool,
    /// 当前角色下、已启用插件在 manifest `bridge.events` 中声明的宿主事件名（去重排序）。
    pub subscribed_host_events: Vec<String>,
    /// 非整壳插件在 `manifest.ui_slots` 中声明的嵌入 UI（主界面消费）。
    pub ui_slots: Vec<PluginUiSlotDto>,
}

/// 将 manifest 内 `shell.bridge` / `ui_slots[].bridge` 的 `events` 并入集合（与 `is_host_event_subscribed` 语义一致）。
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

/// 收集「未全局禁用」的插件在 `shell.bridge` / `ui_slots[].bridge` 中声明的 `events`。
fn collect_subscribed_host_events(state: &AppState, pst: &PluginStateFile) -> Vec<String> {
    let mut set = HashSet::new();
    let roots = state.directory_plugins.plugin_roots.read();
    for (pid, root) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = OclivePluginManifest::load_from_dir(root) else {
            continue;
        };
        merge_manifest_bridge_events(&manifest, &mut set);
    }
    subscribed_events_sorted_vec(set)
}

/// 对**同一插槽**的条目按 `plugin_state.slot_order[slot]` 排序。
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

/// 供 `get_directory_plugin_bootstrap` 与 `plugin_bridge_invoke` 共用。
/// `role_id`：当前角色；省略时尝试 `oclive_last_role_id.txt`，再回退旧版全局插件状态。
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
    let shell_url = shell_plugin_id.as_ref().and_then(|pid| {
        let roots = rt.plugin_roots.read();
        let root = roots.get(pid)?;
        let manifest = OclivePluginManifest::load_from_dir(root).ok()?;
        let entry = manifest.shell.as_ref()?.entry.as_str();
        rt.shell_url_for(pid, entry)
    });
    let shell_vue_entry = shell_plugin_id.as_ref().and_then(|pid| {
        let roots = rt.plugin_roots.read();
        let root = roots.get(pid)?;
        let manifest = OclivePluginManifest::load_from_dir(root).ok()?;
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
    for (pid, root) in roots.iter() {
        if pst.is_plugin_disabled(pid) {
            continue;
        }
        let Ok(manifest) = OclivePluginManifest::load_from_dir(root) else {
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
            by_slot
                .entry(decl.slot.clone())
                .or_default()
                .push(decl);
        }
        for (slot_name, decls) in by_slot {
            if pst.is_slot_contribution_disabled(&slot_name, pid) {
                continue;
            }
            let sel = appearance_for
                .and_then(|m| m.get(&slot_name))
                .map(|s| s.as_str());
            let decl_refs: Vec<&UiSlotDecl> = decls.iter().copied().collect();
            let Some(picked) = pick_ui_slot_decl(&decl_refs, sel) else {
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
    let rel = normalize_plugin_rel(rel.trim());
    if rel.is_empty() {
        return Err(ApiError::InvalidParameter {
            message: "rel required".into(),
        }
        .to_string());
    }
    if rel.split('/').any(|p| p == "..") {
        return Err(ApiError::InvalidParameter {
            message: "invalid rel path".into(),
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
    let path = root.join(&rel);
    let root_canon = root.canonicalize().map_err(|e| {
        ApiError::Io {
            message: format!("plugin root: {}", e),
        }
        .to_string()
    })?;
    let path_canon = path.canonicalize().map_err(|e| {
        ApiError::Io {
            message: format!("read_plugin_asset_text: {}", e),
        }
        .to_string()
    })?;
    if !path_canon.starts_with(&root_canon) {
        return Err(ApiError::PermissionDenied {
            message: "path escapes plugin directory".into(),
        }
        .to_string());
    }
    std::fs::read_to_string(&path_canon).map_err(|e| {
        ApiError::Io {
            message: e.to_string(),
        }
        .to_string()
    })
}

/// 查询某宿主内置事件名是否被当前角色下已启用插件订阅（与 `subscribed_host_events` 一致）。
#[tauri::command]
pub fn is_host_event_subscribed(
    event: String,
    role_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
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

#[tauri::command]
pub fn directory_plugin_invoke(
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
    let url = state
        .directory_plugins
        .ensure_rpc_url(pid)
        .map_err(|e| crate::api::error::map_directory_rpc_url_error(pid, e))?;
    invoke_directory_plugin_rpc_blocking(
        &url,
        req.method.trim(),
        req.params,
        RemoteRpcChannel::Plugin,
    )
    .map_err(|e: AppError| e.to_frontend_error())
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
    /// manifest 是否声明 `process`（可在此面板「启动」JSON-RPC 子进程）。
    pub has_rpc_process: bool,
    /// manifest 是否声明了 `rpcMethods`（便于调试面板预填方法；无 `process` 时仍可手填 RPC 测已运行实例）。
    pub declares_rpc_methods: bool,
    pub is_shell: bool,
    /// 声明的 UI 插槽名（如 `chat_toolbar`）；同一槽多外观时仍只出现一次槽名。
    pub ui_slot_names: Vec<String>,
    /// 每条 manifest `ui_slots`（嵌入槽）对应一条，含 `appearance_id` / `label`。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_slot_variants: Vec<UiSlotVariantDto>,
    pub provides: Vec<String>,
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
    for (pid, root) in roots.iter() {
        if let Ok(m) = OclivePluginManifest::load_from_dir(root) {
            if let Some(v) = parse_manifest_version(&m.version) {
                version_by_id.insert(pid.clone(), v);
            }
        }
    }
    let mut out: Vec<DirectoryPluginCatalogEntry> = roots
        .iter()
        .filter_map(|(pid, root)| {
            let manifest = OclivePluginManifest::load_from_dir(root).ok()?;
            let is_shell = manifest.shell.is_some();
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
                has_rpc_process,
                declares_rpc_methods,
                is_shell,
                ui_slot_names,
                ui_slot_variants,
                provides: manifest.provides.clone(),
                dependency_status,
                dependency_issues,
            })
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
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
    let out = build_directory_plugin_catalog(&state);
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
    /// 当前角色在 `plugin_state.json` 中单独保存的状态（未与全局默认合并）。
    pub role: RolePluginStateDto,
    /// 全局默认（插件管理「全局默认」）；与 `role` 合并后驱动实际嵌入与整壳。
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
    let role = app
        .storage
        .load_role(role_id.trim())
        .map_err(|e| e.to_string())?;
    let ui = role.plugin_state_ui_baseline();
    app.directory_plugins
        .reset_role_plugin_state_from_ui(role_id.trim(), ui)
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
