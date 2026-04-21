use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

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

pub struct BuiltinReActAgent {
    llm: Arc<dyn LlmClient>,
    mcp: Arc<McpClient>,
    traces: RwLock<Vec<AgentDebugTrace>>,
}

impl BuiltinReActAgent {
    #[must_use]
    pub fn new(llm: Arc<dyn LlmClient>, mcp: Arc<McpClient>) -> Self {
        Self {
            llm,
            mcp,
            traces: RwLock::new(Vec::new()),
        }
    }

    fn push_trace(&self, trace: AgentDebugTrace) {
        const MAX_TRACES: usize = 40;
        let mut w = self.traces.write();
        w.push(trace);
        if w.len() > MAX_TRACES {
            let drop_n = w.len() - MAX_TRACES;
            w.drain(0..drop_n);
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
    pub fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        self.mcp.list_servers()
    }

    pub fn call_tool_direct(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult> {
        self.mcp.call_tool(server_id, tool_name, params)
    }

    fn first_weather_server(&self) -> Option<McpServerManifest> {
        self.mcp
            .list_servers()
            .into_iter()
            .find(|s| s.tools.iter().any(|t| t.name == "get_weather"))
    }

    fn extract_city_with_fallback(&self, message: &str, llm_raw: &str) -> String {
        if let Ok(v) = serde_json::from_str::<Value>(llm_raw) {
            if let Some(city) = v.get("city").and_then(|x| x.as_str()) {
                let t = city.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
        let msg = message.trim();
        let markers = ["帮我查一下", "查一下", "查询", "查"];
        for m in markers {
            if let Some(rest) = msg.strip_prefix(m) {
                let c = rest.replace("天气", "").replace('的', "");
                let t = c.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
        "深圳".to_string()
    }
}

#[async_trait]
impl AgentProvider for BuiltinReActAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let message = input.message.trim().to_string();
        if !message.contains("天气") {
            return Ok(AgentOutput {
                handled: false,
                reply: String::new(),
            });
        }
        let mut trace = AgentDebugTrace {
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            role_id: input.role_id.clone(),
            session_namespace: input.session_namespace.clone(),
            message: message.clone(),
            plan: String::new(),
            tool_calls: Vec::new(),
            reply: String::new(),
            error: None,
        };

        let Some(server) = self.first_weather_server() else {
            let reply = "我想帮你查天气，但当前没有可用的 weather skill（请先配置 MCP server）。".to_string();
            trace.plan = "intent=weather; tool=none".to_string();
            trace.reply = reply.clone();
            trace.error = Some("missing weather skill".to_string());
            self.push_trace(trace);
            return Ok(AgentOutput {
                handled: true,
                reply,
            });
        };

        let planner_prompt = format!(
            "你是任务规划器。用户输入：{msg}\n\
            只输出 JSON：{{\"intent\":\"weather\",\"city\":\"城市名\",\"tool\":\"get_weather\"}}。",
            msg = message
        );
        let llm_plan_raw = self
            .llm
            .generate_tag(input.model.as_str(), planner_prompt.as_str())
            .await
            .unwrap_or_else(|_| "{}".to_string());
        let city = self.extract_city_with_fallback(message.as_str(), llm_plan_raw.as_str());
        trace.plan = format!(
            "intent=weather; server={}; tool=get_weather; city={}",
            server.id, city
        );

        let params = json!({ "city": city });
        match self
            .mcp
            .call_tool(server.id.as_str(), "get_weather", params.clone())
        {
            Ok(tool_result) => {
                trace.tool_calls.push(AgentToolCallTrace {
                    server_id: tool_result.server_id.clone(),
                    tool_name: tool_result.tool_name.clone(),
                    params,
                    result: tool_result.result.clone(),
                });
                let reply = if let Some(s) = tool_result.result.get("summary").and_then(|x| x.as_str()) {
                    s.to_string()
                } else if let Some(s) = tool_result.result.get("text").and_then(|x| x.as_str()) {
                    s.to_string()
                } else {
                    format!("天气结果：{}", tool_result.result)
                };
                trace.reply = reply.clone();
                self.push_trace(trace);
                Ok(AgentOutput {
                    handled: true,
                    reply,
                })
            }
            Err(e) => {
                let reply = format!("天气查询失败：{}", e);
                trace.error = Some(e.to_string());
                trace.reply = reply.clone();
                self.push_trace(trace);
                Ok(AgentOutput {
                    handled: true,
                    reply,
                })
            }
        }
    }
}
