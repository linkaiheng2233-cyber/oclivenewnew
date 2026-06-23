//! Theater HTTP routes (`POST /theater/scene`).

use super::{api_error, kernel_http_error, ApiError};
use crate::domain::theater::generate_scene;
use crate::error::http_chat_codes;
use crate::models::dto::{TheaterSceneRequest, TheaterSceneResponse};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;

pub async fn scene_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TheaterSceneRequest>,
) -> Result<Json<TheaterSceneResponse>, ApiError> {
    match generate_scene(state.as_ref(), &req).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(api_error(
            axum::http::StatusCode::BAD_REQUEST,
            kernel_http_error(
                http_chat_codes::THEATER_SCENE_GEN_FAILED,
                e.to_string(),
                Some("请检查 base_beats / fallback_beats 与角色配置".into()),
            ),
        )),
    }
}
