use crate::error::AppError;
use crate::infrastructure::directory_plugins::{parse_manifest_version, OclivePluginManifest};
use crate::infrastructure::plugin_state::PluginStateStore;
use crate::state::AppState;
use semver::VersionReq;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub use oclive_kernel_runtime::infrastructure::plugin_archive::{
    extract_oclive_plugin_archive, peek_plugin_id_from_archive_bytes,
};
pub use oclive_kernel_runtime::infrastructure::plugin_index_sync::{
    load_plugin_index_cache, plugin_index_cache_path_for_source, plugin_index_default_cache_path,
    sync_plugin_index_from_url, DEFAULT_PLUGIN_INDEX_URL,
};
pub use oclive_kernel_runtime::infrastructure::plugin_package_verify::verify_plugin_package_signature_text;
pub use oclive_kernel_runtime::models::plugin_market_index::{
    PluginIndexEntry, PluginIndexFile, PluginIndexModulePluginSpec, PluginIndexModuleSpec,
    PluginIndexProfileSpec, PluginIndexVersionEntry, PublisherPublicKey,
};

pub type PluginInstallMeta = crate::models::dto::PluginInstallMetaDto;

// NOTE: 权限 token 映射与种子逻辑已迁移到 API 层：
// - 市场安装：只写入用户 consent 的权限子集
// - 开发者模式侧载：由 extract_plugin_zip 等命令按 manifest 种子写入

pub fn update_install_meta_permissions(
    state: &AppState,
    plugin_id: &str,
    declared_permissions: Vec<String>,
    granted_permissions: Vec<String>,
) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let root = plugins_dir(state).join(pid);
    let Some(mut meta) = read_install_meta(&root) else {
        return Ok(());
    };
    let mut declared = declared_permissions
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    declared.sort();
    declared.dedup();
    let mut granted = granted_permissions
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    granted.sort();
    granted.dedup();
    meta.declared_permissions = declared;
    meta.granted_permissions = granted;
    write_install_meta(&root, &meta)?;
    Ok(())
}

fn plugins_dir(state: &AppState) -> PathBuf {
    state.directory_plugins.app_data_dir().join("plugins")
}

fn plugin_state_store_path(state: &AppState) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin_state.json")
}

pub fn load_cached_index(state: &AppState) -> Result<PluginIndexFile, AppError> {
    let p = plugin_index_default_cache_path(state.directory_plugins.app_data_dir());
    load_plugin_index_cache(&p)
}

pub fn load_cached_index_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return load_cached_index(state);
    }
    let p = plugin_index_cache_path_for_source(state.directory_plugins.app_data_dir(), url);
    load_plugin_index_cache(&p)
}

pub fn install_plugin_from_archive_bytes_overwrite(
    state: &AppState,
    bytes: &[u8],
    overwrite: bool,
) -> Result<String, AppError> {
    let tmp = plugins_install_temp_dir(state)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir.exists() {
        if !overwrite {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        let _ = fs::remove_dir_all(&final_dir);
    }
    fs::create_dir_all(plugins_dir(state))?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    std::mem::forget(tmp);
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
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
    let cache = plugin_index_default_cache_path(state.directory_plugins.app_data_dir());
    sync_plugin_index_from_url(url, &cache)
}

pub fn sync_plugin_index_online_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return sync_plugin_index_online(state, None);
    }
    let cache = plugin_index_cache_path_for_source(state.directory_plugins.app_data_dir(), url);
    sync_plugin_index_from_url(url, &cache)
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

fn plugins_install_temp_dir(state: &AppState) -> Result<TempDir, AppError> {
    let root = state.directory_plugins.app_data_dir().join("tmp");
    let _ = fs::create_dir_all(&root);
    TempDir::new_in(root).map_err(AppError::IoError)
}

fn write_install_meta(root: &Path, meta: &PluginInstallMeta) -> Result<(), AppError> {
    let p = root.join(".oclive_install.json");
    let raw = serde_json::to_string_pretty(meta).map_err(AppError::from)?;
    fs::write(p, raw)?;
    Ok(())
}

pub fn read_install_meta(root: &Path) -> Option<PluginInstallMeta> {
    oclive_kernel_runtime::infrastructure::directory_plugins::read_plugin_install_meta(root)
}

pub fn install_plugin_from_archive_bytes(
    state: &AppState,
    bytes: &[u8],
) -> Result<String, AppError> {
    let tmp = plugins_install_temp_dir(state)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir.exists() {
        return Err(AppError::InvalidParameter(format!(
            "target plugin id already exists: {}",
            final_dir.display()
        )));
    }
    fs::create_dir_all(plugins_dir(state))?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    // rename 后 tmp 不再拥有目录；阻止 drop 尝试清理不存在路径
    std::mem::forget(tmp);
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 注意：权限授予必须来自“用户同意”（市场安装）或“开发者模式侧载”流程；
    // 这里不再自动授予 manifest 种子权限，避免绕开索引声明与用户授权。
    Ok(pid)
}

pub fn install_plugin_from_download_urls(
    state: &AppState,
    index_entry: &PluginIndexEntry,
    download_url: &str,
    signature_url: &str,
) -> Result<String, AppError> {
    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Unknown(format!("download http client failed: {}", e)))?;
    let archive_bytes = cli
        .get(download_url)
        .send()
        .map_err(|e| AppError::Unknown(format!("download plugin failed: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Unknown(format!("download plugin status failed: {}", e)))?
        .bytes()
        .map_err(|e| AppError::Unknown(format!("read plugin bytes failed: {}", e)))?
        .to_vec();
    let sig_text = cli
        .get(signature_url)
        .send()
        .map_err(|e| AppError::Unknown(format!("download signature failed: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Unknown(format!("download signature status failed: {}", e)))?
        .text()
        .map_err(|e| AppError::Unknown(format!("read signature text failed: {}", e)))?;
    verify_plugin_package_signature_text(index_entry, &sig_text, &archive_bytes)?;
    install_plugin_from_archive_bytes(state, &archive_bytes)
}

pub fn install_plugin_from_git_tag(
    state: &AppState,
    git_url: &str,
    tag: &str,
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
    let tag = tag.trim();
    if url.is_empty() {
        return Err(AppError::InvalidParameter("git_url required".into()));
    }
    if tag.is_empty() {
        return Err(AppError::InvalidParameter("git tag required".into()));
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
            "--branch",
            tag,
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
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "git_tag".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: Some(tag.to_string()),
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 不在 installer 层自动授予权限（见 install_plugin_from_archive_bytes 注释）
    Ok(pid)
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
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "git".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 不在 installer 层自动授予权限（见 install_plugin_from_archive_bytes 注释）
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
    if let Some(meta) = read_install_meta(&root) {
        if let Some(tag) = meta
            .pinned_tag
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Err(AppError::InvalidParameter(format!(
                "[PLUGIN_PINNED_VERSION] plugin is pinned to tag {}; update via market version install",
                tag
            )));
        }
    }
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
