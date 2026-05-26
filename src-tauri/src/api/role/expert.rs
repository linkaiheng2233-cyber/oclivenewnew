//! 专家路由配置读写（`blueprint/includes/expert_routing.json`）。

use crate::api::error::CommandError;
use crate::error::AppError;
use crate::state::AppState;
use oclive_validation::{ExpertRoutingDoc, DEFAULT_EXPERT_ROUTING_PATH};
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

/// 列出角色包 `blueprint/includes/` 下文件名（仅一层）。
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

/// 读取专家路由 JSON。
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

/// 保存专家路由 JSON（自动创建 `blueprint/includes/`）。
#[tauri::command]
pub fn save_expert_routing(
    state: State<'_, AppState>,
    role_id: String,
    doc: ExpertRoutingDoc,
) -> Result<(), CommandError> {
    let path = expert_routing_path(&state, &role_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(&doc).map_err(AppError::SerializationError)?;
    fs::write(&path, raw)?;
    Ok(())
}
