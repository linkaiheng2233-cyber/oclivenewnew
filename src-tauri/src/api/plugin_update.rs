//! 目录插件本地更新（zip 覆盖）；在线版本检查预留。

use crate::infrastructure::directory_plugins::OclivePluginManifest;
use crate::state::AppState;
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

/// 预留：未来对接社区站 `GET /api/plugins/versions`；当前返回无更新 + 说明文案。
#[tauri::command]
pub fn check_plugin_updates(
    plugin_ids: Vec<String>,
    _state: State<'_, AppState>,
) -> Result<HashMap<String, PluginUpdateInfo>, String> {
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
                message: Some("在线版本检查尚未接入".to_string()),
            },
        );
    }
    Ok(out)
}

fn unzip_archive(zip_path: &Path, dst: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("打开 zip: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("解析 zip: {}", e))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip 条目 {}: {}", i, e))?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => {
                return Err(format!("zip 条目 {}: 非法路径", i));
            }
        };
        let outpath = dst.join(&rel);
        if entry.is_dir() || rel.to_string_lossy().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut outf = File::create(&outpath).map_err(|e| e.to_string())?;
        io::copy(&mut entry, &mut outf).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn find_manifest_root(dir: &Path) -> Result<PathBuf, String> {
    let direct = dir.join("manifest.json");
    if direct.is_file() {
        return Ok(dir.to_path_buf());
    }
    let subs: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    if subs.len() == 1 {
        let p = subs[0].path();
        if p.join("manifest.json").is_file() {
            return Ok(p);
        }
    }
    Err("zip 中未找到有效的 manifest.json（根目录或单一顶层目录内）".to_string())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src).map_err(|e| e.to_string())?;
        let out = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(entry.path(), &out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn resolve_install_dir(state: &AppState, plugin_id: &str) -> PathBuf {
    let roots = state.directory_plugins.plugin_roots.read();
    if let Some(p) = roots.get(plugin_id) {
        return p.clone();
    }
    drop(roots);
    state
        .directory_plugins
        .app_data_dir()
        .join("plugins")
        .join(plugin_id)
}

/// 解压 zip 到临时目录，校验 `manifest.json` 的 `id` 与 `plugin_id` 一致后覆盖安装目录。
#[tauri::command]
pub fn extract_plugin_zip(
    zip_path: String,
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id required".to_string());
    }
    let zip_path = PathBuf::from(zip_path.trim());
    if !zip_path.is_file() {
        return Err(format!("zip 文件不存在: {}", zip_path.display()));
    }
    let zip_path = zip_path
        .canonicalize()
        .map_err(|e| format!("zip 路径: {}", e))?;

    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
    unzip_archive(&zip_path, tmp.path())?;
    let staged = find_manifest_root(tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(&staged)?;
    if manifest.id.trim() != pid {
        return Err(format!(
            "manifest id={} 与目标插件 {} 不一致",
            manifest.id.trim(),
            pid
        ));
    }

    let target = resolve_install_dir(&state, pid);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    state.directory_plugins.clear_plugin_process(pid);
    if target.exists() {
        fs::remove_dir_all(&target).map_err(|e| format!("删除旧插件目录: {}", e))?;
    }
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    copy_dir_all(&staged, &target)?;

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}
