use crate::infrastructure::directory_plugins::OclivePluginManifest;
use crate::infrastructure::local_imports::{list_local_import_candidates, read_import_text};
use crate::infrastructure::plugin_installer::{
    install_plugin_from_archive_bytes_overwrite, load_cached_index,
    peek_plugin_id_from_archive_bytes, verify_plugin_package_signature_text,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLocalImportCandidatesResponse {
    pub items: Vec<crate::infrastructure::local_imports::LocalImportCandidate>,
    pub root_dir: String,
}

#[tauri::command]
pub fn list_local_import_candidates_command(
    state: State<'_, AppState>,
) -> Result<ListLocalImportCandidatesResponse, String> {
    let items = list_local_import_candidates(&state)?;
    let root = crate::infrastructure::local_imports::imports_root(&state);
    Ok(ListLocalImportCandidatesResponse {
        items,
        root_dir: root.to_string_lossy().to_string(),
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadLocalImportTextRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadLocalImportTextResponse {
    pub content: String,
}

#[tauri::command]
pub fn read_local_import_text_command(
    req: ReadLocalImportTextRequest,
    state: State<'_, AppState>,
) -> Result<ReadLocalImportTextResponse, String> {
    // enforce path under app_data/imports
    let root = crate::infrastructure::local_imports::imports_root(&state);
    let p = PathBuf::from(req.path.trim());
    let p = p
        .canonicalize()
        .map_err(|e| format!("path canonicalize: {}", e))?;
    let root = root.canonicalize().unwrap_or_else(|_| root.clone());
    if !p.starts_with(&root) {
        return Err("path must be under app_data/imports".to_string());
    }
    const MAX_BYTES: usize = 1024 * 1024;
    let content = read_import_text(&p, MAX_BYTES)?;
    Ok(ReadLocalImportTextResponse { content })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLocalPluginArchiveRequest {
    pub archive_path: String,
    #[serde(default)]
    pub signature_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLocalPluginArchiveResponse {
    pub plugin_id: String,
    pub declared_permissions: Vec<String>,
    pub signature_verified: bool,
    #[serde(default)]
    pub signature_message: Option<String>,
}

fn bridge_permissions_from_manifest(manifest: &OclivePluginManifest) -> Vec<String> {
    // keep consistent with plugin_update.rs logic (invoke mapping)
    let mut out: Vec<String> = Vec::new();
    if let Some(sh) = &manifest.shell {
        if let Some(b) = &sh.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
        }
    }
    for us in &manifest.ui_slots {
        if let Some(b) = &us.bridge {
            for x in &b.invoke {
                let t = x.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[tauri::command]
pub fn preview_local_plugin_archive_command(
    req: PreviewLocalPluginArchiveRequest,
    state: State<'_, AppState>,
) -> Result<PreviewLocalPluginArchiveResponse, String> {
    let root = crate::infrastructure::local_imports::imports_root(&state);
    let archive = PathBuf::from(req.archive_path.trim())
        .canonicalize()
        .map_err(|e| format!("archive canonicalize: {}", e))?;
    let root = root.canonicalize().unwrap_or(root);
    if !archive.starts_with(&root) {
        return Err("archive path must be under app_data/imports".to_string());
    }
    let bytes = std::fs::read(&archive).map_err(|e| format!("read archive failed: {}", e))?;
    let pid =
        peek_plugin_id_from_archive_bytes(&state, &bytes).map_err(|e| e.to_frontend_error())?;

    // declared perms (from manifest inside archive)
    let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
    crate::infrastructure::plugin_installer::extract_oclive_plugin_archive(&bytes, tmp.path())
        .map_err(|e| e.to_frontend_error())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path()).map_err(|e| e.to_string())?;
    let declared_permissions = bridge_permissions_from_manifest(&manifest);

    // verify signature (optional)
    let mut signature_verified = false;
    let signature_message: Option<String>;
    if let Some(sigp) = req
        .signature_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let sig_path = PathBuf::from(sigp)
            .canonicalize()
            .map_err(|e| format!("signature canonicalize: {}", e))?;
        if !sig_path.starts_with(&root) {
            return Err("signature path must be under app_data/imports".to_string());
        }
        let sig_text = std::fs::read_to_string(&sig_path)
            .map_err(|e| format!("read signature failed: {}", e))?;
        let idx = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
        let Some(entry) = idx.plugins.iter().find(|x| x.id.trim() == pid.trim()) else {
            signature_message =
                Some("未在本地缓存索引中找到该插件条目，无法验签（建议先同步官方索引）".into());
            return Ok(PreviewLocalPluginArchiveResponse {
                plugin_id: pid,
                declared_permissions,
                signature_verified,
                signature_message,
            });
        };
        signature_message = match verify_plugin_package_signature_text(entry, &sig_text, &bytes) {
            Ok(_) => {
                signature_verified = true;
                Some("签名校验通过".into())
            }
            Err(e) => Some(e.to_frontend_error()),
        };
    } else {
        signature_message = Some("未提供签名文件（仅开发者模式侧载建议使用）".into());
    }

    Ok(PreviewLocalPluginArchiveResponse {
        plugin_id: pid,
        declared_permissions,
        signature_verified,
        signature_message,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallLocalPluginArchiveRequest {
    pub archive_path: String,
    #[serde(default)]
    pub signature_path: Option<String>,
    pub overwrite: bool,
    #[serde(default)]
    pub accepted_permissions: Option<Vec<String>>,
}

#[tauri::command]
pub async fn install_local_plugin_archive_command(
    req: InstallLocalPluginArchiveRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !state.directory_plugins.host().developer_effective() {
        return Err("developer mode required for local archive install".to_string());
    }
    let root = crate::infrastructure::local_imports::imports_root(&state);
    let archive = PathBuf::from(req.archive_path.trim())
        .canonicalize()
        .map_err(|e| format!("archive canonicalize: {}", e))?;
    let root = root.canonicalize().unwrap_or(root);
    if !archive.starts_with(&root) {
        return Err("archive path must be under app_data/imports".to_string());
    }
    let bytes = std::fs::read(&archive).map_err(|e| format!("read archive failed: {}", e))?;
    let pid =
        peek_plugin_id_from_archive_bytes(&state, &bytes).map_err(|e| e.to_frontend_error())?;

    // if signature provided, enforce verification before install
    if let Some(sigp) = req
        .signature_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let sig_path = PathBuf::from(sigp)
            .canonicalize()
            .map_err(|e| format!("signature canonicalize: {}", e))?;
        if !sig_path.starts_with(&root) {
            return Err("signature path must be under app_data/imports".to_string());
        }
        let sig_text = std::fs::read_to_string(&sig_path)
            .map_err(|e| format!("read signature failed: {}", e))?;
        let idx = load_cached_index(&state).map_err(|e| e.to_frontend_error())?;
        let entry = idx
            .plugins
            .iter()
            .find(|x| x.id.trim() == pid.trim())
            .ok_or_else(|| {
                "未在本地缓存索引中找到该插件条目，无法验签（建议先同步官方索引）".to_string()
            })?;
        verify_plugin_package_signature_text(entry, &sig_text, &bytes)
            .map_err(|e| e.to_frontend_error())?;
    }

    let installed_pid = install_plugin_from_archive_bytes_overwrite(&state, &bytes, req.overwrite)
        .map_err(|e| e.to_frontend_error())?;

    // write grants (accepted must be subset of declared)
    let root_dir = state
        .directory_plugins
        .app_data_dir()
        .join("plugins")
        .join(installed_pid.as_str());
    let manifest = OclivePluginManifest::load_from_dir(&root_dir).map_err(|e| e.to_string())?;
    let declared = bridge_permissions_from_manifest(&manifest);
    let mut perms = req.accepted_permissions.unwrap_or_else(|| declared.clone());
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
    for p in perms {
        let _ = state
            .db_manager
            .upsert_plugin_permission_grant(installed_pid.as_str(), p.as_str(), true)
            .await;
    }
    Ok(installed_pid)
}
