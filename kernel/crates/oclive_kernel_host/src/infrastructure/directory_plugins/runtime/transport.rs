//! Directory plugin scan roots, asset path resolution, and RPC URL transport helpers.

use super::super::manifest::{normalize_plugin_rel, OclivePluginManifest};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Directory plugin root path and its canonical form (written on each rescan, read under one lock).
#[derive(Debug, Clone)]
pub struct PluginRootEntry {
    pub root: PathBuf,
    pub canonical: PathBuf,
}

impl PluginRootEntry {
    pub(crate) fn from_root(root: PathBuf) -> Self {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        Self { root, canonical }
    }
}

pub(crate) fn manifest_json_mtime(root: &Path) -> u64 {
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
/// # Errors
///
/// Returns [`Err`] when `rel` is empty, contains `..`, escapes the plugin root, or the path does not exist.
pub fn find_plugin_asset_path(entry: &PluginRootEntry, rel: &str) -> Result<PathBuf, String> {
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

pub(crate) fn plugin_roots_from_scan(
    roots: HashMap<String, PathBuf>,
) -> HashMap<String, PluginRootEntry> {
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
            match serde_json::from_str(&s) {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::warn!(
                        target: "oclive_plugin",
                        path = %p.display(),
                        error = %e,
                        "oclive_host_plugins.json invalid; using defaults"
                    );
                    Self::default()
                }
            }
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

/// Container directories holding plugin packages (`plugins/`, etc.) for scan and (developer mode) file watching.
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

pub(crate) fn parse_ready_line(
    line: &str,
    prefix: &str,
    plugin_id: &str,
    grants: &HighRiskGrantStore,
) -> Option<String> {
    let t = line.trim();
    let rest = t.strip_prefix(prefix)?.trim();
    if !(rest.starts_with("http://") || rest.starts_with("https://")) {
        return None;
    }
    if !rpc_url_is_loopback(rest) && !grants.is_network_granted(plugin_id) {
        tracing::warn!(
            target: "oclive_plugin",
            plugin_id = %plugin_id,
            url = %rest,
            "directory plugin ready URL is not loopback and network:* is not granted"
        );
        return None;
    }
    Some(rest.to_string())
}

pub(crate) fn rpc_url_is_loopback(url: &str) -> bool {
    let rest = match url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    {
        Some(r) => r,
        None => return false,
    };
    let host = match rest.split('/').next() {
        Some(h) => h.split(':').next().unwrap_or(h),
        None => return false,
    };
    matches!(host, "127.0.0.1" | "localhost" | "[::1]") || host.eq_ignore_ascii_case("localhost")
}
