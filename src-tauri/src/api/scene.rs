use crate::api::role::get_role_info_impl;
use crate::error::AppError;
use crate::models::dto::{
    RoleInfo, SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse,
};
use crate::state::AppState;
use tauri::State;
use crate::api::error::CommandError;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn switch_scene_impl(
    state: &AppState,
    req: &SwitchSceneRequest,
) -> Result<SwitchSceneResponse, CommandError> {
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

    if req.together {
        state
            .db_manager
            .set_current_scene(&req.role_id, &req.scene_id)
            .await
            ?;
    }
    state
        .db_manager
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await
        ?;
    let role = get_role_info_impl(state, &req.role_id, None).await?;
    let scene_welcome = if req.together {
        state
            .storage
            .scene_welcome_line(&req.role_id, &req.scene_id)
    } else {
        None
    };
    Ok(SwitchSceneResponse {
        role,
        scene_welcome,
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn set_user_presence_scene_impl(
    state: &AppState,
    req: &SetUserPresenceSceneRequest,
) -> Result<RoleInfo, CommandError> {
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
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await
        ?;
    get_role_info_impl(state, &req.role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn switch_scene(
    req: SwitchSceneRequest,
    state: State<'_, AppState>,
) -> Result<SwitchSceneResponse, CommandError> {
    switch_scene_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_user_presence_scene(
    req: SetUserPresenceSceneRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, CommandError> {
    set_user_presence_scene_impl(&state, &req).await
}
