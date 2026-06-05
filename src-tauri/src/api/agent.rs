use crate::api::error::CommandError;
use crate::state::SharedAppState;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct CallMcpToolRequest {
    pub server_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Deserialize)]
pub struct ListMcpToolsRequest {
    pub server_id: String,
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn list_mcp_servers(state: State<'_, SharedAppState>) -> Result<Value, CommandError> {
    Ok(serde_json::to_value(state.plugins.list_mcp_servers())?)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn list_mcp_tools(
    req: ListMcpToolsRequest,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    let tools = state
        .plugins
        .list_mcp_tools(req.server_id.as_str())
        .await
        .map_err(CommandError::from)?;
    Ok(serde_json::to_value(tools)?)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn call_mcp_tool(
    req: CallMcpToolRequest,
    state: State<'_, SharedAppState>,
) -> Result<Value, CommandError> {
    let result = state
        .plugins
        .call_mcp_tool(req.server_id.as_str(), req.tool_name.as_str(), req.params)
        .await
        .map_err(CommandError::from)?;
    Ok(serde_json::to_value(result)?)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_agent_debug_traces(state: State<'_, SharedAppState>) -> Result<Value, CommandError> {
    Ok(serde_json::to_value(state.plugins.recent_agent_traces())?)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn clear_agent_debug_traces(state: State<'_, SharedAppState>) -> Result<(), CommandError> {
    state.plugins.clear_agent_traces();
    Ok(())
}
