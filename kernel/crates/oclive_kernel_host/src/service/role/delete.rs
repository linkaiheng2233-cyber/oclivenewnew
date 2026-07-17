//! Delete role pack directory and DB state for a manifest role.

use crate::command_error::CommandError;
use crate::error::AppError;
use crate::state::AppState;
use serde_json::{json, Value};

/// Deletes the local role directory and DB state for that manifest role (including `__sess__` session namespaces).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn delete_role_impl(state: &AppState, role_id: String) -> Result<Value, CommandError> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).into());
    }
    oclive_validation::validate_role_id(rid).map_err(AppError::InvalidParameter)?;
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await?;
    let chat_location = state
        .load_role_cached_async(rid)
        .await
        .ok()
        .map(|r| r.pack_chat_storage_config.location.clone());
    let mirror_root = crate::infrastructure::chat_storage::load_role_chat_storage_root(
        state.directory_plugins.app_data_dir(),
        state.storage.roles_dir(),
        rid,
        chat_location.as_deref(),
    );
    if let Err(e) =
        crate::infrastructure::chat_storage::delete_mirror_tree_for_role(&mirror_root, rid).await
    {
        tracing::warn!(
            target: "oclive_chat_storage",
            role_id = %rid,
            error = %e,
            "delete_mirror_tree_for_role failed"
        );
    }
    for ns in &removed_ns {
        state.clear_all_session_slot_overrides(ns);
    }
    let dir = state.storage.role_dir_path(rid)?;
    if dir.exists() {
        let dir_owned = dir.clone();
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(&dir_owned))
            .await
            .map_err(|e| format!("delete_role: join {e}"))?
            .map_err(|e: std::io::Error| e.to_string())?;
    }
    state.directory_plugins.remove_role_plugin_state(rid)?;
    state.role_cache.write().remove(rid);
    state.invalidate_personality_cache_for_role(rid);
    Ok(json!({ "ok": true, "role_id": rid }))
}
