//! MCP 工具清单与调用抽象（具体 HTTP/stdio 实现在 `oclive_kernel_runtime::mcp_client`）。

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub transport: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub tools: Vec<McpToolManifest>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub server_id: String,
    pub tool_name: String,
    pub result: Value,
}

/// Agent ReAct 等与 MCP 解耦所需的最小异步端口。
#[async_trait]
pub trait McpInvoke: Send + Sync {
    async fn list_servers(&self) -> Vec<McpServerManifest>;
    async fn list_tools(&self, server_id: &str) -> Result<Vec<McpToolManifest>>;
    async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult>;
}
