//! Agent module input/output (pure data structures).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Function-calling tool entry passed to agent backends / `agent.process` RPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentToolSchema {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Role constraints injected by the host before each agent turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRoleConstraints {
    /// Seven-dimensional personality vector (0–1 each); defaults to 0.5 when unknown.
    #[serde(default = "default_personality_vector")]
    pub personality_vector: Vec<f32>,
    #[serde(default)]
    pub relation_state: String,
    #[serde(default)]
    pub favorability: f64,
    #[serde(default)]
    pub scene_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_text: Option<String>,
}

fn default_personality_vector() -> Vec<f32> {
    vec![0.5; 7]
}

impl Default for AgentRoleConstraints {
    fn default() -> Self {
        Self {
            personality_vector: default_personality_vector(),
            relation_state: "Stranger".to_string(),
            favorability: 0.0,
            scene_label: String::new(),
            interaction_mode: None,
            policy_text: None,
        }
    }
}

/// One tool invocation result for multi-round `agent.process`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentToolResult {
    pub server_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub result: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Multi-turn agent context (recent dialogue + tool observations).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentTurnContext {
    #[serde(default)]
    pub recent_turns: Vec<(String, String)>,
    #[serde(default)]
    pub tool_results: Vec<AgentToolResult>,
}

/// Tool call requested by a remote/directory agent sidecar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRpcToolCall {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    pub tool_name: String,
    #[serde(default)]
    pub params: Value,
}

/// Decoded `agent.process` JSON-RPC result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentProcessRpcResult {
    pub handled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AgentRpcToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
}

/// Input for a single agent turn (role, session, message, model, constraints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub role_id: String,
    pub session_namespace: String,
    pub message: String,
    pub model: String,
    #[serde(default)]
    pub scene_id: String,
    #[serde(default)]
    pub constraints: AgentRoleConstraints,
    #[serde(default)]
    pub tools: Vec<AgentToolSchema>,
    #[serde(default)]
    pub turn_context: AgentTurnContext,
    #[serde(default = "default_protocol_version")]
    pub protocol_version: u32,
}

fn default_protocol_version() -> u32 {
    1
}

impl Default for AgentInput {
    fn default() -> Self {
        Self {
            role_id: String::new(),
            session_namespace: String::new(),
            message: String::new(),
            model: String::new(),
            scene_id: String::new(),
            constraints: AgentRoleConstraints::default(),
            tools: Vec::new(),
            turn_context: AgentTurnContext::default(),
            protocol_version: default_protocol_version(),
        }
    }
}

/// Agent turn result: whether the agent handled the message and the reply text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub handled: bool,
    pub reply: String,
}

/// Parsed function call from LLM output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Value,
}

/// OpenAI-style tool call entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

/// MCP / agent tool schema input for function-calling conversion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSchemaInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// One MCP tool invocation recorded in agent debug traces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentToolCallTrace {
    pub server_id: String,
    pub tool_name: String,
    pub params: Value,
    pub result: Value,
}

/// Agent ReAct debug trace (Tauri `get_agent_debug_traces`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
