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
    crate::infrastructure::role_deletion::recover_pending_deletions(
        &state.db_manager,
        state.storage.roles_dir(),
        Some(&state.directory_plugins),
    )
    .await?;
    let _guard = crate::infrastructure::role_files::lock_role_writes(state.storage.roles_dir())?;
    crate::infrastructure::role_files::recover_imports_locked(state.storage.roles_dir())?;
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
    let (record, removed_ns) = crate::infrastructure::role_deletion::delete_files_and_data(
        &state.db_manager,
        state.storage.roles_dir(),
        rid,
        &mirror_root.join(rid),
    )
    .await?;
    for ns in &removed_ns {
        state.clear_all_session_slot_overrides(ns);
    }
    state.role_cache.write().remove(rid);
    state.invalidate_personality_cache_for_role(rid);
    let cleanup_pending = match crate::infrastructure::role_deletion::finish_deletion(
        &state.db_manager,
        &record,
        &state.directory_plugins,
    )
    .await
    {
        Ok(()) => false,
        Err(error) => {
            tracing::warn!(target: "oclive_role_files", role_id = rid, %error,
                "role deleted; cleanup deferred until retry/restart");
            true
        }
    };
    Ok(json!({ "ok": true, "role_id": rid, "cleanup_pending": cleanup_pending }))
}
