//! Persist `config.json` → `chat_storage` for a role pack.

use crate::error::{AppError, Result};
use crate::models::RolePackChatStorageConfig;
use std::path::{Path, PathBuf};

/// Merge `chat_storage` into `{roles_dir}/{role_id}/config.json`.
///
/// # Errors
///
/// IO / JSON errors propagate.
pub fn save_role_chat_storage_config(
    roles_dir: &Path,
    role_id: &str,
    config: &RolePackChatStorageConfig,
) -> Result<()> {
    let rid = role_id.trim();
    oclive_validation::validate_role_id(rid).map_err(AppError::InvalidParameter)?;
    let config_path = roles_dir.join(rid).join("config.json");
    if !config_path.is_file() {
        return Err(AppError::RoleNotFound(format!(
            "config.json missing for role: {rid}"
        )));
    }
    let raw = std::fs::read_to_string(&config_path).map_err(AppError::IoError)?;
    let mut root: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| AppError::InvalidParameter(e.to_string()))?;
    let chat_storage =
        serde_json::to_value(config).map_err(|e| AppError::InvalidParameter(e.to_string()))?;
    if let Some(obj) = root.as_object_mut() {
        obj.insert("chat_storage".into(), chat_storage);
    } else {
        return Err(AppError::InvalidParameter(
            "config.json root must be object".into(),
        ));
    }
    let pretty = serde_json::to_string_pretty(&root)
        .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
    write_atomic(&config_path, &pretty)?;
    Ok(())
}

fn write_atomic(path: &PathBuf, content: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, content).map_err(AppError::IoError)?;
    std::fs::rename(&tmp, path).map_err(AppError::IoError)?;
    Ok(())
}
