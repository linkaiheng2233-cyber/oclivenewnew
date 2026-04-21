use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::function_call_parser::{
    parse_from_llm_response, to_function_calling_schema, ToolSchemaInput,
};
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

    pub fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>> {
        self.mcp.list_tools(server_id)
    }

    pub fn call_tool_direct(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult> {
        self.mcp.call_tool(server_id, tool_name, params)
    }

    fn collect_tool_schema_inputs(&self) -> Vec<ToolSchemaInput> {
        let mut out: Vec<ToolSchemaInput> = Vec::new();
        for s in self.mcp.list_servers() {
            let tools = self.mcp.list_tools(s.id.as_str()).unwrap_or_else(|_| s.tools.clone());
            for t in tools {
                let name = t.name.trim().to_string();
                if name.is_empty() {
                    continue;
                }
                let desc = t
                    .description
                    .as_ref()
                    .map(|d| format!("server={} {}", s.id, d))
                    .or_else(|| Some(format!("server={}", s.id)));
                out.push(ToolSchemaInput {
                    name,
                    description: desc,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out.dedup_by(|a, b| a.name == b.name);
        out
    }

    fn server_for_tool(&self, tool_name: &str) -> Option<McpServerManifest> {
        for s in self.mcp.list_servers() {
            let listed = self.mcp.list_tools(s.id.as_str()).unwrap_or_else(|_| s.tools.clone());
            if listed.iter().any(|t| t.name.trim() == tool_name) {
                return Some(s);
            }
        }
        None
    }

    fn extract_final_answer(raw: &str) -> Option<String> {
        let v = serde_json::from_str::<Value>(raw).ok()?;
        if let Some(s) = v.get("final_answer").and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        if let Some(s) = v.get("answer").and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
        None
    }
}

#[async_trait]
impl AgentProvider for BuiltinReActAgent {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let message = input.message.trim().to_string();
        if message.is_empty() {
            return Ok(AgentOutput {
                handled: false,
                reply: String::new(),
            });
        }
        let tool_schema_inputs = self.collect_tool_schema_inputs();
        if tool_schema_inputs.is_empty() {
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
        trace.plan = format!(
            "react + function-calling; tools={}",
            tool_schema_inputs
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<String>>()
                .join(",")
        );
        let schema = to_function_calling_schema(&tool_schema_inputs);
        let mut observations: Vec<String> = Vec::new();
        for _ in 0..3 {
            let prompt = format!(
                "你是 oclive Agent。可用函数 schema: {schema}\n\
                 用户请求: {msg}\n\
                 观察历史: {obs}\n\
                 你必须输出 JSON：\n\
                 1) 若要调工具: {{\"tool_calls\":[{{\"id\":\"1\",\"function\":{{\"name\":\"tool_name\",\"arguments\":{{...}}}}}}]}}\n\
                 2) 若可直接回答: {{\"final_answer\":\"...\"}}\n\
                 不要输出 markdown。",
                schema = schema,
                msg = message,
                obs = observations.join(" | ")
            );
            let llm_raw = self
                .llm
                .generate(input.model.as_str(), prompt.as_str())
                .await
                .unwrap_or_else(|_| "{}".to_string());
            if let Some(answer) = Self::extract_final_answer(&llm_raw) {
                trace.reply = answer.clone();
                self.push_trace(trace);
                return Ok(AgentOutput {
                    handled: true,
                    reply: answer,
                });
            }
            let calls = parse_from_llm_response(&llm_raw);
            if calls.is_empty() {
                break;
            }
            for call in calls {
                let tool_name = call.function.name.trim().to_string();
                if tool_name.is_empty() {
                    continue;
                }
                let Some(server) = self.server_for_tool(tool_name.as_str()) else {
                    let msg = format!("tool {} has no mapped server", tool_name);
                    trace.error = Some(msg.clone());
                    observations.push(msg);
                    continue;
                };
                match self.mcp.call_tool(
                    server.id.as_str(),
                    tool_name.as_str(),
                    call.function.arguments.clone(),
                ) {
                    Ok(result) => {
                        trace.tool_calls.push(AgentToolCallTrace {
                            server_id: result.server_id.clone(),
                            tool_name: result.tool_name.clone(),
                            params: call.function.arguments.clone(),
                            result: result.result.clone(),
                        });
                        observations.push(format!(
                            "{}.{} -> {}",
                            result.server_id, result.tool_name, result.result
                        ));
                    }
                    Err(e) => {
                        let msg = format!("tool {} failed: {}", tool_name, e);
                        trace.error = Some(msg.clone());
                        observations.push(msg);
                    }
                }
            }
        }
        let fallback = if let Some(last) = trace.tool_calls.last() {
            if let Some(s) = last.result.get("summary").and_then(|x| x.as_str()) {
                s.to_string()
            } else {
                format!("工具调用完成：{}", last.result)
            }
        } else if let Some(e) = trace.error.clone() {
            format!("Agent 执行未完成：{}", e)
        } else {
            "我已尝试调度工具，但模型没有返回可执行的 function call。".to_string()
        };
        trace.reply = fallback.clone();
        self.push_trace(trace);
        Ok(AgentOutput {
            handled: true,
            reply: fallback,
        })
    }
}
