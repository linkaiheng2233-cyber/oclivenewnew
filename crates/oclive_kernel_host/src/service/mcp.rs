//! MCP HTTP mirror for VS Code extension (see handoff/VSCODE_MCP_HTTP_GATE.md).

use crate::command_error::CommandError;
use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;

/// # Errors
///
/// JSON serialization failure.
pub fn list_mcp_servers_impl(state: &AppState) -> Result<Value, CommandError> {
    Ok(serde_json::to_value(state.plugins.list_mcp_servers())?)
}

/// # Errors
///
/// MCP list tools failure.
pub async fn list_mcp_tools_impl(state: &AppState, server_id: &str) -> Result<Value, CommandError> {
    let tools = state
        .plugins
        .list_mcp_tools(server_id)
        .await
        .map_err(CommandError::from)?;
    Ok(serde_json::to_value(tools)?)
}

#[derive(Debug, Deserialize)]
pub struct CallMcpToolHttpRequest {
    pub server_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub params: Value,
}

/// # Errors
///
/// MCP call failure.
pub async fn call_mcp_tool_impl(
    state: &AppState,
    req: &CallMcpToolHttpRequest,
) -> Result<Value, CommandError> {
    let result = state
        .plugins
        .call_mcp_tool(
            req.server_id.as_str(),
            req.tool_name.as_str(),
            req.params.clone(),
        )
        .await
        .map_err(CommandError::from)?;
    Ok(serde_json::to_value(result)?)
}
