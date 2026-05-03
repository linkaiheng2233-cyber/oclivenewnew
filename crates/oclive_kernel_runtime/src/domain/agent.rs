use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "kernel-agent")]
#[path = "agent_builtin.rs"]
mod agent_builtin;

#[cfg(feature = "kernel-agent")]
pub use agent_builtin::BuiltinReActAgent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub role_id: String,
    pub session_namespace: String,
    pub message: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCallTrace {
    pub server_id: String,
    pub tool_name: String,
    pub params: Value,
    pub result: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDebugTrace {
    pub timestamp_ms: i64,
    pub role_id: String,
    pub session_namespace: String,
    pub message: String,
    pub plan: String,
    pub tool_calls: Vec<AgentToolCallTrace>,
    pub reply: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub handled: bool,
    pub reply: String,
}

#[async_trait]
pub trait AgentProvider: Send + Sync {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput>;
}

/// Agent 能力关闭时的占位实现（不走 MCP / ReAct）。
pub struct NoopAgent;

#[async_trait]
impl AgentProvider for NoopAgent {
    async fn process(&self, _: AgentInput) -> Result<AgentOutput> {
        Ok(AgentOutput {
            handled: false,
            reply: String::new(),
        })
    }
}
