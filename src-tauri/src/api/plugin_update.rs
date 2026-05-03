//! 目录插件本地更新（zip 覆盖）；在线版本检查预留。

use crate::error::AppError;
use crate::infrastructure::directory_plugins::OclivePluginManifest;
use crate::state::AppState;
use oclive_kernel_runtime::infrastructure::plugin_archive::extract_oclive_plugin_archive_file;
use oclive_kernel_runtime::infrastructure::plugin_layout::{copy_plugin_tree, find_plugin_manifest_root};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use super::bridge_manifest_permissions::bridge_permission_tokens_from_manifest;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginZipPermissionPreview {
    pub plugin_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDirPermissionPreview {
    pub plugin_id: String,
    pub permissions: Vec<String>,
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
pub async fn extract_plugin_zip(
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
    extract_oclive_plugin_archive_file(&zip_path, tmp.path())
        .map_err(|e: AppError| e.to_frontend_error())?;
    let staged = find_plugin_manifest_root(tmp.path()).map_err(|e: AppError| e.to_frontend_error())?;
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
    copy_plugin_tree(&staged, &target).map_err(|e: AppError| e.to_frontend_error())?;

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 开发者模式侧载：支持手动勾选授权；未提供时默认按 manifest seed（便于调试）
    let declared = bridge_permission_tokens_from_manifest(&manifest);
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
    for p in perms {
        let _ = state
            .db_manager
            .upsert_plugin_permission_grant(pid, p.as_str(), true)
            .await;
    }
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
    extract_oclive_plugin_archive_file(&zip_path, tmp.path())
        .map_err(|e: AppError| e.to_frontend_error())?;
    let staged = find_plugin_manifest_root(tmp.path()).map_err(|e: AppError| e.to_frontend_error())?;
    let manifest = OclivePluginManifest::load_from_dir(&staged)?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err("manifest.id required".to_string());
    }
    let permissions = bridge_permission_tokens_from_manifest(&manifest);
    Ok(PluginZipPermissionPreview {
        plugin_id: pid,
        permissions,
    })
}

/// 开发者模式：预览目录插件的 manifest bridge 权限。
#[tauri::command]
pub fn preview_plugin_dir_permissions(
    dir_path: String,
    state: State<'_, AppState>,
) -> Result<PluginDirPermissionPreview, String> {
    if !state.directory_plugins.host().developer_effective() {
        return Err("developer mode required for dir preview".to_string());
    }
    let dir_path = PathBuf::from(dir_path.trim());
    if !dir_path.is_dir() {
        return Err(format!("目录不存在: {}", dir_path.display()));
    }
    let dir_path = dir_path
        .canonicalize()
        .map_err(|e| format!("dir 路径: {}", e))?;
    let manifest = OclivePluginManifest::load_from_dir(&dir_path)?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err("manifest.id required".to_string());
    }
    let permissions = bridge_permission_tokens_from_manifest(&manifest);
    Ok(PluginDirPermissionPreview {
        plugin_id: pid,
        permissions,
    })
}

/// 开发者模式：安装目录插件（复制目录到 app_data/plugins/{id}），并按用户勾选写入 grants。
#[tauri::command]
pub fn install_plugin_dir(
    dir_path: String,
    plugin_id: String,
    accepted_permissions: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !state.directory_plugins.host().developer_effective() {
        return Err("developer mode required for dir sideload".to_string());
    }
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id required".to_string());
    }
    let dir_path = PathBuf::from(dir_path.trim());
    if !dir_path.is_dir() {
        return Err(format!("目录不存在: {}", dir_path.display()));
    }
    let dir_path = dir_path
        .canonicalize()
        .map_err(|e| format!("dir 路径: {}", e))?;
    let manifest = OclivePluginManifest::load_from_dir(&dir_path)?;
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
    copy_plugin_tree(&dir_path, &target).map_err(|e: AppError| e.to_frontend_error())?;

    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    let declared = bridge_permission_tokens_from_manifest(&manifest);
    let mut perms = accepted_permissions.unwrap_or_else(|| declared.clone());
    perms = perms
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    perms.sort();
    perms.dedup();
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
