//! JSON-RPC：`agent.process` — see AGENT_REMOTE_PROTOCOL.md

use oclive_kernel_contracts::McpBridgePort;
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::{AppError, Result};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterAsync;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use async_trait::async_trait;
use oclive_kernel_contracts::AgentProvider;
use oclive_kernel_types::{
    AgentInput, AgentOutput, AgentProcessRpcResult, AgentToolResult, AgentTurnContext,
};
use serde_json::{json, Value};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_AGENT_PROCESS: &str = "agent.process";
const MAX_REACT_LOOPS: usize = 3;

/// Remote or directory JSON-RPC agent backend (host-orchestrated tool loop).
pub struct AgentRpcProvider {
    adapter: RemotePluginAdapterAsync,
    bridge: Arc<dyn McpBridgePort>,
}

impl AgentRpcProvider {
    #[must_use]
    pub fn new(
        http_client: Arc<reqwest::Client>,
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        network_grant_id: Option<String>,
        bridge: Arc<dyn McpBridgePort>,
    ) -> Self {
        Self {
            adapter: RemotePluginAdapterAsync::new(
                http_client,
                cfg,
                remote_fallback_allowed,
                high_risk_grants,
                network_grant_id,
            ),
            bridge,
        }
    }

    fn build_params(input: &AgentInput, turn_context: &AgentTurnContext) -> Value {
        json!({
            "protocol_version": input.protocol_version,
            "role_id": input.role_id,
            "session_namespace": input.session_namespace,
            "scene_id": input.scene_id,
            "message": input.message,
            "model": input.model,
            "constraints": input.constraints,
            "tools": input.tools,
            "turn_context": turn_context,
        })
    }

    async fn call_process(
        &self,
        input: &AgentInput,
        turn_context: &AgentTurnContext,
    ) -> Result<AgentProcessRpcResult> {
        let params = Self::build_params(input, turn_context);
        let v = self
            .adapter
            .http
            .call_plugin(METHOD_AGENT_PROCESS, params)
            .await?;
        serde_json::from_value(v).map_err(|e| serde_to_ollama("agent.process decode", e))
    }

    async fn execute_tool_calls(
        &self,
        turn_context: &mut AgentTurnContext,
        tool_calls: &[oclive_kernel_types::AgentRpcToolCall],
    ) -> Result<()> {
        turn_context.tool_results.clear();
        for tc in tool_calls {
            let qualified = tc.tool_name.trim();
            if qualified.is_empty() {
                continue;
            }
            match self
                .bridge
                .call_tool_qualified(qualified, tc.params.clone())
                .await
            {
                Ok(result) => {
                    turn_context.tool_results.push(AgentToolResult {
                        server_id: result.server_id.clone(),
                        tool_name: result.tool_name.clone(),
                        params: tc.params.clone(),
                        result: result.result.clone(),
                        error: None,
                    });
                }
                Err(e) => {
                    turn_context.tool_results.push(AgentToolResult {
                        server_id: tc
                            .server_id
                            .clone()
                            .unwrap_or_else(|| "unknown".into()),
                        tool_name: qualified.to_string(),
                        params: tc.params.clone(),
                        result: Value::Null,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentProvider for AgentRpcProvider {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let message = input.message.trim();
        if message.is_empty() {
            return Ok(AgentOutput {
                handled: false,
                reply: String::new(),
            });
        }
        let mut turn_context = input.turn_context.clone();
        for _ in 0..MAX_REACT_LOOPS {
            let result = self.call_process(&input, &turn_context).await?;
            if result.handled {
                return Ok(AgentOutput {
                    handled: true,
                    reply: result.reply.unwrap_or_default(),
                });
            }
            let Some(tool_calls) = result.tool_calls.filter(|t| !t.is_empty()) else {
                return Ok(AgentOutput {
                    handled: false,
                    reply: String::new(),
                });
            };
            self.execute_tool_calls(&mut turn_context, &tool_calls)
                .await?;
        }
        Err(AppError::RemoteServiceUnavailable(
            "agent.process exceeded max tool rounds without handled reply".into(),
        ))
    }
}
