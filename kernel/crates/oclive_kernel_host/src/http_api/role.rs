use super::{api_error, ApiError};
use crate::models::dto::{
    CreateEventRequest, CreateEventResponse, DisplayMetricsDto, GetRoleInfoRequest,
    GetUserIdentityStateRequest, JumpTimeRequest, JumpTimeResponse, RoleInfo,
    SetRoleInteractionModeRequest, SetSceneUserIdentityRequest, SetUserIdentityRequest,
    SetUserPresenceSceneRequest, SwitchSceneRequest, SwitchSceneResponse, TimeStateResponse,
    UserIdentityStateResponse,
};
use crate::models::role::PersonalitySource;
use crate::service::{
    get_role_info_impl, get_time_state_impl, get_user_identity_state_impl, jump_time_impl,
    load_role_impl, set_role_interaction_mode_impl, set_scene_user_identity_impl,
    set_user_identity_impl, set_user_presence_scene_impl, switch_scene_impl,
};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct RoleIdQuery {
    role_id: String,
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoleSnapshotQuery {
    role_id: String,
    #[serde(default)]
    scene_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoleSnapshotResponse {
    role_id: String,
    current_favorability: f64,
    current_emotion: String,
    portrait_emotion: String,
    relation_state: String,
    display_metrics: Option<DisplayMetricsDto>,
    personality_source: PersonalitySource,
    current_scene: Option<String>,
    user_presence_scene: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoadRoleBody {
    role_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserIdentityStateQuery {
    role_id: String,
    #[serde(default)]
    scene_id: Option<String>,
}

pub(crate) async fn role_info_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<RoleInfo>, ApiError> {
    let req = GetRoleInfoRequest {
        role_id: q.role_id.trim().to_string(),
        session_id: q.session_id,
    };
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn role_snapshot_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleSnapshotQuery>,
) -> Result<Json<RoleSnapshotResponse>, ApiError> {
    let role_id = q.role_id.trim();
    let info = get_role_info_impl(&state, role_id, None)
        .await
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })?;
    let _scene = q.scene_id.as_deref();
    Ok(Json(RoleSnapshotResponse {
        role_id: info.role_id,
        current_favorability: info.current_favorability,
        current_emotion: info.current_emotion.clone(),
        portrait_emotion: info.current_emotion,
        relation_state: info.relation_state,
        display_metrics: info.display_metrics,
        personality_source: info.personality_source,
        current_scene: info.current_scene,
        user_presence_scene: info.user_presence_scene,
    }))
}

pub(crate) async fn load_role_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoadRoleBody>,
) -> Result<axum::http::StatusCode, ApiError> {
    load_role_impl(&state, body.role_id.trim(), false)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn set_role_interaction_mode_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetRoleInteractionModeRequest>,
) -> Result<Json<RoleInfo>, ApiError> {
    set_role_interaction_mode_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn time_state_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<TimeStateResponse>, ApiError> {
    get_time_state_impl(&state, q.role_id.trim())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn jump_time_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JumpTimeRequest>,
) -> Result<Json<JumpTimeResponse>, ApiError> {
    jump_time_impl(&state, &req).await.map(Json).map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::BAD_REQUEST, k)
    })
}

pub(crate) async fn switch_scene_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SwitchSceneRequest>,
) -> Result<Json<SwitchSceneResponse>, ApiError> {
    switch_scene_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn set_user_identity_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetUserIdentityRequest>,
) -> Result<Json<UserIdentityStateResponse>, ApiError> {
    set_user_identity_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn set_scene_user_identity_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetSceneUserIdentityRequest>,
) -> Result<Json<UserIdentityStateResponse>, ApiError> {
    set_scene_user_identity_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn get_user_identity_state_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UserIdentityStateQuery>,
) -> Result<Json<UserIdentityStateResponse>, ApiError> {
    get_user_identity_state_impl(
        &state,
        &GetUserIdentityStateRequest {
            role_id: q.role_id.trim().to_string(),
            scene_id: q.scene_id.filter(|s| !s.trim().is_empty()),
        },
    )
    .await
    .map(Json)
    .map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::BAD_REQUEST, k)
    })
}

pub(crate) async fn set_user_presence_scene_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetUserPresenceSceneRequest>,
) -> Result<Json<RoleInfo>, ApiError> {
    set_user_presence_scene_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn create_event_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEventRequest>,
) -> Result<Json<CreateEventResponse>, ApiError> {
    crate::service::plugin_bridge::create_event_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}
