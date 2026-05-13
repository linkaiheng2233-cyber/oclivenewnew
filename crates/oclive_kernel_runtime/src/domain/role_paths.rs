//! 角色包资源路径解析（供前端 `convertFileSrc` / `readBinaryFile`、打开资源管理器等）。

use crate::error::{AppError, Result};
use crate::state::KernelAppState;
use std::path::PathBuf;

/// 去掉 Windows 冗长路径前缀 `\\?\`，避免前端路径异常。
pub fn path_string_for_frontend(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    const VERBATIM: &str = "\\\\?\\";
    if let Some(stripped) = s.strip_prefix(VERBATIM) {
        stripped.to_string()
    } else {
        s.into_owned()
    }
}

/// 解析 `roles/{role_id}/{relative}`；仅当路径为已存在文件时返回字符串。
#[must_use]
pub fn resolve_role_asset_path(
    state: &KernelAppState,
    role_id: &str,
    relative: &str,
) -> Option<String> {
    let p = state.storage.role_asset_path(role_id, relative);
    if p.is_file() {
        return Some(path_string_for_frontend(&p));
    }
    None
}

/// `roles/{role_id}` 目录；用于在文件管理器中打开角色包根目录。
pub fn role_pack_root_dir(state: &KernelAppState, role_id: &str) -> Result<PathBuf> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id is empty".into()));
    }
    let dir = state.storage.roles_dir().join(rid);
    if !dir.is_dir() {
        return Err(AppError::InvalidParameter(format!(
            "role pack folder not found: {}",
            dir.display()
        )));
    }
    Ok(dir)
}
