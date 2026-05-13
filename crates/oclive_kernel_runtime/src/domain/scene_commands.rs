//! 场景切换与用户在场场景（无 Tauri 依赖）。

use crate::domain::role_info_snapshot::get_role_info_snapshot;
use crate::error::{AppError, Result};
use crate::models::dto::{
    RoleInfo, SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse,
};
use crate::state::KernelAppState;

pub async fn switch_scene(
    state: &KernelAppState,
    req: &SwitchSceneRequest,
) -> Result<SwitchSceneResponse> {
    let scenes = state.storage.list_scene_ids(&req.role_id)?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        )));
    }

    if req.together {
        state
            .db_manager
            .set_current_scene(&req.role_id, &req.scene_id)
            .await?;
    }
    state
        .db_manager
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await?;
    let role = get_role_info_snapshot(state, &req.role_id, None).await?;
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

pub async fn set_user_presence_scene(
    state: &KernelAppState,
    req: &SetUserPresenceSceneRequest,
) -> Result<RoleInfo> {
    let scenes = state.storage.list_scene_ids(&req.role_id)?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        )));
    }
    state
        .db_manager
        .set_user_presence_scene(&req.role_id, &req.scene_id)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}
