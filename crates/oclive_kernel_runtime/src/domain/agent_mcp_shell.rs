//! 仅 MCP 工具链（列表/调用），不执行 ReAct；在 `kernel-agent` 开、`default-agent-providers` 关时使用。

use super::{AgentDebugTrace, AgentInput, AgentOutput, AgentProvider};
use crate::error::Result;
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;

pub struct McpShellAgent {
    mcp: Arc<McpClient>,
    traces: RwLock<Vec<AgentDebugTrace>>,
}

impl McpShellAgent {
    pub fn new(mcp: Arc<McpClient>) -> Self {
        Self {
            mcp,
            traces: RwLock::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn recent_traces(&self) -> Vec<AgentDebugTrace> {
        self.traces.read().clone()
    }

    pub fn clear_traces(&self) {
        self.traces.write().clear();
    }

    #[must_use]
    pub async fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        self.mcp.list_servers().await
    }

    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>> {
        self.mcp.list_tools(server_id).await
    }

    pub async fn call_tool_direct(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult> {
        self.mcp.call_tool(server_id, tool_name, params).await
    }
}

#[async_trait]
impl AgentProvider for McpShellAgent {
    async fn process(&self, _: AgentInput) -> Result<AgentOutput> {
        Ok(AgentOutput {
            handled: false,
            reply: String::new(),
        })
    }
}
