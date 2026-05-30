//! Expert routing config read/write (`blueprint/includes/expert_routing.json`).

use crate::api::error::CommandError;
use crate::error::AppError;
use crate::state::AppState;
use oclive_validation::{
    validate_expert_routing_doc, ExpertRoutingDoc, DEFAULT_EXPERT_ROUTING_PATH,
};
use std::fs;
use std::path::PathBuf;
use tauri::State;

fn expert_routing_path(state: &AppState, role_id: &str) -> Result<PathBuf, CommandError> {
    let role_dir = state.storage.roles_dir().join(role_id.trim());
    if !role_dir.is_dir() {
        return Err(AppError::RoleNotFound(format!("角色目录不存在: {role_id}")).into());
    }
    Ok(role_dir.join(DEFAULT_EXPERT_ROUTING_PATH))
}

/// Lists filenames under the role pack `blueprint/includes/` directory (single level only).
///
/// # Errors
///
/// Returns `CommandError` when the directory cannot be read.
#[tauri::command]
pub fn list_blueprint_includes(
    state: State<'_, AppState>,
    role_id: String,
) -> Result<Vec<String>, CommandError> {
    let includes_dir = state
        .storage
        .roles_dir()
        .join(role_id.trim())
        .join("blueprint/includes");
    if !includes_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&includes_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Reads expert routing JSON.
///
/// # Errors
///
/// Returns `CommandError` when the role is missing, the file is unreadable, or JSON is invalid.
#[tauri::command]
pub fn get_expert_routing(
    state: State<'_, AppState>,
    role_id: String,
) -> Result<Option<ExpertRoutingDoc>, CommandError> {
    let path = expert_routing_path(&state, &role_id)?;
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let doc: ExpertRoutingDoc = serde_json::from_str(&raw).map_err(|e| {
        AppError::InvalidParameter(format!("expert_routing.json: {e}"))
    })?;
    Ok(Some(doc))
}

/// Saves expert routing JSON (creates `blueprint/includes/` automatically).
///
/// # Errors
///
/// Returns `CommandError` on validation failure, directory creation failure, or write failure.
#[tauri::command]
pub fn save_expert_routing(
    state: State<'_, AppState>,
    role_id: String,
    doc: ExpertRoutingDoc,
) -> Result<(), CommandError> {
    let path = expert_routing_path(&state, &role_id)?;
    validate_expert_routing_doc(&doc).map_err(|errs| {
        AppError::InvalidParameter(format!(
            "expert_routing.json 校验失败:\n{}",
            errs.join("\n")
        ))
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(&doc).map_err(AppError::SerializationError)?;
    fs::write(&path, raw)?;
    Ok(())
}
