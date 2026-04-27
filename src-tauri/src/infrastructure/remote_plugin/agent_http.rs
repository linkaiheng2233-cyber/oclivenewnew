//! JSON-RPC: `agent.process`

use crate::domain::agent::{AgentInput, AgentOutput, AgentProvider};
use crate::error::Result;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use async_trait::async_trait;
use serde_json::json;

const METHOD_AGENT_PROCESS: &str = "agent.process";

pub struct RemoteAgentHttp {
    client: reqwest::Client,
    cfg: RemotePluginHttpConfig,
}

impl RemoteAgentHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest async client");
        Self { client, cfg }
    }
}

#[async_trait]
impl AgentProvider for RemoteAgentHttp {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let params = json!({
            "role_id": input.role_id,
            "session_namespace": input.session_namespace,
            "message": input.message,
            "model": input.model,
        });
        let v = jsonrpc::call_async(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_AGENT_PROCESS,
            params,
            self.cfg.bearer_token.as_deref(),
        )
        .await?;
        let handled = v
            .get("handled")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let reply = v
            .get("reply")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        Ok(AgentOutput { handled, reply })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_match_remote_protocol() {
        assert_eq!(METHOD_AGENT_PROCESS, "agent.process");
    }
}

