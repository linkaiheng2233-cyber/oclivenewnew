//! 扫描根目录、解析 manifest、懒启动子进程并缓存 RPC URL。

use super::manifest::OclivePluginManifest;
use crate::infrastructure::plugin_state::{PluginStateFile, PluginStateStore, RolePluginState};
use crate::models::ui_config::UiConfig;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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
    pub fn load(app_data: &Path) -> Self {
        let p = app_data.join("oclive_host_plugins.json");
        if let Ok(s) = std::fs::read_to_string(&p) {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Self::default()
        }
    }

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
        if let Ok(m) = OclivePluginManifest::load_from_dir(&p) {
            let id = m.id.trim().to_string();
            if let Some(prev) = out.insert(id.clone(), p.clone()) {
                log::warn!(
                    target: "oclive_plugin",
                    "directory plugin id={} duplicate; replacing path={} with {}",
                    id,
                    prev.display(),
                    p.display()
                );
            }
        }
    }
}

fn default_scan_roots(roles_dir: &Path, app_data: &Path, host: &HostPluginsFile) -> Vec<PathBuf> {
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

pub fn scan_plugins(roles_dir: &Path, app_data: &Path, host: &HostPluginsFile) -> PluginScanSummary {
    let mut roots = HashMap::new();
    for r in default_scan_roots(roles_dir, app_data, host) {
        collect_plugin_dirs(&r, &mut roots);
    }
    let mut plugin_ids: Vec<String> = roots.keys().cloned().collect();
    plugin_ids.sort();
    PluginScanSummary {
        plugin_ids,
        roots,
    }
}

fn parse_ready_line(line: &str, prefix: &str) -> Option<String> {
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
    pub plugin_roots: Arc<RwLock<HashMap<String, PathBuf>>>,
    rpc_urls: Mutex<HashMap<String, String>>,
    children: Mutex<HashMap<String, std::process::Child>>,
    /// 同一 `plugin_id` 仅允许一处执行 spawn + 握手，避免并发 `ensure_rpc_url` 拉起重复子进程。
    startup_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    host: HostPluginsFile,
    app_data_dir: PathBuf,
    /// `app_data_dir/plugin_state.json`（v2：按 `role_id` 隔离）
    plugin_state_store: Arc<RwLock<PluginStateStore>>,
    /// 当前主界面所加载的角色（用于插槽/禁用解析；资产网关与 RPC 共用）。
    active_role_id: Arc<RwLock<Option<String>>>,
}

impl DirectoryPluginRuntime {
    pub fn bootstrap(roles_dir: &Path, app_data: &Path) -> Arc<Self> {
        let host = HostPluginsFile::load(app_data);
        let scan = scan_plugins(roles_dir, app_data, &host);
        log::info!(
            target: "oclive_plugin",
            "directory plugins scanned count={} ids={:?}",
            scan.roots.len(),
            scan.plugin_ids
        );
        let app_data_dir = app_data.to_path_buf();
        let ps_path = app_data_dir.join("plugin_state.json");
        let plugin_state_store = Arc::new(RwLock::new(PluginStateStore::load(&ps_path)));
        Arc::new(Self {
            plugin_roots: Arc::new(RwLock::new(scan.roots)),
            rpc_urls: Mutex::new(HashMap::new()),
            children: Mutex::new(HashMap::new()),
            startup_locks: Mutex::new(HashMap::new()),
            host,
            app_data_dir,
            plugin_state_store,
            active_role_id: Arc::new(RwLock::new(None)),
        })
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
            if let Some(rs) = store.roles.get(id) {
                return rs.slots.clone();
            }
        }
        if let Some(leg) = &store.legacy_v1 {
            return leg.clone();
        }
        PluginStateFile::default()
    }

    #[must_use]
    pub fn plugin_state_snapshot(&self) -> PluginStateFile {
        self.effective_slots()
    }

    #[must_use]
    pub fn role_plugin_state_for(&self, role_id: &str) -> RolePluginState {
        let store = self.plugin_state_store.read();
        store
            .roles
            .get(role_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn save_role_plugin_state(&self, role_id: &str, mut state: RolePluginState) -> Result<(), String> {
        self.sanitize_role_shell(&mut state);
        let mut store = self.plugin_state_store.write();
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
        store.roles.insert(role_id.to_string(), new_state);
        drop(store);
        let _ = self.persist_plugin_state_store();
    }

    /// 用磁盘上的 `ui.json` 覆盖该角色的用户记录（「重置为角色包推荐」）。
    pub fn reset_role_plugin_state_from_ui(&self, role_id: &str, ui: &UiConfig) -> Result<(), String> {
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

    #[must_use]
    pub fn read_last_role_id_from_disk(&self) -> Option<String> {
        let p = self.app_data_dir.join("oclive_last_role_id.txt");
        std::fs::read_to_string(&p)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn sanitize_role_shell(&self, state: &mut RolePluginState) {
        let sid = state.shell_plugin_id.trim();
        if sid.is_empty() {
            state.shell_plugin_id.clear();
            return;
        }
        let roots = self.plugin_roots.read();
        let Some(root) = roots.get(sid) else {
            log::warn!(
                target: "oclive_plugin",
                "shell plugin id not in scan roots: {}",
                sid
            );
            state.shell_plugin_id.clear();
            return;
        };
        let Ok(manifest) = OclivePluginManifest::load_from_dir(root) else {
            state.shell_plugin_id.clear();
            return;
        };
        let ok = manifest.plugin_type.as_deref() == Some("ocliveplugin") && manifest.shell.is_some();
        if !ok {
            log::warn!(
                target: "oclive_plugin",
                "invalid shell plugin (require type=ocliveplugin + shell): {}",
                sid
            );
            state.shell_plugin_id.clear();
        }
    }

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

    /// 懒启动：返回 JSON-RPC 根 URL（含 path，如 `http://127.0.0.1:9/rpc`）。
    pub fn ensure_rpc_url(&self, plugin_id: &str) -> Result<String, String> {
        let id = plugin_id.trim();
        if self.effective_slots().is_plugin_disabled(id) {
            return Err(format!("plugin disabled: {}", id));
        }
        if let Some(u) = self.rpc_urls.lock().get(plugin_id) {
            return Ok(u.clone());
        }
        let lock = {
            let mut map = self.startup_locks.lock();
            map.entry(plugin_id.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _startup = lock.lock();
        if let Some(u) = self.rpc_urls.lock().get(plugin_id) {
            return Ok(u.clone());
        }
        let root = self
            .plugin_roots
            .read()
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| format!("unknown directory plugin_id={}", plugin_id))?;
        let manifest = OclivePluginManifest::load_from_dir(&root)?;
        let proc = manifest
            .process
            .as_ref()
            .ok_or_else(|| format!("plugin {} has no process section", plugin_id))?;
        let prefix = if manifest.ready_prefix.trim().is_empty() {
            "OCLIVE_READY"
        } else {
            manifest.ready_prefix.trim()
        };
        let mut cmd = Command::new(&proc.command);
        for a in &proc.args {
            cmd.arg(a);
        }
        let cwd = proc
            .cwd
            .as_ref()
            .map(|c| root.join(c))
            .unwrap_or_else(|| root.clone());
        cmd.current_dir(&cwd);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("spawn plugin {}: {}", plugin_id, e))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format!("plugin {}: no stdout", plugin_id))?;
        let lines = Arc::new(Mutex::new(Vec::<String>::new()));
        let lines_clone = lines.clone();
        thread::spawn(move || {
            let r = BufReader::new(stdout);
            for line in r.lines().flatten() {
                lines_clone.lock().push(line);
            }
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        let url = 'wait: loop {
            if Instant::now() > deadline {
                let _ = child.kill();
                return Err(format!(
                    "plugin {}: timeout waiting for {} URL",
                    plugin_id, prefix
                ));
            }
            for line in lines.lock().iter() {
                if let Some(u) = parse_ready_line(line, prefix) {
                    break 'wait u;
                }
            }
            thread::sleep(Duration::from_millis(50));
        };
        self.children.lock().insert(plugin_id.to_string(), child);
        self.rpc_urls
            .lock()
            .insert(plugin_id.to_string(), url.clone());
        log::info!(
            target: "oclive_plugin",
            "directory plugin id={} rpc_url={}",
            plugin_id,
            url
        );
        Ok(url)
    }
}
