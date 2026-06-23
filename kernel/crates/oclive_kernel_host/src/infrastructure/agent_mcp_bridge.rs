//! Unified MCP tool listing and invocation for all Agent backends.

use crate::domain::agent::AgentToolCallTrace;
use crate::error::{AppError, Result};
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use async_trait::async_trait;
use oclive_kernel_contracts::McpBridgePort;
use oclive_kernel_types::ToolSchemaInput;
use oclive_kernel_types::{AgentToolResult, AgentToolSchema, McpServerInfo, McpToolInfo};
use serde_json::Value;
use std::sync::Arc;

fn server_to_info(s: &McpServerManifest) -> McpServerInfo {
    McpServerInfo {
        id: s.id.clone(),
        name: s.name.clone(),
        transport: s.transport.clone(),
        url: s.url.clone(),
        command: s.command.clone(),
        args: s.args.clone(),
        tools: s
            .tools
            .iter()
            .map(|t| McpToolInfo {
                name: t.name.clone(),
                description: t.description.clone(),
            })
            .collect(),
        timeout_ms: s.timeout_ms,
    }
}

fn call_result_to_dto(result: &McpToolCallResult) -> AgentToolResult {
    AgentToolResult {
        server_id: result.server_id.clone(),
        tool_name: result.tool_name.clone(),
        params: Value::Null,
        result: result.result.clone(),
        error: None,
    }
}

/// Host-side MCP bridge: grant checks and tool execution live here only.
pub struct AgentMcpBridge {
    mcp: Arc<McpClient>,
}

impl AgentMcpBridge {
    #[must_use]
    pub fn new(mcp: Arc<McpClient>) -> Self {
        Self { mcp }
    }

    #[must_use]
    pub fn mcp(&self) -> &McpClient {
        &self.mcp
    }

    async fn list_tools_for_server(
        &self,
        s: &McpServerManifest,
    ) -> Vec<crate::infrastructure::mcp_client::McpToolManifest> {
        match self.mcp.list_tools(s.id.as_str()).await {
            Ok(t) => t,
            Err(AppError::HighRiskCapabilityNotGranted { .. }) => {
                tracing::info!(
                    target: "oclive_agent",
                    server_id = %s.id,
                    "mcp server omitted from agent tool schema (transport not granted)"
                );
                Vec::new()
            }
            Err(_) => s.tools.clone(),
        }
    }

    async fn server_for_tool(&self, tool_name: &str) -> Option<McpServerManifest> {
        if let Some((server_id, bare)) = tool_name.split_once("::") {
            let server_id = server_id.trim();
            let bare = bare.trim();
            if server_id.is_empty() || bare.is_empty() {
                return None;
            }
            let server = self
                .mcp
                .list_servers()
                .into_iter()
                .find(|s| s.id.trim() == server_id)?;
            let listed = self.list_tools_for_server(&server).await;
            if listed.iter().any(|t| t.name.trim() == bare) {
                return Some(server);
            }
            return None;
        }
        for s in self.mcp.list_servers() {
            let listed = self.list_tools_for_server(&s).await;
            if listed.iter().any(|t| t.name.trim() == tool_name) {
                return Some(s);
            }
        }
        None
    }

    /// Build a trace row from an MCP result.
    #[must_use]
    pub fn trace_from_call(
        &self,
        params: &Value,
        result: &McpToolCallResult,
    ) -> AgentToolCallTrace {
        AgentToolCallTrace {
            server_id: result.server_id.clone(),
            tool_name: result.tool_name.clone(),
            params: params.clone(),
            result: result.result.clone(),
        }
    }
}

#[async_trait]
impl McpBridgePort for AgentMcpBridge {
    fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        self.mcp.list_servers().iter().map(server_to_info).collect()
    }

    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>> {
        let tools = self.mcp.list_tools(server_id).await?;
        Ok(tools
            .into_iter()
            .map(|t| McpToolInfo {
                name: t.name,
                description: t.description,
            })
            .collect())
    }

    async fn list_agent_tool_schemas(&self) -> Result<Vec<AgentToolSchema>> {
        let mut out: Vec<ToolSchemaInput> = Vec::new();
        for s in self.mcp.list_servers() {
            let tools = self.list_tools_for_server(&s).await;
            for t in tools {
                let name = t.name.trim().to_string();
                if name.is_empty() {
                    continue;
                }
                let qualified = format!("{}::{}", s.id.trim(), name);
                let desc = t
                    .description
                    .as_ref()
                    .map(|d| format!("server={} {}", s.id, d))
                    .or_else(|| Some(format!("server={}", s.id)));
                out.push(ToolSchemaInput {
                    name: qualified,
                    description: desc,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out
            .into_iter()
            .map(|t| AgentToolSchema {
                name: t.name,
                description: t.description,
            })
            .collect())
    }

    async fn call_tool_qualified(&self, tool_name: &str, params: Value) -> Result<AgentToolResult> {
        let tool_name = tool_name.trim();
        let Some(server) = self.server_for_tool(tool_name).await else {
            return Err(AppError::InvalidParameter(format!(
                "agent tool {tool_name} has no mapped MCP server"
            )));
        };
        let bare_tool = tool_name
            .split_once("::")
            .map(|(_, bare)| bare.trim())
            .unwrap_or(tool_name);
        let result = self
            .mcp
            .call_tool(server.id.as_str(), bare_tool, params)
            .await?;
        Ok(call_result_to_dto(&result))
    }

    async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<AgentToolResult> {
        let result = self.mcp.call_tool(server_id, tool_name, params).await?;
        Ok(call_result_to_dto(&result))
    }
}
