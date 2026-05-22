//! Agent 模块输入/输出（纯数据结构）。

use serde::{Deserialize, Serialize};

/// Input for a single agent turn (role, session, message, model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub role_id: String,
    pub session_namespace: String,
    pub message: String,
    pub model: String,
}

/// Agent turn result: whether the agent handled the message and the reply text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub handled: bool,
    pub reply: String,
}
