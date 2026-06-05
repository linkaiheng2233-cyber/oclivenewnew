//! Directory plugin local updates (zip overwrite); online version check reserved.

use crate::api::error::CommandError;
use crate::error::AppError;
use crate::infrastructure::directory_plugins::OclivePluginManifest;
use crate::state::{AppState, SharedAppState};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUpdateInfo {
    pub has_update: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Reserved: future hook to community site `GET /api/plugins/versions`; currently returns no update + explanatory message.
#[tauri::command]
pub fn check_plugin_updates(
    plugin_ids: Vec<String>,
    _state: State<'_, SharedAppState>,
) -> Result<HashMap<String, PluginUpdateInfo>, CommandError> {
    let mut out = HashMap::new();
    for id in plugin_ids {
        let t = id.trim().to_string();
        if t.is_empty() {
            continue;
        }
        out.insert(
            t,
            PluginUpdateInfo {
                has_update: false,
                latest_version: None,
                message: None,
            },
        );
    }
    Ok(out)
}

fn unzip_archive(zip_path: &Path, dst: &Path) -> Result<(), CommandError> {
    let file =
        File::open(zip_path).map_err(|e| AppError::InvalidParameter(format!("open zip: {e}")))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| AppError::InvalidParameter(format!("parse zip: {e}")))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::InvalidParameter(format!("zip entry {i}: {e}")))?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => {
                return Err(
                    AppError::InvalidParameter(format!("zip entry {i}: illegal path")).into(),
                );
            }
        };
        let outpath = dst.join(&rel);
        if entry.is_dir() || rel.to_string_lossy().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(AppError::from)?;
            continue;
        }
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent).map_err(AppError::from)?;
        }
        let mut outf = File::create(&outpath).map_err(AppError::from)?;
        io::copy(&mut entry, &mut outf).map_err(AppError::from)?;
    }
    Ok(())
}

fn find_manifest_root(dir: &Path) -> Result<PathBuf, CommandError> {
    let direct = dir.join("manifest.json");
    if direct.is_file() {
        return Ok(dir.to_path_buf());
    }
    let subs: Vec<_> = fs::read_dir(dir)
        .map_err(AppError::from)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    if subs.len() == 1 {
        let p = subs[0].path();
        if p.join("manifest.json").is_file() {
            return Ok(p);
        }
    }
    Err(AppError::InvalidParameter(
        "No valid manifest.json in zip (root or single top-level folder)".into(),
    )
    .into())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), CommandError> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
        let out = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&out).map_err(AppError::from)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(AppError::from)?;
            }
            fs::copy(entry.path(), &out).map_err(AppError::from)?;
        }
    }
    Ok(())
}

fn resolve_install_dir(state: &AppState, plugin_id: &str) -> PathBuf {
    let roots = state.directory_plugins.plugin_roots.read();
    if let Some(entry) = roots.get(plugin_id) {
        return entry.root.clone();
    }
    drop(roots);
    state
        .directory_plugins
        .app_data_dir()
        .join("plugins")
        .join(plugin_id)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Extract zip to a temp dir, verify `manifest.json` `id` matches `plugin_id`, then overwrite the install directory.
#[tauri::command]
pub fn extract_plugin_zip(
    zip_path: String,
    plugin_id: String,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()).into());
    }
    let zip_path = PathBuf::from(zip_path.trim());
    if !zip_path.is_file() {
        return Err(AppError::InvalidParameter(format!(
            "zip file not found: {}",
            zip_path.display()
        ))
        .into());
    }
    let zip_path = zip_path
        .canonicalize()
        .map_err(|e| AppError::InvalidParameter(format!("zip path: {e}")))?;

    let tmp = tempfile::tempdir().map_err(AppError::from)?;
    unzip_archive(&zip_path, tmp.path())?;
    let staged = find_manifest_root(tmp.path())?;
    let manifest =
        OclivePluginManifest::load_from_dir(&staged).map_err(AppError::InvalidParameter)?;
    if manifest.id.trim() != pid {
        return Err(AppError::InvalidParameter(format!(
            "manifest id={} does not match target plugin {}",
            manifest.id.trim(),
            pid
        ))
        .into());
    }

    install_staged_directory_plugin(&state, &staged, pid)?;
    Ok(())
}

fn install_staged_directory_plugin(
    state: &AppState,
    staged: &Path,
    plugin_id: &str,
) -> Result<(), CommandError> {
    let pid = plugin_id.trim();
    let target = resolve_install_dir(state, pid);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }

    state.directory_plugins.clear_plugin_process(pid);
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|e| AppError::InvalidParameter(format!("remove old plugin dir: {e}")))?;
    }
    fs::create_dir_all(&target).map_err(AppError::from)?;
    copy_dir_all(staged, &target)?;

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}

/// Install directory plugin from zip: reads `manifest.id` from the package; caller need not pass `plugin_id` upfront.
///
/// # Errors
///
/// Returns [`Err`] when the zip is missing, invalid, or `manifest.id` cannot be read.
#[tauri::command]
pub fn install_plugin_from_zip(
    zip_path: String,
    state: State<'_, SharedAppState>,
) -> Result<String, CommandError> {
    let zip_path = PathBuf::from(zip_path.trim());
    if !zip_path.is_file() {
        return Err(AppError::InvalidParameter(format!(
            "zip file not found: {}",
            zip_path.display()
        ))
        .into());
    }
    let zip_path = zip_path
        .canonicalize()
        .map_err(|e| AppError::InvalidParameter(format!("zip path: {e}")))?;

    let tmp = tempfile::tempdir().map_err(AppError::from)?;
    unzip_archive(&zip_path, tmp.path())?;
    let staged = find_manifest_root(tmp.path())?;
    let manifest =
        OclivePluginManifest::load_from_dir(&staged).map_err(AppError::InvalidParameter)?;
    let pid = manifest.id.trim().to_string();
    install_staged_directory_plugin(&state, &staged, &pid)?;
    Ok(pid)
}
