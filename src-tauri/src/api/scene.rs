use crate::models::dto::{
    RoleInfo, SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse,
};
use crate::state::AppState;
use tauri::State;

pub async fn switch_scene_impl(
    state: &AppState,
    req: &SwitchSceneRequest,
) -> Result<SwitchSceneResponse, String> {
    oclive_kernel_runtime::domain::scene_commands::switch_scene(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn set_user_presence_scene_impl(
    state: &AppState,
    req: &SetUserPresenceSceneRequest,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::scene_commands::set_user_presence_scene(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
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
