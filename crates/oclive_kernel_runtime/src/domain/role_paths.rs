//! 角色包资源路径解析（供前端 `convertFileSrc` / `readBinaryFile`）。

use crate::state::KernelAppState;

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
