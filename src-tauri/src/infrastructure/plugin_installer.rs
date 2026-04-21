use crate::error::AppError;
use crate::infrastructure::directory_plugins::{parse_manifest_version, OclivePluginManifest};
use crate::infrastructure::plugin_state::PluginStateStore;
use crate::state::AppState;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    pub git: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexFile {
    #[serde(default, alias = "generated_at")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginIndexEntry>,
}

pub const DEFAULT_PLUGIN_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json";

fn plugins_dir(state: &AppState) -> PathBuf {
    state.directory_plugins.app_data_dir().join("plugins")
}

fn cache_path(state: &AppState) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin_index_cache.json")
}

fn plugin_state_store_path(state: &AppState) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin_state.json")
}

pub fn load_cached_index(state: &AppState) -> Result<PluginIndexFile, AppError> {
    let p = cache_path(state);
    if !p.exists() {
        return Ok(PluginIndexFile {
            generated_at: None,
            plugins: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&p)?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse plugin index cache failed: {}", e)))
}

pub fn sync_plugin_index_online(
    state: &AppState,
    index_url: Option<&str>,
) -> Result<PluginIndexFile, AppError> {
    let env_url = std::env::var("OCLIVE_PLUGIN_INDEX_URL").ok();
    let url = index_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| env_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .unwrap_or(DEFAULT_PLUGIN_INDEX_URL);
    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("index http client failed: {}", e)))?;
    let resp = cli
        .get(url)
        .send()
        .map_err(|e| AppError::Unknown(format!("sync plugin index failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::Unknown(format!(
            "sync plugin index status={} url={}",
            resp.status(),
            url
        )));
    }
    let text = resp
        .text()
        .map_err(|e| AppError::Unknown(format!("read plugin index response failed: {}", e)))?;
    let mut parsed: PluginIndexFile = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse plugins.json failed: {}", e)))?;
    parsed.plugins.sort_by(|a, b| a.id.cmp(&b.id));
    let cache = cache_path(state);
    if let Some(parent) = cache.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        &cache,
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| AppError::Unknown(format!("encode index cache failed: {}", e)))?,
    )?;
    Ok(parsed)
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<(), AppError> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd
        .output()
        .map_err(|e| AppError::Unknown(format!("git command failed: {}", e)))?;
    if !out.status.success() {
        return Err(AppError::Unknown(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

fn installed_version_map(state: &AppState) -> HashMap<String, semver::Version> {
    let mut out = HashMap::new();
    let roots = state.directory_plugins.plugin_roots.read();
    for (pid, root) in roots.iter() {
        if let Ok(manifest) = OclivePluginManifest::load_from_dir(root) {
            if let Some(v) = parse_manifest_version(&manifest.version) {
                out.insert(pid.clone(), v);
            }
        }
    }
    out
}

pub fn missing_dependencies(
    state: &AppState,
    deps: &HashMap<String, String>,
) -> Result<Vec<String>, AppError> {
    let versions = installed_version_map(state);
    let mut missing = Vec::new();
    for (dep_id, req_s) in deps {
        let dep = dep_id.trim();
        if dep.is_empty() {
            continue;
        }
        let req = VersionReq::parse(req_s.trim()).map_err(|e| {
            AppError::InvalidParameter(format!(
                "dependency range invalid dep={} req={} err={}",
                dep, req_s, e
            ))
        })?;
        match versions.get(dep) {
            None => missing.push(format!("{} ({})", dep, req_s)),
            Some(v) => {
                if !req.matches(v) {
                    missing.push(format!("{} (need {}, local {})", dep, req_s, v));
                }
            }
        }
    }
    Ok(missing)
}

pub fn install_plugin(
    state: &AppState,
    git_url: &str,
    deps: Option<&HashMap<String, String>>,
) -> Result<String, AppError> {
    if let Some(deps_map) = deps {
        let miss = missing_dependencies(state, deps_map)?;
        if !miss.is_empty() {
            return Err(AppError::InvalidParameter(format!(
                "[MISSING_DEPENDENCIES] {}",
                miss.join(" | ")
            )));
        }
    }
    let url = git_url.trim();
    if url.is_empty() {
        return Err(AppError::InvalidParameter("git_url required".into()));
    }
    let mut target = plugins_dir(state);
    fs::create_dir_all(&target)?;
    let name = url
        .split('/')
        .next_back()
        .unwrap_or("plugin")
        .trim_end_matches(".git")
        .trim();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("invalid git_url".into()));
    }
    target = target.join(name);
    if target.exists() {
        return Err(AppError::InvalidParameter(format!(
            "plugin dir already exists: {}",
            target.display()
        )));
    }
    run_git(
        &[
            "clone",
            "--depth",
            "1",
            url,
            target.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    let manifest = OclivePluginManifest::load_from_dir(&target)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir != target {
        if final_dir.exists() {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        fs::rename(&target, &final_dir)?;
    }
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub fn update_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .cloned()
            .ok_or_else(|| AppError::InvalidParameter(format!("plugin not found: {}", pid)))?
    };
    run_git(&["pull", "--ff-only"], Some(&root))?;
    let _ = OclivePluginManifest::load_from_dir(&root)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed after pull: {}", e)))?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}

fn remove_plugin_from_state_store(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let p = plugin_state_store_path(state);
    let mut store = PluginStateStore::load(&p);
    if let Some(g) = store.global.as_mut() {
        g.slots.disabled_plugins.retain(|x| x.trim() != pid);
        g.slots
            .slot_order
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        g.slots
            .disabled_slot_contributions
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        g.slots.slot_appearance.remove(pid);
        if g.shell_plugin_id.trim() == pid {
            g.shell_plugin_id.clear();
        }
    }
    for role in store.roles.values_mut() {
        role.slots.disabled_plugins.retain(|x| x.trim() != pid);
        role.slots
            .slot_order
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        role.slots
            .disabled_slot_contributions
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        role.slots.slot_appearance.remove(pid);
        if role.shell_plugin_id.trim() == pid {
            role.shell_plugin_id.clear();
        }
    }
    store
        .save(&p)
        .map_err(|e| AppError::Unknown(format!("save plugin_state failed: {}", e)))?;
    let _ = state.directory_plugins.reload_plugin_state();
    Ok(())
}

pub fn uninstall_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .cloned()
            .ok_or_else(|| AppError::InvalidParameter(format!("plugin not found: {}", pid)))?
    };
    state.directory_plugins.clear_plugin_process(pid);
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    remove_plugin_from_state_store(state, pid)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}
