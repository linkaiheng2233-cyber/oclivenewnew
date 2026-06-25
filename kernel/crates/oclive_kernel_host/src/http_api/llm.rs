use super::{api_error, ApiError};
use crate::models::dto::RoleInfo;
use crate::service::{
    get_global_ollama_model_impl, get_llm_user_settings_impl, list_cloud_models_impl,
    list_ollama_models_impl, probe_cloud_llm_impl, save_llm_user_settings_impl,
    set_global_ollama_model_impl, set_session_llm_model_impl, GlobalOllamaModelDto,
    ListCloudModelsRequest, LlmUserSettingsDto, SaveLlmUserSettingsRequest,
    SetGlobalOllamaModelRequest, SetSessionLlmModelRequest,
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

#[derive(Serialize)]
pub(crate) struct LlmReloadResponse {
    ok: bool,
    provider: String,
}

#[derive(Serialize)]
pub(crate) struct LlmProbeResponse {
    ok: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaModelsQuery {
    ollama_base_url: Option<String>,
}

pub(crate) async fn llm_reload_route(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LlmReloadResponse>, ApiError> {
    state.mark_user_llm_env_dirty();
    crate::domain::user_llm_env::apply_user_llm_env(state.as_ref())
        .await
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })?;
    let provider = state.user_llm_provider.read().clone();
    Ok(Json(LlmReloadResponse { ok: true, provider }))
}

pub(crate) async fn llm_user_settings_get_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<LlmUserSettingsDto>, ApiError> {
    get_llm_user_settings_impl(&state, q.role_id.trim(), q.session_id.as_deref())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn llm_user_settings_post_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveLlmUserSettingsRequest>,
) -> Result<Json<RoleInfo>, ApiError> {
    let info = save_llm_user_settings_impl(state.as_ref(), &req)
        .await
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })?;
    Ok(Json(info))
}

pub(crate) async fn llm_probe_cloud_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleIdQuery>,
) -> Result<Json<LlmProbeResponse>, ApiError> {
    probe_cloud_llm_impl(&state, q.role_id.trim(), q.session_id.as_deref())
        .await
        .map(|_| Json(LlmProbeResponse { ok: true }))
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn llm_ollama_models_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<OllamaModelsQuery>,
) -> Result<Json<Vec<String>>, ApiError> {
    list_ollama_models_impl(state.as_ref(), q.ollama_base_url.as_deref())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn llm_cloud_models_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ListCloudModelsRequest>,
) -> Result<Json<Vec<String>>, ApiError> {
    list_cloud_models_impl(
        state.as_ref(),
        req.remote_url.as_deref(),
        req.remote_token.as_deref(),
    )
    .await
    .map(Json)
    .map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::BAD_REQUEST, k)
    })
}

pub(crate) async fn llm_session_model_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetSessionLlmModelRequest>,
) -> Result<Json<RoleInfo>, ApiError> {
    set_session_llm_model_impl(state.as_ref(), &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn llm_global_ollama_model_get_route(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GlobalOllamaModelDto>, ApiError> {
    get_global_ollama_model_impl(state.as_ref())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
        })
}

pub(crate) async fn llm_global_ollama_model_post_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetGlobalOllamaModelRequest>,
) -> Result<Json<GlobalOllamaModelDto>, ApiError> {
    set_global_ollama_model_impl(state.as_ref(), &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}
