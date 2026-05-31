//! Resolve the "current user identity key": shared by `load_role` / `get_role_info` / the chat engine to avoid forking scene logic.

use crate::error::Result;
use crate::models::role::{IdentityBinding, Role};
use crate::state::AppState;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Effective identity key consistent with [`crate::api::role::runtime::role_runtime_extras`] and the chat turn.
pub async fn resolve_effective_user_relation_key(
    state: &AppState,
    role: &Role,
    role_id: &str,
    scene_id: Option<&str>,
) -> Result<String> {
    let scene_override = if matches!(role.identity_binding, IdentityBinding::PerScene) {
        if let Some(sid) = scene_id {
            state
                .db_manager
                .get_user_relation_for_scene(role_id, sid)
                .await?
        } else {
            None
        }
    } else {
        None
    };

    if let Some(sr) = scene_override {
        return Ok(sr);
    }
    if state.db_manager.get_use_manifest_default(role_id).await? {
        return Ok(role.default_relation.clone());
    }
    Ok(state
        .db_manager
        .get_user_relation(role_id)
        .await?
        .unwrap_or_else(|| role.default_relation.clone()))
}
