//! 角色加载与删除（与桌面 `load_role` / `delete_role` 行为一致）。

use crate::domain::role_info_snapshot::build_role_data;
use crate::error::{AppError, Result};
use crate::models::dto::RoleData;
use crate::state::KernelAppState;
use serde_json::{json, Value};
use std::sync::Arc;

/// `reset_portrait_emotion`：为 `true` 时（应用启动 `load_role`）立绘重置为 `neutral`；切换角色时为 `false`。
pub async fn load_role(
    state: &KernelAppState,
    role_id: &str,
    reset_portrait_emotion: bool,
) -> Result<RoleData> {
    let role = state.storage.load_role(role_id)?;
    let role = Arc::new(role);

    state.directory_plugins.set_active_role_id(role_id);
    state
        .directory_plugins
        .ensure_role_plugin_state(role_id, role.plugin_state_ui_baseline());

    state.invalidate_personality_cache_for_role(role_id);

    state.db_manager.ensure_role_runtime(role_id).await?;

    if reset_portrait_emotion {
        state
            .db_manager
            .set_current_emotion(role_id, "neutral")
            .await?;
    }

    let role_data = build_role_data(state, role_id, role.as_ref()).await?;

    state
        .role_cache
        .write()
        .insert(role_id.to_string(), Arc::clone(&role));

    Ok(role_data)
}

pub async fn delete_role(state: &KernelAppState, role_id: String) -> Result<Value> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter(
            "delete_role: role_id required".to_string(),
        ));
    }
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await?;
    for ns in &removed_ns {
        state.clear_session_backend_override(ns);
    }
    let dir = state.storage.roles_dir().join(rid);
    if dir.exists() {
        let dir_owned = dir.clone();
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(&dir_owned))
            .await
            .map_err(|e| AppError::Unknown(format!("delete_role: join {}", e)))?
            .map_err(AppError::from)?;
    }
    state
        .directory_plugins
        .remove_role_plugin_state(rid)
        .map_err(AppError::Unknown)?;
    state.role_cache.write().remove(rid);
    state.invalidate_personality_cache_for_role(rid);
    Ok(json!({ "ok": true, "role_id": rid }))
}
