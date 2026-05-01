use crate::state::AppState;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMcpServerImportRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMcpServerRequest {
    pub path: String,
    /// When true, write the required permission grant.
    #[serde(default)]
    pub grant_required_permission: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerImportPreview {
    pub server_id: String,
    pub name: String,
    pub transport: String,
    pub required_permission: String,
}

fn read_mcp_manifest_from_path(path: &PathBuf) -> Result<crate::infrastructure::mcp_client::McpServerManifest, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read file failed: {}", e))?;
    let m = serde_json::from_str::<crate::infrastructure::mcp_client::McpServerManifest>(&raw)
        .map_err(|e| format!("invalid mcp server json: {}", e))?;
    Ok(m)
}

#[tauri::command]
pub fn preview_mcp_server_import(
    req: PreviewMcpServerImportRequest,
    state: State<'_, AppState>,
) -> Result<McpServerImportPreview, String> {
    let path = PathBuf::from(req.path.trim());
    if !path.is_file() {
        return Err(format!("file not found: {}", path.display()));
    }
    let mut m = read_mcp_manifest_from_path(&path)?;
    m.id = m.id.trim().to_string();
    if m.id.is_empty() {
        return Err("mcp server id required".to_string());
    }
    if m.transport.trim().is_empty() {
        m.transport = "http".to_string();
    }
    let required_permission = if m.transport.trim().eq_ignore_ascii_case("stdio") {
        "process:spawn".to_string()
    } else {
        "network:*".to_string()
    };
    let _ = state; // keep signature stable for future policy checks
    Ok(McpServerImportPreview {
        server_id: m.id,
        name: m.name,
        transport: m.transport,
        required_permission,
    })
}

#[tauri::command]
pub async fn import_mcp_server_from_path(
    req: ImportMcpServerRequest,
    state: State<'_, AppState>,
) -> Result<McpServerImportPreview, String> {
    let path = PathBuf::from(req.path.trim());
    if !path.is_file() {
        return Err(format!("file not found: {}", path.display()));
    }
    let mut m = read_mcp_manifest_from_path(&path)?;
    m.id = m.id.trim().to_string();
    if m.id.is_empty() {
        return Err("mcp server id required".to_string());
    }
    if m.transport.trim().is_empty() {
        m.transport = "http".to_string();
    }
    let required_permission = if m.transport.trim().eq_ignore_ascii_case("stdio") {
        "process:spawn".to_string()
    } else {
        "network:*".to_string()
    };
    let dir = state.directory_plugins.app_data_dir().join("mcp-servers");
    let _ = fs::create_dir_all(&dir);
    let target = dir.join(format!("{}.json", m.id));
    let raw = fs::read_to_string(&path).map_err(|e| format!("read file failed: {}", e))?;
    fs::write(&target, raw).map_err(|e| format!("write file failed: {}", e))?;

    if req.grant_required_permission {
        let provider_id = format!("system:mcp_server:{}", m.id);
        let _ = state
            .db_manager
            .upsert_plugin_permission_grant(provider_id.as_str(), required_permission.as_str(), true)
            .await;
    }

    Ok(McpServerImportPreview {
        server_id: m.id,
        name: m.name,
        transport: m.transport,
        required_permission,
    })
}
