//! MCP tool bridge port for Agent backends (implementation in host infrastructure).

use async_trait::async_trait;
use oclive_kernel_types::{AgentToolResult, AgentToolSchema, McpServerInfo, McpToolInfo, Result};
use serde_json::Value;

/// Host-side MCP bridge: grant checks and tool execution live in infrastructure only.
#[async_trait]
pub trait McpBridgePort: Send + Sync {
    /// List configured MCP servers (manifest scan; no network I/O).
    fn list_mcp_servers(&self) -> Vec<McpServerInfo>;

    /// List tools for one server (may perform MCP list_tools RPC).
    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>>;

    /// Collect function-calling schemas for all granted MCP tools.
    async fn list_agent_tool_schemas(&self) -> Result<Vec<AgentToolSchema>>;

    /// Invoke one MCP tool by qualified `server::tool` or bare tool name.
    async fn call_tool_qualified(&self, tool_name: &str, params: Value) -> Result<AgentToolResult>;

    /// Direct MCP call when `server_id` and bare `tool_name` are known.
    async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<AgentToolResult>;
}
