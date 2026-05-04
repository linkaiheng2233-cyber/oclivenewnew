//! Agent 任务门面（Builtin ReAct 等实现留在 runtime）。

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
