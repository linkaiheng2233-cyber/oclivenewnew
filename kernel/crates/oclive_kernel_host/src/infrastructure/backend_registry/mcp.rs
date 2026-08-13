//! Split from backend_registry.rs (zero semantic change, facade in mod.rs).

use oclive_kernel_types::{AgentToolResult, McpServerInfo, McpToolInfo};
use serde_json::Value;

use super::BackendRegistry;

impl BackendRegistry {
    pub fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        self.agent_builtin.list_mcp_servers()
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<McpToolInfo>, String> {
        crate::map_frontend_err!(self.agent_builtin.list_mcp_tools(server_id).await)
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> std::result::Result<AgentToolResult, String> {
        crate::map_frontend_err!(
            self.agent_builtin
                .call_tool_direct(server_id, tool_name, params)
                .await
        )
    }
}
