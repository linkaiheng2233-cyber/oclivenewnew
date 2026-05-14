use crate::state::AppState;
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

#[tauri::command]
pub fn list_mcp_servers(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(state.plugins.list_mcp_servers()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_mcp_tools(
    req: ListMcpToolsRequest,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    state
        .plugins
        .list_mcp_tools(req.server_id.as_str())
        .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn call_mcp_tool(req: CallMcpToolRequest, state: State<'_, AppState>) -> Result<Value, String> {
    state
        .plugins
        .call_mcp_tool(req.server_id.as_str(), req.tool_name.as_str(), req.params)
        .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn get_agent_debug_traces(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(state.plugins.recent_agent_traces()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_agent_debug_traces(state: State<'_, AppState>) -> Result<(), String> {
    state.plugins.clear_agent_traces();
    Ok(())
}
