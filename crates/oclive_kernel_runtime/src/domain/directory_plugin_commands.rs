//! 目录插件相关命令（`RoleStorage` + `DirectoryPluginRuntime`）。

use crate::error::{AppError, Result};
use crate::state::KernelAppState;

/// 「重置为角色包推荐」：按当前磁盘角色包的 UI 基线覆盖该角色的 `plugin_state`。
pub fn reset_role_plugin_state_to_pack_default(
    state: &KernelAppState,
    role_id: &str,
) -> Result<()> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter(
            "reset_role_plugin_state: role_id required".into(),
        ));
    }
    let role = state.storage.load_role(rid)?;
    let ui = role.plugin_state_ui_baseline();
    state
        .directory_plugins
        .reset_role_plugin_state_from_ui(rid, ui)
        .map_err(AppError::Unknown)
}
