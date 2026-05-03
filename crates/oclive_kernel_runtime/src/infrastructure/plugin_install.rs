//! 目录插件安装：归档解压、市场下载签名校验、Git clone；路径由宿主传入。

use crate::error::{AppError, Result};
use crate::infrastructure::directory_plugins::{
    parse_manifest_version, read_plugin_install_meta, write_plugin_install_meta,
    OclivePluginManifest,
};
#[cfg(feature = "role-pack-zip")]
use crate::infrastructure::plugin_archive::extract_oclive_plugin_archive;
#[cfg(feature = "role-pack-zip")]
use crate::infrastructure::plugin_package_verify::verify_plugin_package_signature_text;
use crate::infrastructure::plugin_state::PluginStateStore;
use crate::models::dto::PluginInstallMetaDto;
use crate::models::plugin_market_index::PluginIndexEntry;
use semver::VersionReq;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub fn plugins_install_temp_dir(app_data_dir: &Path) -> Result<TempDir> {
    let root = app_data_dir.join("tmp");
    let _ = fs::create_dir_all(&root);
    TempDir::new_in(root).map_err(AppError::IoError)
}

#[must_use]
pub fn plugin_state_store_default_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("plugin_state.json")
}

pub fn installed_plugin_version_map(
    plugin_roots: &HashMap<String, PathBuf>,
) -> HashMap<String, semver::Version> {
    let mut out = HashMap::new();
    for (pid, root) in plugin_roots.iter() {
        if let Ok(manifest) = OclivePluginManifest::load_from_dir(root) {
            if let Some(v) = parse_manifest_version(&manifest.version) {
                out.insert(pid.clone(), v);
            }
        }
    }
    out
}

pub fn missing_plugin_dependencies(
    installed_versions: &HashMap<String, semver::Version>,
    deps: &HashMap<String, String>,
) -> Result<Vec<String>> {
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
        match installed_versions.get(dep) {
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

pub fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().map_err(|e| {
        AppError::InvalidParameter(format!("[PLUGIN_INSTALL_GIT] command failed: {}", e))
    })?;
    if !out.status.success() {
        return Err(AppError::InvalidParameter(format!(
            "[PLUGIN_INSTALL_GIT] git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

fn git_clone_folder_name(git_url: &str) -> Result<String> {
    let name = git_url
        .split('/')
        .next_back()
        .unwrap_or("plugin")
        .trim_end_matches(".git")
        .trim();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("invalid git_url".into()));
    }
    Ok(name.to_string())
}

pub fn update_install_meta_permissions_at(
    plugin_root: &Path,
    declared_permissions: Vec<String>,
    granted_permissions: Vec<String>,
) -> Result<()> {
    let Some(mut meta) = read_plugin_install_meta(plugin_root) else {
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
    write_plugin_install_meta(plugin_root, &meta)?;
    Ok(())
}

#[cfg(feature = "role-pack-zip")]
fn finalize_archive_install(
    plugins_root: &Path,
    tmp: TempDir,
    overwrite: bool,
    meta: &PluginInstallMetaDto,
) -> Result<String> {
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_MANIFEST] {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_root.join(pid.as_str());
    if final_dir.exists() {
        if !overwrite {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        let _ = fs::remove_dir_all(&final_dir);
    }
    fs::create_dir_all(plugins_root)?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_plugin_install_meta(final_dir.as_path(), meta);
    std::mem::forget(tmp);
    Ok(pid)
}

#[cfg(feature = "role-pack-zip")]
pub fn install_plugin_from_archive_bytes_overwrite_at(
    plugins_root: &Path,
    app_data_dir: &Path,
    bytes: &[u8],
    overwrite: bool,
    meta: &PluginInstallMetaDto,
) -> Result<String> {
    let tmp = plugins_install_temp_dir(app_data_dir)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    finalize_archive_install(plugins_root, tmp, overwrite, meta)
}

#[cfg(not(feature = "role-pack-zip"))]
pub fn install_plugin_from_archive_bytes_overwrite_at(
    _plugins_root: &Path,
    _app_data_dir: &Path,
    _bytes: &[u8],
    _overwrite: bool,
    _meta: &PluginInstallMetaDto,
) -> Result<String> {
    Err(AppError::InvalidParameter(
        "[PLUGIN_INSTALL_BUILD] compiled without role-pack-zip; plugin archive install unavailable"
            .into(),
    ))
}

#[cfg(feature = "role-pack-zip")]
pub fn install_plugin_from_archive_bytes_at(
    plugins_root: &Path,
    app_data_dir: &Path,
    bytes: &[u8],
) -> Result<String> {
    let tmp = plugins_install_temp_dir(app_data_dir)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_MANIFEST] {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_root.join(pid.as_str());
    if final_dir.exists() {
        return Err(AppError::InvalidParameter(format!(
            "target plugin id already exists: {}",
            final_dir.display()
        )));
    }
    fs::create_dir_all(plugins_root)?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_plugin_install_meta(
        &final_dir,
        &PluginInstallMetaDto {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    std::mem::forget(tmp);
    Ok(pid)
}

#[cfg(not(feature = "role-pack-zip"))]
pub fn install_plugin_from_archive_bytes_at(
    _plugins_root: &Path,
    _app_data_dir: &Path,
    _bytes: &[u8],
) -> Result<String> {
    Err(AppError::InvalidParameter(
        "[PLUGIN_INSTALL_BUILD] compiled without role-pack-zip; plugin archive install unavailable"
            .into(),
    ))
}

#[cfg(feature = "role-pack-zip")]
pub async fn install_plugin_from_download_urls_at(
    plugins_root: &Path,
    app_data_dir: &Path,
    index_entry: &PluginIndexEntry,
    download_url: &str,
    signature_url: &str,
) -> Result<String> {
    let download_url = download_url.to_string();
    let signature_url = signature_url.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] http client: {}", e))
        })?;
    let archive_bytes = cli
        .get(&download_url)
        .send()
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] get: {}", e))
        })?
        .error_for_status()
        .map_err(|e| {
            AppError::InvalidParameter(format!(
                "[PLUGIN_INSTALL_DOWNLOAD] archive status: {}",
                e
            ))
        })?
        .bytes()
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] read archive: {}", e))
        })?
        .to_vec();
    let sig_text = cli
        .get(&signature_url)
        .send()
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] sig get: {}", e))
        })?
        .error_for_status()
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] sig status: {}", e))
        })?
        .text()
        .await
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_INSTALL_DOWNLOAD] read sig: {}", e))
        })?;
    verify_plugin_package_signature_text(index_entry, &sig_text, &archive_bytes)?;
    install_plugin_from_archive_bytes_at(plugins_root, app_data_dir, &archive_bytes)
}

#[cfg(not(feature = "role-pack-zip"))]
pub async fn install_plugin_from_download_urls_at(
    _plugins_root: &Path,
    _app_data_dir: &Path,
    _index_entry: &PluginIndexEntry,
    _download_url: &str,
    _signature_url: &str,
) -> Result<String> {
    Err(AppError::InvalidParameter(
        "[PLUGIN_INSTALL_BUILD] compiled without role-pack-zip; market archive install unavailable"
            .into(),
    ))
}

pub fn install_plugin_from_git_tag_at(
    plugins_root: &Path,
    git_url: &str,
    tag: &str,
    installed_versions: &HashMap<String, semver::Version>,
    deps: Option<&HashMap<String, String>>,
) -> Result<String> {
    if let Some(deps_map) = deps {
        let miss = missing_plugin_dependencies(installed_versions, deps_map)?;
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
    let mut target = plugins_root.to_path_buf();
    fs::create_dir_all(&target)?;
    let name = git_clone_folder_name(url)?;
    target = target.join(&name);
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
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_MANIFEST] {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_root.join(pid.as_str());
    if final_dir != target {
        if final_dir.exists() {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        fs::rename(&target, &final_dir)?;
    }
    let _ = write_plugin_install_meta(
        &final_dir,
        &PluginInstallMetaDto {
            install_method: "git_tag".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: Some(tag.to_string()),
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    Ok(pid)
}

pub fn install_plugin_from_git_head_at(
    plugins_root: &Path,
    git_url: &str,
    installed_versions: &HashMap<String, semver::Version>,
    deps: Option<&HashMap<String, String>>,
) -> Result<String> {
    if let Some(deps_map) = deps {
        let miss = missing_plugin_dependencies(installed_versions, deps_map)?;
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
    let mut target = plugins_root.to_path_buf();
    fs::create_dir_all(&target)?;
    let name = git_clone_folder_name(url)?;
    target = target.join(&name);
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
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_MANIFEST] {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_root.join(pid.as_str());
    if final_dir != target {
        if final_dir.exists() {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        fs::rename(&target, &final_dir)?;
    }
    let _ = write_plugin_install_meta(
        &final_dir,
        &PluginInstallMetaDto {
            install_method: "git".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    Ok(pid)
}

pub fn update_git_plugin_at(plugin_root: &Path) -> Result<()> {
    if let Some(meta) = read_plugin_install_meta(plugin_root) {
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
    run_git(&["pull", "--ff-only"], Some(plugin_root))?;
    let _ = OclivePluginManifest::load_from_dir(plugin_root)
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_MANIFEST] after pull: {}", e)))?;
    Ok(())
}

pub fn remove_plugin_from_plugin_state_file(store_path: &Path, plugin_id: &str) -> Result<()> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let mut store = PluginStateStore::load(store_path);
    store.remove_plugin_references(pid);
    store
        .save(store_path)
        .map_err(|e| AppError::DatabaseError(format!("[PLUGIN_STATE_PERSIST] {}", e)))?;
    Ok(())
}

/// 异步更新 `plugin_state.json`（`tokio::fs`），供宿主 async 卸载路径使用。
pub async fn remove_plugin_from_plugin_state_file_async(
    store_path: &Path,
    plugin_id: &str,
) -> Result<()> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let mut store = PluginStateStore::load_async(store_path).await;
    store.remove_plugin_references(pid);
    store
        .save_async(store_path)
        .await
        .map_err(|e| AppError::DatabaseError(format!("[PLUGIN_STATE_PERSIST] {}", e)))?;
    Ok(())
}
