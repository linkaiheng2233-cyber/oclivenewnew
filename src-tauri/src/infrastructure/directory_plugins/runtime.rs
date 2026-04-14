//! 扫描根目录、解析 manifest、懒启动子进程并缓存 RPC URL。

use super::manifest::OclivePluginManifest;
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
        Arc::new(Self {
            plugin_roots: Arc::new(RwLock::new(scan.roots)),
            rpc_urls: Mutex::new(HashMap::new()),
            children: Mutex::new(HashMap::new()),
            startup_locks: Mutex::new(HashMap::new()),
            host,
        })
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
