use crate::infrastructure::deep_link::take_pending_install_git_urls;
use crate::infrastructure::directory_plugins::{parse_manifest_version, OclivePluginManifest};
use crate::infrastructure::plugin_data::ensure_default_config_for_manifest;
use crate::infrastructure::plugin_installer::{
    install_plugin, install_plugin_from_download_urls, install_plugin_from_git_tag,
    load_cached_index, load_cached_index_for_source, missing_dependencies, sync_plugin_index_online,
    sync_plugin_index_online_for_source, uninstall_plugin, update_plugin, PluginIndexEntry,
    PluginIndexFile, PluginIndexVersionEntry,
};
use crate::state::AppState;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketEntry {
    #[serde(flatten)]
    pub index: PluginIndexEntry,
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_version: Option<String>,
    pub has_update: bool,
    #[serde(default)]
    pub missing_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMarketSnapshot {
    pub plugins: Vec<PluginMarketEntry>,
    pub offline_mode: bool,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingProtocolInstall {
    pub git_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPluginFromMarketResponse {
    pub installed_plugin_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPluginFromGitRequest {
    pub git_url: String,
}

fn cmp_version(local: &str, remote: &str) -> bool {
    let lv = parse_manifest_version(local).or_else(|| Version::parse(local).ok());
    let rv = parse_manifest_version(remote).or_else(|| Version::parse(remote).ok());
    match (lv, rv) {
        (Some(l), Some(r)) => r > l,
        _ => remote.trim() != local.trim(),
    }
}

fn build_snapshot(
    state: &AppState,
    index: PluginIndexFile,
    offline_mode: bool,
    source: &str,
    warning: Option<String>,
) -> PluginMarketSnapshot {
    let mut local_map: HashMap<String, String> = HashMap::new();
    {
        let roots = state.directory_plugins.plugin_roots.read();
        for (pid, root) in roots.iter() {
            if let Ok(manifest) = OclivePluginManifest::load_from_dir(root) {
                local_map.insert(pid.clone(), manifest.version);
            }
        }
    }
    let mut plugins = Vec::with_capacity(index.plugins.len());
    for item in index.plugins {
        let latest = pick_latest_version(&item);
        let mut compat_item = item.clone();
        // 兼容旧前端：把 version 字段对齐到 latest（若存在）
        if let Some(v) = latest.as_ref().map(|x| x.version.trim()).filter(|s| !s.is_empty()) {
            compat_item.version = v.to_string();
        }
        let installed_version = local_map.get(&item.id).cloned();
        let installed = installed_version.is_some();
        let has_update = installed_version
            .as_deref()
            .map(|v| cmp_version(v, &compat_item.version))
            .unwrap_or(false);
        let missing = missing_dependencies(state, &compat_item.dependencies).unwrap_or_default();
        plugins.push(PluginMarketEntry {
            index: compat_item,
            installed,
            installed_version,
            has_update,
            missing_dependencies: missing,
        });
    }
    plugins.sort_by(|a, b| a.index.id.cmp(&b.index.id));
    PluginMarketSnapshot {
        plugins,
        offline_mode,
        source: source.to_string(),
        warning,
    }
}

fn pick_latest_version(item: &PluginIndexEntry) -> Option<PluginIndexVersionEntry> {
    // 优先使用 versions；fallback 到 entry.version（兼容旧索引）
    if item.versions.is_empty() {
        return None;
    }
    let mut best: Option<(Version, PluginIndexVersionEntry)> = None;
    for v in &item.versions {
        let parsed = crate::infrastructure::directory_plugins::parse_manifest_version(&v.version)
            .or_else(|| Version::parse(v.version.trim()).ok());
        let Some(p) = parsed else {
            continue;
        };
        match best.as_ref() {
            None => best = Some((p, v.clone())),
            Some((bp, _)) => {
                if p > *bp {
                    best = Some((p, v.clone()));
                }
            }
        }
    }
    best.map(|(_, x)| x)
}

#[tauri::command]
pub fn sync_plugin_index_command(
    index_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<PluginMarketSnapshot, String> {
    let dev = state.directory_plugins.host().developer_effective();
    let requested = index_url.as_deref().map(str::trim).unwrap_or("");
    let wants_custom = !requested.is_empty() && requested != "official";
    if wants_custom && !dev {
        // 非开发者模式：禁止自定义源（保持官方默认体验基石）
        match sync_plugin_index_online(&state, None) {
            Ok(index) => Ok(build_snapshot(
                &state,
                index,
                false,
                "official",
                Some("已忽略自定义索引源：仅开发者模式可使用第三方源。".to_string()),
            )),
            Err(err) => {
                let cache = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
                Ok(build_snapshot(
                    &state,
                    cache,
                    true,
                    "official-cache",
                    Some(format!("在线索引不可达，已回退本地缓存：{}", err)),
                ))
            }
        }
    } else if wants_custom {
        match sync_plugin_index_online_for_source(&state, requested) {
            Ok(index) => Ok(build_snapshot(&state, index, false, requested, None)),
            Err(err) => {
                let cache =
                    load_cached_index_for_source(&state, requested).map_err(|e| e.to_frontend_error())?;
                Ok(build_snapshot(
                    &state,
                    cache,
                    true,
                    requested,
                    Some(format!("在线索引不可达，已回退本地缓存：{}", err)),
                ))
            }
        }
    } else {
        match sync_plugin_index_online(&state, None) {
            Ok(index) => Ok(build_snapshot(&state, index, false, "official", None)),
            Err(err) => {
                let cache = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
                Ok(build_snapshot(
                    &state,
                    cache,
                    true,
                    "official-cache",
                    Some(format!("在线索引不可达，已回退本地缓存：{}", err)),
                ))
            }
        }
    }
}

#[tauri::command]
pub fn get_cached_plugin_index(state: State<'_, AppState>) -> Result<PluginMarketSnapshot, String> {
    let index = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
    Ok(build_snapshot(&state, index, true, "official-cache", None))
}

#[tauri::command]
pub fn install_plugin_from_market(
    plugin_id: String,
    git_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<InstallPluginFromMarketResponse, String> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id required".to_string());
    }
    let from_index = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
    let index_item = from_index.plugins.iter().find(|p| p.id == pid).cloned();
    // 默认：git clone + checkout tag（以 versions.latest.git_tag 或 version 作为 tag）
    // 若调用方传了 git_url，则仍按旧逻辑直接 clone（开发者模式可用）
    let installed_id = if let Some(g) = git_url.as_deref().map(str::trim).filter(|s| !s.is_empty())
    {
        install_plugin(&state, g, index_item.as_ref().map(|x| &x.dependencies))
            .map_err(|e| e.to_frontend_error())?
    } else {
        let idx = index_item
            .as_ref()
            .ok_or_else(|| format!("plugin not found in index: {}", pid))?;
        let latest = pick_latest_version(idx)
            .or_else(|| Some(PluginIndexVersionEntry {
                version: idx.version.clone(),
                download_url: None,
                signature_url: None,
                git_tag: None,
            }))
            .ok_or_else(|| format!("plugin version not found in index: {}", pid))?;
        let tag = latest
            .git_tag
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(latest.version.trim());
        install_plugin_from_git_tag(&state, &idx.git, tag, Some(&idx.dependencies))
            .map_err(|e| e.to_frontend_error())?
    };
    let root_opt = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots.get(installed_id.as_str()).cloned()
    };
    if let Some(root) = root_opt {
        if let Ok(m) = OclivePluginManifest::load_from_dir(&root) {
            ensure_default_config_for_manifest(&state, &m);
        }
    }
    Ok(InstallPluginFromMarketResponse {
        installed_plugin_id: installed_id,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPluginVersionFromMarketRequest {
    pub plugin_id: String,
    pub version: String,
}

/// 回滚/指定版本安装：从索引读取 download_url + signature_url → 验签 → 安装
#[tauri::command]
pub fn install_plugin_version_from_market(
    req: InstallPluginVersionFromMarketRequest,
    state: State<'_, AppState>,
) -> Result<InstallPluginFromMarketResponse, String> {
    let pid = req.plugin_id.trim();
    if pid.is_empty() {
        return Err("plugin_id required".to_string());
    }
    let want = req.version.trim();
    if want.is_empty() {
        return Err("version required".to_string());
    }
    let from_index = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
    let index_item = from_index
        .plugins
        .iter()
        .find(|p| p.id == pid)
        .cloned()
        .ok_or_else(|| format!("plugin not found in index: {}", pid))?;
    let picked = index_item
        .versions
        .iter()
        .find(|v| v.version.trim() == want)
        .cloned()
        .ok_or_else(|| format!("version not found in index: {} {}", pid, want))?;
    let download_url = picked
        .download_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("download_url missing in index: {} {}", pid, want))?;
    let signature_url = picked
        .signature_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("signature_url missing in index: {} {}", pid, want))?;
    let installed_id = install_plugin_from_download_urls(
        &state,
        &index_item,
        download_url,
        signature_url,
    )
    .map_err(|e| e.to_frontend_error())?;
    Ok(InstallPluginFromMarketResponse {
        installed_plugin_id: installed_id,
    })
}

#[tauri::command]
pub fn install_plugin_from_git(
    req: InstallPluginFromGitRequest,
    state: State<'_, AppState>,
) -> Result<InstallPluginFromMarketResponse, String> {
    let git = req.git_url.trim();
    if git.is_empty() {
        return Err("git_url required".to_string());
    }
    let installed_id = install_plugin(&state, git, None).map_err(|e| e.to_frontend_error())?;
    let root_opt = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots.get(installed_id.as_str()).cloned()
    };
    if let Some(root) = root_opt {
        if let Ok(m) = OclivePluginManifest::load_from_dir(&root) {
            ensure_default_config_for_manifest(&state, &m);
        }
    }
    Ok(InstallPluginFromMarketResponse {
        installed_plugin_id: installed_id,
    })
}

#[tauri::command]
pub fn update_plugin_from_market(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    update_plugin(&state, &plugin_id).map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub fn uninstall_plugin_from_market(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    uninstall_plugin(&state, &plugin_id).map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub fn batch_update_plugins(
    plugin_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    for pid in plugin_ids {
        let t = pid.trim();
        if t.is_empty() {
            continue;
        }
        update_plugin(&state, t).map_err(|e| e.to_frontend_error())?;
    }
    Ok(())
}

#[tauri::command]
pub fn batch_uninstall_plugins(
    plugin_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    for pid in plugin_ids {
        let t = pid.trim();
        if t.is_empty() {
            continue;
        }
        uninstall_plugin(&state, t).map_err(|e| e.to_frontend_error())?;
    }
    Ok(())
}

#[tauri::command]
pub fn consume_pending_protocol_installs() -> Vec<PendingProtocolInstall> {
    take_pending_install_git_urls()
        .into_iter()
        .map(|git_url| PendingProtocolInstall { git_url })
        .collect()
}
