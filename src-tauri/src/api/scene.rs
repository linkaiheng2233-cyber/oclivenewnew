use crate::api::role::get_role_info_impl;
use crate::error::AppError;
use crate::models::dto::{
    RoleInfo, SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse,
};
use crate::state::AppState;
use tauri::State;

pub async fn switch_scene_impl(
    state: &AppState,
    req: &SwitchSceneRequest,
) -> Result<SwitchSceneResponse, String> {
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .to_frontend_error());
    }

    if req.together {
        state
            .db_manager
            .set_current_scene(&req.role_id, &req.scene_id)
            .await
            .map_err(|e| e.to_frontend_error())?;
    }
    state
        .db_manager
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
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

pub async fn set_user_presence_scene_impl(
    state: &AppState,
    req: &SetUserPresenceSceneRequest,
) -> Result<RoleInfo, String> {
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .to_frontend_error());
    }
    state
        .db_manager
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

#[tauri::command]
pub async fn switch_scene(
    req: SwitchSceneRequest,
    state: State<'_, AppState>,
) -> Result<SwitchSceneResponse, String> {
    switch_scene_impl(&state, &req).await
}

#[tauri::command]
pub async fn set_user_presence_scene(
    req: SetUserPresenceSceneRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_user_presence_scene_impl(&state, &req).await
}
