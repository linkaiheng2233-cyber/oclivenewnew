//! Agent MCP bridge + debug trace port.

use async_trait::async_trait;
use oclive_kernel_types::{AgentDebugTrace, AgentToolResult, McpServerInfo, McpToolInfo};
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::McpBridgePort;

/// MCP listing/call + agent debug traces + remote fallback toggle.
#[async_trait]
pub trait AgentMcpRegistryPort: Send + Sync {
    fn list_mcp_servers(&self) -> Vec<McpServerInfo>;

    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>, String>;

    async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<AgentToolResult, String>;

    fn recent_agent_traces(&self) -> Vec<AgentDebugTrace>;

    fn clear_agent_traces(&self);

    fn agent_mcp_bridge(&self) -> Arc<dyn McpBridgePort>;

    fn remote_fallback_allowed(&self) -> Arc<AtomicBool>;
}
