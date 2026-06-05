use crate::env_flags;
use crate::error::AppError;
use crate::infrastructure::directory_plugins::{parse_manifest_version, OclivePluginManifest};
use crate::infrastructure::plugin_state::PluginStateStore;
use crate::state::AppState;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    /// Monorepo path inside `git` clone (e.g. `examples/directory-plugin-minimal`).
    #[serde(default, alias = "git_subdir")]
    pub git_subdir: Option<String>,
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

/// Main repo `data/plugins.json` raw URL (when awesome list is empty or dev override applies).
pub const FALLBACK_PLUGIN_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/oclivenewnew/main/data/plugins.json";

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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
fn fetch_index_url(url: &str, cli: &reqwest::Client) -> Result<PluginIndexFile, AppError> {
    let url = url.to_string();
    let cli = cli.clone();
    crate::utils::block_on::block_on_isolated(async move {
        let resp = cli
            .get(&url)
            .send()
            .await
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
            .await
            .map_err(|e| AppError::Unknown(format!("read plugin index response failed: {}", e)))?;
        serde_json::from_str(&text)
            .map_err(|e| AppError::Unknown(format!("parse plugins.json failed: {}", e)))
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("index http client failed: {}", e)))?;
    let mut parsed = fetch_index_url(url, &cli)?;
    if parsed.plugins.is_empty() && url.contains("awesome-oclive-plugins") {
        if let Ok(fb) = fetch_index_url(FALLBACK_PLUGIN_INDEX_URL, &cli) {
            if !fb.plugins.is_empty() {
                parsed = fb;
            }
        }
    }
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
    for (pid, entry) in roots.iter() {
        if let Ok(manifest) = OclivePluginManifest::load_from_dir(&entry.root) {
            if let Some(v) = parse_manifest_version(&manifest.version) {
                out.insert(pid.clone(), v);
            }
        }
    }
    out
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn install_plugin(
    state: &AppState,
    git_url: &str,
    git_subdir: Option<&str>,
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
    let url = resolve_git_clone_url(git_url.trim());
    if url.is_empty() {
        return Err(AppError::InvalidParameter("git_url required".into()));
    }
    let base = plugins_dir(state);
    fs::create_dir_all(&base)?;
    let clone_label = url
        .split('/')
        .next_back()
        .unwrap_or("plugin")
        .trim_end_matches(".git")
        .trim();
    if clone_label.is_empty() {
        return Err(AppError::InvalidParameter("invalid git_url".into()));
    }
    let clone_dir = base.join(format!(".clone-{}", clone_label));
    if clone_dir.exists() {
        fs::remove_dir_all(&clone_dir).map_err(|e| {
            AppError::Unknown(format!(
                "failed to clear previous clone dir {}: {}",
                clone_dir.display(),
                e
            ))
        })?;
    }
    git_clone_with_fallback(git_url.trim(), &url, &clone_dir)?;
    let plugin_root = resolve_plugin_root_after_clone(&clone_dir, git_subdir)?;
    let manifest = OclivePluginManifest::load_from_dir(&plugin_root)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        cleanup_clone_dir(&clone_dir, &plugin_root);
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = base.join(pid.as_str());
    if final_dir.exists() {
        cleanup_clone_dir(&clone_dir, &plugin_root);
        return Err(AppError::InvalidParameter(format!(
            "target plugin id already exists: {}",
            final_dir.display()
        )));
    }
    if plugin_root == clone_dir {
        fs::rename(&clone_dir, &final_dir).map_err(|e| {
            AppError::Unknown(format!(
                "move plugin into {} failed: {}",
                final_dir.display(),
                e
            ))
        })?;
    } else {
        fs::rename(&plugin_root, &final_dir).map_err(|e| {
            AppError::Unknown(format!(
                "move plugin into {} failed: {}",
                final_dir.display(),
                e
            ))
        })?;
        let _ = fs::remove_dir_all(&clone_dir);
    }
    verify_plugin_signature_strict(&final_dir, &pid)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

fn plugin_signature_strict_enabled() -> bool {
    env_flags::env_flag_enabled("OCLIVE_PLUGIN_SIGNATURE_STRICT")
}

#[derive(Debug, Deserialize)]
struct PluginSignatureFile {
    plugin_id: String,
    sha256: String,
    #[serde(default)]
    archive: String,
}

/// When `OCLIVE_PLUGIN_SIGNATURE_STRICT=1`, require a sidecar `{plugin_id}.signature.json` and matching `{plugin_id}.oclive-plugin` SHA-256.
///
/// # Errors
///
/// Returns [`AppError::InvalidParameter`] when strict mode is on and verification fails.
fn verify_plugin_signature_strict(plugin_dir: &Path, plugin_id: &str) -> Result<(), AppError> {
    if !plugin_signature_strict_enabled() {
        return Ok(());
    }
    let parent = plugin_dir
        .parent()
        .ok_or_else(|| AppError::InvalidParameter("plugin install parent missing".into()))?;
    let sig_path = parent.join(format!("{plugin_id}.signature.json"));
    if !sig_path.is_file() {
        return Err(AppError::InvalidParameter(format!(
            "OCLIVE_PLUGIN_SIGNATURE_STRICT: missing signature file {}",
            sig_path.display()
        )));
    }
    let raw = fs::read_to_string(&sig_path).map_err(AppError::IoError)?;
    let sig: PluginSignatureFile = serde_json::from_str(&raw)
        .map_err(|e| AppError::InvalidParameter(format!("invalid signature json: {e}")))?;
    if sig.plugin_id.trim() != plugin_id {
        return Err(AppError::InvalidParameter(format!(
            "signature plugin_id mismatch: expected {plugin_id}, got {}",
            sig.plugin_id.trim()
        )));
    }
    let archive_name = if sig.archive.trim().is_empty() {
        format!("{plugin_id}.oclive-plugin")
    } else {
        sig.archive.trim().to_string()
    };
    let archive_path = parent.join(&archive_name);
    if !archive_path.is_file() {
        return Err(AppError::InvalidParameter(format!(
            "signature archive not found: {}",
            archive_path.display()
        )));
    }
    let blob = fs::read(&archive_path).map_err(AppError::IoError)?;
    let mut hasher = Sha256::new();
    hasher.update(&blob);
    let digest = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let expected = sig.sha256.trim().to_lowercase();
    if digest != expected {
        return Err(AppError::InvalidParameter(format!(
            "plugin archive sha256 mismatch for {plugin_id}: expected {expected}, got {digest}"
        )));
    }
    Ok(())
}

fn resolve_git_clone_url(https_git: &str) -> String {
    if let Ok(root) = std::env::var("OCLIVE_LOCAL_MONOREPO") {
        let root = root.trim();
        if !root.is_empty()
            && https_git.contains("oclivenewnew")
            && Path::new(root).join("examples").is_dir()
        {
            let path = root.replace('\\', "/");
            return if path.starts_with('/') {
                format!("file://{path}")
            } else {
                format!("file:///{path}")
            };
        }
    }
    https_git.to_string()
}

fn git_clone_with_fallback(
    https_git: &str,
    primary_url: &str,
    clone_dir: &Path,
) -> Result<(), AppError> {
    let mut urls = vec![primary_url.to_string()];
    if primary_url != https_git {
        urls.push(https_git.to_string());
    } else if let Some(file_url) = local_monorepo_file_git_url(https_git) {
        if !urls.iter().any(|u| u == &file_url) {
            urls.push(file_url);
        }
    }
    let mut last = String::new();
    for url in urls {
        if clone_dir.exists() {
            let _ = fs::remove_dir_all(clone_dir);
        }
        match run_git(
            &[
                "clone",
                "--depth",
                "1",
                url.as_str(),
                clone_dir.to_string_lossy().as_ref(),
            ],
            None,
        ) {
            Ok(()) => return Ok(()),
            Err(e) => last = format!("{} ({})", url, e),
        }
    }
    Err(AppError::Unknown(format!("git clone failed: {}", last)))
}

fn local_monorepo_file_git_url(https_git: &str) -> Option<String> {
    let root = std::env::var("OCLIVE_LOCAL_MONOREPO")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    if !https_git.contains("oclivenewnew") {
        return None;
    }
    let examples = Path::new(&root).join("examples");
    if !examples.is_dir() {
        return None;
    }
    let path = root.replace('\\', "/");
    Some(if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    })
}

fn resolve_plugin_root_after_clone(
    clone_dir: &Path,
    git_subdir: Option<&str>,
) -> Result<PathBuf, AppError> {
    let sub = git_subdir.map(str::trim).filter(|s| !s.is_empty());
    let root = match sub {
        None => clone_dir.to_path_buf(),
        Some(rel) => {
            let rel = normalize_git_subdir(rel)?;
            let p = clone_dir.join(&rel);
            if !p.is_dir() {
                let _ = fs::remove_dir_all(clone_dir);
                return Err(AppError::InvalidParameter(format!(
                    "gitSubdir not found in clone: {}",
                    rel
                )));
            }
            let clone_canon = fs::canonicalize(clone_dir).map_err(|e| {
                AppError::InvalidParameter(format!("clone dir canonicalize failed: {e}"))
            })?;
            let joined_canon = fs::canonicalize(&p).map_err(|e| {
                AppError::InvalidParameter(format!("gitSubdir canonicalize failed: {e}"))
            })?;
            if !joined_canon.starts_with(&clone_canon) {
                let _ = fs::remove_dir_all(clone_dir);
                return Err(AppError::InvalidParameter(
                    "gitSubdir escapes clone root (path traversal rejected)".into(),
                ));
            }
            joined_canon
        }
    };
    Ok(root)
}

fn normalize_git_subdir(rel: &str) -> Result<String, AppError> {
    let rel = rel.replace('\\', "/");
    let rel = rel.trim().trim_matches('/');
    if rel.is_empty() {
        return Ok(String::new());
    }
    if rel.contains("..") {
        return Err(AppError::InvalidParameter(
            "gitSubdir must not contain '..'".into(),
        ));
    }
    if Path::new(rel).is_absolute() {
        return Err(AppError::InvalidParameter(
            "gitSubdir must be a relative path".into(),
        ));
    }
    Ok(rel.to_string())
}

fn cleanup_clone_dir(clone_dir: &Path, plugin_root: &Path) {
    if plugin_root != clone_dir && plugin_root.starts_with(clone_dir) {
        let _ = fs::remove_dir_all(plugin_root);
    }
    let _ = fs::remove_dir_all(clone_dir);
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn update_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .map(|entry| entry.root.clone())
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub fn uninstall_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .map(|entry| entry.root.clone())
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
