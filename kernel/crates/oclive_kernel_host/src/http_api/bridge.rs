use super::{api_error, kernel_http_error, ApiError};
use crate::service::{
    dispatch_bridge_command, grant_high_risk_capability_impl, list_high_risk_grants_impl,
    revoke_high_risk_capability_impl, MutateHighRiskGrantRequest,
};
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use oclive_kernel_types::KernelErrorBody;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct BridgeDispatchRequest {
    command: String,
    #[serde(default)]
    params: serde_json::Value,
}

const BRIDGE_TOKEN_HEADER: &str = "x-oclive-bridge-token";

fn bridge_dispatch_authorized(headers: &HeaderMap) -> bool {
    let Some(expected) = std::env::var("OCLIVE_BRIDGE_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
    else {
        return true;
    };
    headers
        .get(BRIDGE_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t == expected)
}

fn bridge_token_unauthorized() -> ApiError {
    let k = kernel_http_error(
        "INVALID_PARAMETER",
        "missing or invalid x-oclive-bridge-token",
        Some("Set OCLIVE_BRIDGE_TOKEN on kernel and pass the same value in the header.".into()),
    );
    api_error(axum::http::StatusCode::UNAUTHORIZED, k)
}

pub(crate) async fn list_high_risk_grants_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !bridge_dispatch_authorized(&headers) {
        return Err(bridge_token_unauthorized());
    }
    list_high_risk_grants_impl(&state).map(Json).map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
    })
}

pub(crate) async fn grant_high_risk_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MutateHighRiskGrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !bridge_dispatch_authorized(&headers) {
        return Err(bridge_token_unauthorized());
    }
    grant_high_risk_capability_impl(&state, &req)
        .map(|_| Json(serde_json::json!({ "ok": true })))
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn revoke_high_risk_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MutateHighRiskGrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !bridge_dispatch_authorized(&headers) {
        return Err(bridge_token_unauthorized());
    }
    revoke_high_risk_capability_impl(&state, &req)
        .map(|_| Json(serde_json::json!({ "ok": true })))
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn bridge_dispatch_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BridgeDispatchRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !bridge_dispatch_authorized(&headers) {
        return Err(bridge_token_unauthorized());
    }
    let cmd = req.command.trim();
    if cmd.is_empty() {
        let k = KernelErrorBody {
            code: "INVALID_PARAMETER".to_string(),
            message: "bridge dispatch: command required".into(),
            hint: None,
        };
        return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
    }
    dispatch_bridge_command(state.as_ref(), cmd, req.params)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}
