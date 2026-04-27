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
pub struct PluginZipPermissionPreview {
    pub plugin_id: String,
    pub permissions: Vec<String>,
}

fn bridge_command_to_permission_token(cmd: &str) -> String {
    match cmd {
        "get_conversation" => "read:conversation".to_string(),
        "get_roles" => "read:roles".to_string(),
        "get_current_role" => "read:current_role".to_string(),
        "update_memory" | "delete_memory" => "write:memory".to_string(),
        "update_emotion" => "write:emotion".to_string(),
        "update_event" => "write:event".to_string(),
        "update_prompt" => "write:prompt".to_string(),
        "export_conversation" => "export:conversation".to_string(),
        "import_role" => "import:role".to_string(),
        "delete_role" => "delete:role".to_string(),
        "update_settings" => "write:settings".to_string(),
        "get_conversation_list" => "read:conversations".to_string(),
        _ => cmd.to_string(),
    }
}

fn bridge_permissions_from_manifest(manifest: &OclivePluginManifest) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(sh) = &manifest.shell {
        if let Some(b) = &sh.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if t.is_empty() {
                    continue;
                }
                let perm = if t.contains(':') {
                    t.to_string()
                } else {
                    bridge_command_to_permission_token(t)
                };
                out.push(perm);
            }
        }
    }
    for us in &manifest.ui_slots {
        if let Some(b) = &us.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if t.is_empty() {
                    continue;
                }
                let perm = if t.contains(':') {
                    t.to_string()
                } else {
                    bridge_command_to_permission_token(t)
                };
                out.push(perm);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

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
    const MAX_FILES: usize = 2000;
    const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
    const MAX_SINGLE_BYTES: u64 = 10 * 1024 * 1024;
    let mut files = 0usize;
    let mut total: u64 = 0;
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
        files += 1;
        if files > MAX_FILES {
            return Err(format!(
                "[ZIP_TOO_MANY_FILES] zip 文件过多（>{}）",
                MAX_FILES
            ));
        }
        let sz = entry.size();
        if sz > MAX_SINGLE_BYTES {
            return Err(format!(
                "[ZIP_SINGLE_FILE_TOO_LARGE] 单文件过大（{} bytes）: {}",
                sz,
                rel.to_string_lossy()
            ));
        }
        total = total.saturating_add(sz);
        if total > MAX_TOTAL_BYTES {
            return Err(format!(
                "[ZIP_TOTAL_TOO_LARGE] 总大小过大（>{} bytes）",
                MAX_TOTAL_BYTES
            ));
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
    accepted_permissions: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.directory_plugins.host().developer_effective() {
        return Err("developer mode required for zip sideload".to_string());
    }
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
    // 开发者模式侧载：支持手动勾选授权；未提供时默认按 manifest seed（便于调试）
    let declared = bridge_permissions_from_manifest(&manifest);
    let mut perms = accepted_permissions.unwrap_or_else(|| declared.clone());
    perms = perms
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    perms.sort();
    perms.dedup();
    // 若用户选择的权限不在声明里，则拒绝（防止滥用参数注入授权）
    if !perms.is_empty() {
        let declared_set: std::collections::HashSet<String> =
            declared.iter().map(|s| s.trim().to_string()).collect();
        let ok = perms.iter().all(|p| declared_set.contains(p.trim()));
        if !ok {
            return Err(
                "accepted_permissions must be a subset of declared permissions".to_string(),
            );
        }
    }
    tauri::async_runtime::block_on(async {
        for p in perms {
            let _ = state
                .db_manager
                .upsert_plugin_permission_grant(pid, p.as_str(), true)
                .await;
        }
    });
    Ok(())
}

/// 开发者模式：预览 zip 中 manifest 的 bridge 权限（用于“安装/更新前勾选授权”）。
#[tauri::command]
pub fn preview_plugin_zip_permissions(
    zip_path: String,
    state: State<'_, AppState>,
) -> Result<PluginZipPermissionPreview, String> {
    if !state.directory_plugins.host().developer_effective() {
        return Err("developer mode required for zip preview".to_string());
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
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err("manifest.id required".to_string());
    }
    let permissions = bridge_permissions_from_manifest(&manifest);
    Ok(PluginZipPermissionPreview {
        plugin_id: pid,
        permissions,
    })
}
