//! Local GGUF model directory discovery and persistence.

use super::user_llm_env::KEY_LOCAL_MODELS_DIR;
use crate::api::error::CommandError;
use crate::state::{
    ensure_models_dir_for_roles, is_managed_legacy_models_path, migrate_and_cleanup_models,
    paths_equal, reconcile_legacy_models_layout, AppState,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelFileDto {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

pub(crate) fn canonical_models_dir(state: &AppState) -> PathBuf {
    ensure_models_dir_for_roles(state.storage.roles_dir())
}

pub(crate) async fn persist_local_models_dir(
    state: &AppState,
    path: &str,
) -> Result<(), CommandError> {
    state
        .db_manager
        .upsert_app_setting(KEY_LOCAL_MODELS_DIR, path.trim())
        .await?;
    Ok(())
}

/// Effective GGUF folder: repo-root `models/` (like `roles/`), migrating legacy app-data paths.
pub(crate) async fn local_models_dir_for_state(state: &AppState) -> Result<String, CommandError> {
    let canonical = canonical_models_dir(state);
    let canonical_str = canonical.to_string_lossy().into_owned();
    let app_data = state.directory_plugins.app_data_dir().to_path_buf();
    reconcile_legacy_models_layout(&canonical, &app_data);

    if let Ok(Some(v)) = state.db_manager.get_app_setting(KEY_LOCAL_MODELS_DIR).await {
        let t = v.trim();
        if !t.is_empty() {
            let stored = PathBuf::from(t);
            if paths_equal(&stored, &canonical) {
                return Ok(canonical_str);
            }
            if is_managed_legacy_models_path(&stored, &canonical, &app_data) {
                migrate_and_cleanup_models(&stored, &canonical);
                persist_local_models_dir(state, &canonical_str).await?;
                return Ok(canonical_str);
            }
            return Ok(t.to_string());
        }
    }

    persist_local_models_dir(state, &canonical_str).await?;
    Ok(canonical_str)
}

pub(crate) fn scan_local_model_files_in(dir: &Path) -> Vec<LocalModelFileDto> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "gguf" && ext != "bin" {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("model")
            .to_string();
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(LocalModelFileDto {
            name,
            path: path.to_string_lossy().into_owned(),
            size_bytes,
        });
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

pub(crate) fn model_name_from_gguf_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("local-model")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
