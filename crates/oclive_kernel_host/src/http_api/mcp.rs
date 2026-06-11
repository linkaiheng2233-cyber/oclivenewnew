use super::{api_error, ApiError};
use crate::service::{call_mcp_tool_impl, list_mcp_servers_impl, list_mcp_tools_impl, CallMcpToolHttpRequest};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct ListMcpToolsQuery {
    server_id: String,
}

pub(crate) async fn list_mcp_servers_route(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    list_mcp_servers_impl(&state).map(Json).map_err(|e| {
        let k = e.kernel_error_body();
        api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, k)
    })
}

pub(crate) async fn list_mcp_tools_route(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListMcpToolsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if q.server_id.trim().is_empty() {
        let k = super::kernel_http_error("INVALID_PARAMETER", "server_id required", None);
        return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
    }
    list_mcp_tools_impl(&state, q.server_id.trim())
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}

pub(crate) async fn call_mcp_tool_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CallMcpToolHttpRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if req.server_id.trim().is_empty() || req.tool_name.trim().is_empty() {
        let k = super::kernel_http_error(
            "INVALID_PARAMETER",
            "server_id and tool_name required",
            None,
        );
        return Err(api_error(axum::http::StatusCode::BAD_REQUEST, k));
    }
    call_mcp_tool_impl(&state, &req)
        .await
        .map(Json)
        .map_err(|e| {
            let k = e.kernel_error_body();
            api_error(axum::http::StatusCode::BAD_REQUEST, k)
        })
}
