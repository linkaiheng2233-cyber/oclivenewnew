//! Relation / identity API commands.
#![allow(clippy::missing_errors_doc)]

use super::get_role_info_impl;
use crate::error::AppError;
use crate::models::dto::{ClearSceneUserRelationRequest, RoleInfo, SetSceneUserRelationRequest, SetUserRelationRequest, OCLIVE_DEFAULT_RELATION_SENTINEL};
use crate::models::role::IdentityBinding;
use crate::state::AppState;
use tauri::State;
use crate::api::error::CommandError;
pub async fn set_user_relation_impl(
    state: &AppState,
    req: &SetUserRelationRequest,
) -> Result<RoleInfo, CommandError> {
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        ?
    {
        return Err(AppError::RoleRuntimeNotReady.into());
    }
    let role = state
        .load_role_cached_async(&req.role_id)
        .await
        ?;

    if matches!(role.identity_binding, IdentityBinding::Global) {
        state
            .db_manager
            .clear_all_scene_identities_for_role(&req.role_id)
            .await
            ?;
    }

    if req.relation == OCLIVE_DEFAULT_RELATION_SENTINEL {
        state
            .db_manager
            .set_use_manifest_default(&req.role_id, true)
            .await
            ?;
        let eff = role.default_relation.clone();
        let seed = role.initial_favorability_for_relation(eff.as_str());
        state
            .db_manager
            .ensure_identity_stats_row(&req.role_id, &eff, seed)
            .await
            ?;
        state
            .db_manager
            .mirror_runtime_from_identity(&req.role_id, &eff)
            .await
            ?;
        return get_role_info_impl(state, &req.role_id, None).await;
    }

    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(AppError::InvalidParameter(format!(
            "unknown relation: {}",
            req.relation
        ))
        .into());
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await
        ?;
    state
        .db_manager
        .set_user_relation(&req.role_id, &req.relation)
        .await
        ?;
    let seed = role.initial_favorability_for_relation(&req.relation);
    state
        .db_manager
        .ensure_identity_stats_row(&req.role_id, &req.relation, seed)
        .await
        ?;
    state
        .db_manager
        .mirror_runtime_from_identity(&req.role_id, &req.relation)
        .await
        ?;
    get_role_info_impl(state, &req.role_id, None).await
}
pub async fn clear_scene_user_relation_impl(
    state: &AppState,
    req: &ClearSceneUserRelationRequest,
) -> Result<RoleInfo, CommandError> {
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        ?
    {
        return Err(AppError::RoleRuntimeNotReady.into());
    }
    let role = state
        .load_role_cached_async(&req.role_id)
        .await
        ?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "This role pack uses global identity_binding; per-scene identity overrides are not used."
                .to_string(),
        )
        .into());
    }
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        ?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .into());
    }
    state
        .db_manager
        .clear_user_relation_for_scene(&req.role_id, &req.scene_id)
        .await
        ?;
    get_role_info_impl(state, &req.role_id, None).await
}
pub async fn set_scene_user_relation_impl(
    state: &AppState,
    req: &SetSceneUserRelationRequest,
) -> Result<RoleInfo, CommandError> {
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        ?
    {
        return Err(AppError::RoleRuntimeNotReady.into());
    }
    let role = state
        .load_role_cached_async(&req.role_id)
        .await
        ?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "This role pack uses global identity_binding; set identity globally instead of per scene."
                .to_string(),
        )
        .into());
    }
    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(AppError::InvalidParameter(format!(
            "unknown relation: {}",
            req.relation
        ))
        .into());
    }
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        ?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .into());
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await
        ?;
    state
        .db_manager
        .set_user_relation_for_scene(&req.role_id, &req.scene_id, &req.relation)
        .await
        ?;
    get_role_info_impl(state, &req.role_id, None).await
}
#[tauri::command]
pub async fn set_user_relation(
    req: SetUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, CommandError> {
    set_user_relation_impl(&state, &req).await
}
#[tauri::command]
pub async fn set_scene_user_relation(
    req: SetSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, CommandError> {
    set_scene_user_relation_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_scene_user_relation(
    req: ClearSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, CommandError> {
    clear_scene_user_relation_impl(&state, &req).await
}
