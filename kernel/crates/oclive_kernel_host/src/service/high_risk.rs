//! High-risk capability grants (canonical `{app_data}/high_risk_grants.json`).

use crate::command_error::CommandError;
use crate::infrastructure::high_risk_grants::{normalize_grant_kind, GrantKind};
use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct MutateHighRiskGrantRequest {
    pub kind: String,
    pub id: String,
}

/// # Errors
///
/// JSON serialization failure.
pub fn list_high_risk_grants_impl(state: &AppState) -> Result<Value, CommandError> {
    Ok(serde_json::to_value(state.high_risk_grants.snapshot())?)
}

/// # Errors
///
/// Unknown kind or disk write failure.
pub fn grant_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), CommandError> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state
            .high_risk_grants
            .grant_mcp_http(id)
            .map_err(Into::into),
        Some(GrantKind::McpStdio) => state
            .high_risk_grants
            .grant_mcp_stdio(id)
            .map_err(Into::into),
        Some(GrantKind::ProcessSpawn) => state
            .high_risk_grants
            .grant_process_spawn(id)
            .map_err(Into::into),
        Some(GrantKind::Network) => state.high_risk_grants.grant_network(id).map_err(Into::into),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim()).into()),
    }
}

/// # Errors
///
/// Unknown kind or disk write failure.
pub fn revoke_high_risk_capability_impl(
    state: &AppState,
    req: &MutateHighRiskGrantRequest,
) -> Result<(), CommandError> {
    let id = req.id.as_str();
    match normalize_grant_kind(&req.kind) {
        Some(GrantKind::McpHttp) => state
            .high_risk_grants
            .revoke_mcp_http(id)
            .map_err(Into::into),
        Some(GrantKind::McpStdio) => state
            .high_risk_grants
            .revoke_mcp_stdio(id)
            .map_err(Into::into),
        Some(GrantKind::ProcessSpawn) => state
            .high_risk_grants
            .revoke_process_spawn(id)
            .map_err(Into::into),
        Some(GrantKind::Network) => state
            .high_risk_grants
            .revoke_network(id)
            .map_err(Into::into),
        None => Err(format!("unknown high_risk grant kind: {}", req.kind.trim()).into()),
    }
}
