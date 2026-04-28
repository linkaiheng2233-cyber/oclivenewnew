use crate::infrastructure::local_imports::{list_local_import_candidates, read_import_text};
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
    let root = root
        .canonicalize()
        .unwrap_or_else(|_| root.clone());
    if !p.starts_with(&root) {
        return Err("path must be under app_data/imports".to_string());
    }
    const MAX_BYTES: usize = 1024 * 1024;
    let content = read_import_text(&p, MAX_BYTES)?;
    Ok(ReadLocalImportTextResponse { content })
}

