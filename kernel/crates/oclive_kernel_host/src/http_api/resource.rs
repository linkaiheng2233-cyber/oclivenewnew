use super::{api_error, ApiError};
use crate::service::transition_resource_adapter_impl;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use oclive_kernel_types::{ResourceAdapterTransitionRequest, ResourceAdapterTransitionResponse};
use std::sync::Arc;

pub(crate) async fn resource_adapter_transition_route(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResourceAdapterTransitionRequest>,
) -> Result<Json<ResourceAdapterTransitionResponse>, ApiError> {
    transition_resource_adapter_impl(state.as_ref(), &request)
        .await
        .map(Json)
        .map_err(|error| {
            let status = match error {
                oclive_kernel_types::AppError::InvalidParameter(_) => {
                    axum::http::StatusCode::BAD_REQUEST
                }
                _ => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            };
            api_error(status, error.kernel_error_body())
        })
}
