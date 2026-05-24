//! 扫描根目录、解析 manifest、懒启动子进程并缓存 RPC URL。

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use dashmap::DashMap;

use super::manifest::{normalize_plugin_rel, normalize_ui_slot_appearance_id, OclivePluginManifest};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::plugin_state::{PluginStateFile, PluginStateStore, RolePluginState};
use crate::models::ui_config::UiConfig;

/// 目录插件根路径与其 canonical 形式（同一次 rescan 写入，单次锁读取）。
#[derive(Debug, Clone)]
pub struct PluginRootEntry {
    pub root: PathBuf,
    pub canonical: PathBuf,
}

impl PluginRootEntry {
    fn from_root(root: PathBuf) -> Self {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        Self { root, canonical }
    }
}

fn manifest_json_mtime(root: &Path) -> u64 {
    let p = root.join("manifest.json");
    std::fs::metadata(&p)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Resolve a normalized relative asset path under a plugin root without `canonicalize`.
///
/// Uses the cached [`PluginRootEntry::canonical`] from rescan; callers avoid per-request syscalls.
pub fn resolve_plugin_asset_path(entry: &PluginRootEntry, rel: &str) -> Result<PathBuf, String> {
    let rel = normalize_plugin_rel(rel);
    if rel.is_empty() {
        return Err("empty rel".into());
    }
    if rel.split('/').any(|p| p == ".." || p == ".") {
        return Err("invalid rel path".into());
    }
    let mut resolved = entry.canonical.clone();
    for segment in rel.split('/').filter(|s| !s.is_empty()) {
        resolved = resolved.join(segment);
    }
    if !resolved.starts_with(&entry.canonical) {
        return Err("path escapes plugin directory".into());
    }
    if !resolved.exists() {
        return Err("not found".into());
    }
    Ok(resolved)
}

fn plugin_roots_from_scan(roots: HashMap<String, PathBuf>) -> HashMap<String, PluginRootEntry> {
    roots
        .into_iter()
        .map(|(id, root)| (id, PluginRootEntry::from_root(root)))
        .collect()
}

/// `%APPDATA%/…/oclive_host_plugins.json`
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HostPluginsFile {
    #[serde(default)]
    pub developer_mode: bool,
    #[serde(default)]
    pub extra_plugin_roots: Vec<String>,
    #[serde(default)]
    pub shell_plugin_id: Option<String>,
}

impl HostPluginsFile {
    #[must_use]
    pub fn load(app_data: &Path) -> Self {
        let p = app_data.join("oclive_host_plugins.json");
        if let Ok(s) = std::fs::read_to_string(&p) {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    #[must_use]
    pub fn developer_effective(&self) -> bool {
        self.developer_mode || env_developer()
    }
}

fn env_developer() -> bool {
    std::env::var("OCLIVE_DEVELOPER")
        .ok()
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Default)]
pub struct PluginScanSummary {
    pub plugin_ids: Vec<String>,
    pub roots: HashMap<String, PathBuf>,
}

fn collect_plugin_dirs(root: &Path, out: &mut HashMap<String, PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        let mf = p.join("manifest.json");
        if !mf.is_file() {
            continue;
        }
        match OclivePluginManifest::load_from_dir(&p) {
            Ok(m) => {
                let id = m.id.trim().to_string();
                if let Some(prev) = out.insert(id.clone(), p.clone()) {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "directory plugin id={} duplicate; replacing path={} with {}",
                        id,
                        prev.display(),
                        p.display()
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    "skipping plugin directory (manifest invalid or unreadable): {} — {}",
                    p.display(),
                    e
                );
            }
        }
    }
}

/// 插件包所在容器目录（`plugins/` 等），用于扫描与（开发者模式）文件监听。
#[must_use]
pub fn plugin_scan_container_roots(
    roles_dir: &Path,
    app_data: &Path,
    host: &HostPluginsFile,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = roles_dir.parent() {
        let p = parent.join("plugins");
        if p.is_dir() {
            roots.push(p);
        }
    }
    let cwd = PathBuf::from("plugins");
    if cwd.is_dir() {
        roots.push(cwd);
    }
    let ad = app_data.join("plugins");
    if ad.is_dir() {
        roots.push(ad);
    }
    if host.developer_effective() {
        for s in &host.extra_plugin_roots {
            let p = PathBuf::from(s.trim());
            if p.is_dir() {
                roots.push(p);
            }
        }
    }
    roots
}

fn default_scan_roots(roles_dir: &Path, app_data: &Path, host: &HostPluginsFile) -> Vec<PathBuf> {
    plugin_scan_container_roots(roles_dir, app_data, host)
}

pub fn scan_plugins(
    roles_dir: &Path,
    app_data: &Path,
    host: &HostPluginsFile,
) -> PluginScanSummary {
    let mut roots = HashMap::new();
    for r in default_scan_roots(roles_dir, app_data, host) {
        collect_plugin_dirs(&r, &mut roots);
    }
    let mut plugin_ids: Vec<String> = roots.keys().cloned().collect();
    plugin_ids.sort();
    PluginScanSummary { plugin_ids, roots }
}

pub(crate) const DEBUG_LOG_RING_CAP: usize = 1000;

#[derive(Debug, Default)]
pub(crate) struct DebugLogRing {
    lines: VecDeque<String>,
}

impl DebugLogRing {
    fn push_line(&mut self, line: String) {
        while self.lines.len() >= DEBUG_LOG_RING_CAP {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    fn tail(&self, n: usize) -> Vec<String> {
        let n = n.min(self.lines.len());
        self.lines.iter().rev().take(n).rev().cloned().collect()
    }

    fn clear(&mut self) {
        self.lines.clear();
    }
}

/// 宿主为目录插件拉起的 RPC 子进程快照（开发者调试面板）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginProcessDebugInfo {
    pub plugin_id: String,
    pub pid: u32,
    pub rpc_url: String,
    pub started_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_kb: Option<u64>,
}

pub(crate) fn parse_ready_line(line: &str, prefix: &str) -> Option<String> {
    let t = line.trim();
    let rest = t.strip_prefix(prefix)?.trim();
    if rest.starts_with("http://") || rest.starts_with("https://") {
        Some(rest.to_string())
    } else {
        None
    }
}

/// 目录插件运行时：根路径表 + 懒启动 RPC。
pub struct DirectoryPluginRuntime {
    pub plugin_roots: Arc<RwLock<HashMap<String, PluginRootEntry>>>,
    rpc_urls: Mutex<HashMap<String, String>>,
    children: Mutex<HashMap<String, std::process::Child>>,
    /// 同一 `plugin_id` 仅允许一处执行 spawn + 握手，避免并发 `ensure_rpc_url` 拉起重复子进程。
    startup_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    /// 子进程 stdout/stderr 环形缓冲（开发者调试）。
    debug_log_rings: Mutex<HashMap<String, Arc<Mutex<DebugLogRing>>>>,
    /// 子进程启动时刻（Unix 毫秒）。
    process_started_ms: Mutex<HashMap<String, u64>>,
    host: HostPluginsFile,
    app_data_dir: PathBuf,
    /// `app_data_dir/plugin_state.json`（v2：按 `role_id` 隔离）
    plugin_state_store: Arc<RwLock<PluginStateStore>>,
    /// 当前主界面所加载的角色（用于插槽/禁用解析；资产网关与 RPC 共用）。
    active_role_id: Arc<RwLock<Option<String>>>,
    /// 与 `get_directory_plugin_catalog` 短时缓存联动；`rescan_plugin_roots` 时递增使缓存失效。
    catalog_invalidate_gen: AtomicU64,
    high_risk_grants: Arc<HighRiskGrantStore>,
    /// `plugin_id` → (`manifest.json` mtime ms, parsed manifest).
    manifest_cache: DashMap<String, (u64, Arc<OclivePluginManifest>)>,
}

impl DirectoryPluginRuntime {
    pub fn bootstrap(
        roles_dir: &Path,
        app_data: &Path,
        high_risk_grants: Arc<HighRiskGrantStore>,
    ) -> Arc<Self> {
        Self::bootstrap_inner(roles_dir, app_data, high_risk_grants, true)
    }

    /// 跳过目录扫描，供启动后后台 [`Self::rescan_plugin_roots`] 懒加载。
    pub fn bootstrap_deferred_scan(
        roles_dir: &Path,
        app_data: &Path,
        high_risk_grants: Arc<HighRiskGrantStore>,
    ) -> Arc<Self> {
        Self::bootstrap_inner(roles_dir, app_data, high_risk_grants, false)
    }

    fn bootstrap_inner(
        roles_dir: &Path,
        app_data: &Path,
        high_risk_grants: Arc<HighRiskGrantStore>,
        scan_now: bool,
    ) -> Arc<Self> {
        let host = HostPluginsFile::load(app_data);
        let roots = if scan_now {
            let scan = scan_plugins(roles_dir, app_data, &host);
            tracing::info!(
                target: "oclive_plugin",
                "directory plugins scanned count={} ids={:?}",
                scan.roots.len(),
                scan.plugin_ids
            );
            scan.roots
        } else {
            tracing::info!(
                target: "oclive_plugin",
                "directory plugin scan deferred to background"
            );
            HashMap::new()
        };
        let app_data_dir = app_data.to_path_buf();
        let ps_path = app_data_dir.join("plugin_state.json");
        let plugin_state_store = Arc::new(RwLock::new(PluginStateStore::load(&ps_path)));
        let root_entries = plugin_roots_from_scan(roots);
        Arc::new(Self {
            plugin_roots: Arc::new(RwLock::new(root_entries)),
            rpc_urls: Mutex::new(HashMap::new()),
            children: Mutex::new(HashMap::new()),
            startup_locks: Mutex::new(HashMap::new()),
            debug_log_rings: Mutex::new(HashMap::new()),
            process_started_ms: Mutex::new(HashMap::new()),
            host,
            app_data_dir,
            plugin_state_store,
            active_role_id: Arc::new(RwLock::new(None)),
            catalog_invalidate_gen: AtomicU64::new(0),
            high_risk_grants,
            manifest_cache: DashMap::new(),
        })
    }

    /// Load manifest from disk at most once per `manifest.json` mtime within a scan cycle.
    pub fn load_manifest_cached(
        &self,
        plugin_id: &str,
        root: &Path,
    ) -> Result<Arc<OclivePluginManifest>, String> {
        let mtime = manifest_json_mtime(root);
        if let Some(entry) = self.manifest_cache.get(plugin_id) {
            if entry.0 == mtime {
                return Ok(Arc::clone(&entry.1));
            }
        }
        let manifest = OclivePluginManifest::load_from_dir(root)?;
        let arc = Arc::new(manifest);
        self.manifest_cache
            .insert(plugin_id.to_string(), (mtime, Arc::clone(&arc)));
        Ok(arc)
    }

    #[must_use]
    pub fn catalog_cache_invalidation_gen(&self) -> u64 {
        self.catalog_invalidate_gen.load(Ordering::Relaxed)
    }

    fn plugin_state_path(&self) -> PathBuf {
        self.app_data_dir.join("plugin_state.json")
    }

    fn persist_plugin_state_store(&self) -> Result<(), String> {
        let store = self.plugin_state_store.read().clone();
        store.save(&self.plugin_state_path())
    }

    /// 无活跃角色时回退旧版全局状态（迁移），否则空默认。
    #[must_use]
    pub fn effective_slots(&self) -> PluginStateFile {
        let store = self.plugin_state_store.read();
        if let Some(id) = self.active_role_id.read().as_ref() {
            let raw = store.roles.get(id).cloned().unwrap_or_default();
            let merged = RolePluginState::merge_global_defaults(store.global.as_ref(), &raw);
            return merged.slots;
        }
        if let Some(leg) = &store.legacy_v1 {
            return leg.clone();
        }
        if let Some(g) = &store.global {
            return g.slots.clone();
        }
        PluginStateFile::default()
    }

    #[must_use]
    pub fn plugin_state_snapshot(&self) -> PluginStateFile {
        self.effective_slots()
    }

    /// 磁盘上按 `role_id` 存储的原始状态（不含 `global` 合并）。
    #[must_use]
    pub fn role_plugin_state_stored_for(&self, role_id: &str) -> RolePluginState {
        let store = self.plugin_state_store.read();
        store.roles.get(role_id).cloned().unwrap_or_default()
    }

    /// 将 `global` 默认与按角色存储合并后的有效状态（整壳 / 插槽 / 禁用等）。
    #[must_use]
    pub fn role_plugin_state_for(&self, role_id: &str) -> RolePluginState {
        let store = self.plugin_state_store.read();
        let raw = store.roles.get(role_id).cloned().unwrap_or_default();
        RolePluginState::merge_global_defaults(store.global.as_ref(), &raw)
    }

    #[must_use]
    pub fn global_plugin_state(&self) -> RolePluginState {
        self.plugin_state_store
            .read()
            .global
            .clone()
            .unwrap_or_default()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn save_global_plugin_state(&self, mut state: RolePluginState) -> Result<(), String> {
        self.sanitize_role_shell(&mut state);
        let mut store = self.plugin_state_store.write();
        store.global = Some(state);
        store.schema_version = 3;
        drop(store);
        self.persist_plugin_state_store()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn save_role_plugin_state(
        &self,
        role_id: &str,
        mut state: RolePluginState,
    ) -> Result<(), String> {
        self.sanitize_role_shell(&mut state);
        let mut store = self.plugin_state_store.write();
        store.schema_version = 3;
        store.roles.insert(role_id.trim().to_string(), state);
        drop(store);
        self.persist_plugin_state_store()
    }

    /// 角色加载后：若本地尚无该角色的用户记录，则用 `ui.json`（或 legacy 全局）初始化。
    pub fn ensure_role_plugin_state(&self, role_id: &str, ui: &UiConfig) {
        {
            let store = self.plugin_state_store.read();
            if store.roles.contains_key(role_id) {
                return;
            }
        }
        let mut new_state = {
            let mut store = self.plugin_state_store.write();
            if store.roles.contains_key(role_id) {
                return;
            }
            if ui.is_effectively_empty() {
                if let Some(leg) = store.legacy_v1.take() {
                    RolePluginState {
                        shell_plugin_id: String::new(),
                        slots: leg,
                    }
                } else {
                    RolePluginState::default()
                }
            } else {
                RolePluginState::from_ui_config(ui)
            }
        };
        self.sanitize_role_shell(&mut new_state);
        let mut store = self.plugin_state_store.write();
        if store.roles.contains_key(role_id) {
            return;
        }
        store.schema_version = 3;
        store.roles.insert(role_id.to_string(), new_state);
        drop(store);
        let _ = self.persist_plugin_state_store();
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// 用磁盘上的 `ui.json` 覆盖该角色的用户记录（「重置为角色包推荐」）。
    pub fn reset_role_plugin_state_from_ui(
        &self,
        role_id: &str,
        ui: &UiConfig,
    ) -> Result<(), String> {
        let mut new_state = RolePluginState::from_ui_config(ui);
        self.sanitize_role_shell(&mut new_state);
        let mut store = self.plugin_state_store.write();
        store.roles.insert(role_id.trim().to_string(), new_state);
        drop(store);
        self.persist_plugin_state_store()
    }

    pub fn set_active_role_id(&self, role_id: &str) {
        *self.active_role_id.write() = Some(role_id.trim().to_string());
        let p = self.app_data_dir.join("oclive_last_role_id.txt");
        let _ = std::fs::write(&p, role_id.trim());
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// 删除某角色的插件 UI 状态；若当前活跃角色相同则清空活跃标记。
    pub fn remove_role_plugin_state(&self, role_id: &str) -> Result<(), String> {
        let rid = role_id.trim();
        if rid.is_empty() {
            return Ok(());
        }
        {
            let mut store = self.plugin_state_store.write();
            store.roles.remove(rid);
        }
        let mut ar = self.active_role_id.write();
        if ar.as_deref() == Some(rid) {
            *ar = None;
        }
        drop(ar);
        self.persist_plugin_state_store()
    }

    #[must_use]
    pub fn read_last_role_id_from_disk(&self) -> Option<String> {
        let p = self.app_data_dir.join("oclive_last_role_id.txt");
        std::fs::read_to_string(&p)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub(crate) fn sanitize_role_shell(&self, state: &mut RolePluginState) {
        let sid = state.shell_plugin_id.trim();
        if sid.is_empty() {
            state.shell_plugin_id.clear();
        } else {
            let roots = self.plugin_roots.read();
            if let Some(entry) = roots.get(sid) {
                if let Ok(manifest) = self.load_manifest_cached(sid, &entry.root) {
                    let ok = manifest.plugin_type.as_deref() == Some("ocliveplugin")
                        && manifest.shell.is_some();
                    if !ok {
                        tracing::warn!(
                            target: "oclive_plugin",
                            "invalid shell plugin (require type=ocliveplugin + shell): {}",
                            sid
                        );
                        state.shell_plugin_id.clear();
                    }
                } else {
                    state.shell_plugin_id.clear();
                }
            } else {
                tracing::warn!(
                    target: "oclive_plugin",
                    "shell plugin id not in scan roots: {}",
                    sid
                );
                state.shell_plugin_id.clear();
            }
        }
        Self::sanitize_slot_appearance_maps(self, &self.plugin_roots.read(), &mut state.slots);
    }

    fn sanitize_slot_appearance_maps(
        rt: &DirectoryPluginRuntime,
        roots: &HashMap<String, PluginRootEntry>,
        slots: &mut PluginStateFile,
    ) {
        slots.slot_appearance.retain(|pid, by_slot| {
            let Some(entry) = roots.get(pid) else {
                return false;
            };
            let Ok(manifest) = rt.load_manifest_cached(pid, &entry.root) else {
                return false;
            };
            by_slot.retain(|slot, aid| {
                let want = normalize_ui_slot_appearance_id(aid);
                manifest.ui_slots.iter().any(|d| {
                    d.slot == *slot && normalize_ui_slot_appearance_id(&d.appearance_id) == want
                })
            });
            !by_slot.is_empty()
        });
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn reload_plugin_state(&self) -> Result<(), String> {
        let p = self.plugin_state_path();
        let next = PluginStateStore::load(&p);
        *self.plugin_state_store.write() = next;
        Ok(())
    }

    #[must_use]
    pub fn host(&self) -> &HostPluginsFile {
        &self.host
    }

    #[must_use]
    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    /// 终止子进程并丢弃 RPC 缓存（插件目录被替换后应调用，再 `rescan_plugin_roots`）。
    pub fn clear_plugin_process(&self, plugin_id: &str) {
        let id = plugin_id.trim();
        if id.is_empty() {
            return;
        }
        if let Some(mut child) = self.children.lock().remove(id) {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.rpc_urls.lock().remove(id);
        self.startup_locks.lock().remove(id);
        self.process_started_ms.lock().remove(id);
        self.debug_log_rings.lock().remove(id);
    }

    /// 重新扫描 `plugins/` 等根目录并替换内存中的 `plugin_roots`。
    pub fn rescan_plugin_roots(&self, roles_dir: &Path) {
        for id in self.rpc_urls.lock().keys().cloned().collect::<Vec<_>>() {
            self.clear_plugin_process(&id);
        }
        self.catalog_invalidate_gen.fetch_add(1, Ordering::Relaxed);
        self.manifest_cache.clear();
        let scan = scan_plugins(roles_dir, &self.app_data_dir, &self.host);
        let n = scan.roots.len();
        *self.plugin_roots.write() = plugin_roots_from_scan(scan.roots);
        tracing::info!(
            target: "oclive_plugin",
            "plugin roots rescanned count={}",
            n
        );
    }

    /// 目录插件 manifest `provides` 是否声明某能力（如 `complex_emotion`）。
    #[must_use]
    pub fn manifest_provides_capability(&self, plugin_id: &str, capability: &str) -> bool {
        let id = plugin_id.trim();
        let cap = capability.trim();
        if id.is_empty() || cap.is_empty() {
            return false;
        }
        let root = match self.plugin_roots.read().get(id) {
            Some(entry) => entry.root.clone(),
            None => return false,
        };
        self.load_manifest_cached(id, &root)
            .ok()
            .is_some_and(|m| m.provides.iter().any(|p| p.trim() == cap))
    }

    pub fn shell_url_for(&self, plugin_id: &str, entry: &str) -> Option<String> {
        let roots = self.plugin_roots.read();
        if !roots.contains_key(plugin_id) {
            return None;
        }
        let entry = entry.trim_start_matches('/');
        if entry.is_empty() {
            return None;
        }
        // Windows WebView2 使用 https://scheme.localhost/…
        Some(format!(
            "https://ocliveplugin.localhost/{}/{}",
            plugin_id, entry
        ))
    }
}

#[cfg(test)]
mod asset_path_tests {
    use super::{PluginRootEntry, resolve_plugin_asset_path};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_plugin_asset_path_rejects_parent_traversal() {
        let tmp = TempDir::new().expect("temp");
        let root = tmp.path().join("plugin");
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join("manifest.json"), r#"{"schema_version":1,"id":"p","version":"1.0.0"}"#)
            .expect("write manifest");
        let entry = PluginRootEntry::from_root(root.clone());
        let err = resolve_plugin_asset_path(&entry, "../secret.txt").expect_err("traversal");
        assert!(err.contains("invalid") || err.contains("escapes"));
    }

    #[test]
    fn resolve_plugin_asset_path_serves_file_under_root() {
        let tmp = TempDir::new().expect("temp");
        let root = tmp.path().join("plugin");
        fs::create_dir_all(&root).expect("mkdir");
        fs::write(root.join("hello.txt"), "hi").expect("write file");
        let entry = PluginRootEntry::from_root(root);
        let path = resolve_plugin_asset_path(&entry, "hello.txt").expect("resolve");
        assert!(path.is_file());
    }
}

mod rpc;
mod spawn;
