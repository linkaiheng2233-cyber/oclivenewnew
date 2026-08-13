//! Split from backend_registry.rs (zero semantic change, facade in mod.rs).

use crate::domain::agent::AgentProvider;
use crate::domain::fallback_agent::FallbackAgentProvider;
use crate::infrastructure::remote_plugin::{
    agent_remote_backend, AgentRpcProvider, RemotePluginHttpConfig,
};
use crate::models::{AgentBackend, PluginBackends};
use oclive_kernel_contracts::McpBridgePort;
use oclive_kernel_types::AgentDebugTrace;
use std::sync::Arc;

use super::BackendRegistry;

impl BackendRegistry {
    pub(super) fn agent_remote(&self) -> Arc<dyn AgentProvider> {
        self.agent_remote
            .get_or_init(|| {
                agent_remote_backend(
                    self.remote_http_client.clone(),
                    self.agent_builtin.clone() as Arc<dyn AgentProvider>,
                    self.agent_mcp_bridge.clone(),
                    self.remote_fallback_allowed.clone(),
                    self.high_risk_grants.clone(),
                )
            })
            .clone()
    }

    pub(super) fn agent_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        let builtin = self.agent_builtin.clone() as Arc<dyn AgentProvider>;
        self.pick_directory_slot(
            "agent",
            backends,
            &self.directory_agent_cache,
            |s| &s.agent,
            builtin.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), false);
                let primary = Arc::new(AgentRpcProvider::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.remote_fallback_allowed.clone(),
                    reg.high_risk_grants.clone(),
                    None,
                    reg.agent_mcp_bridge.clone(),
                )) as Arc<dyn AgentProvider>;
                FallbackAgentProvider::new(
                    primary,
                    reg.agent_builtin.clone() as Arc<dyn AgentProvider>,
                    "directory",
                )
            },
        )
    }

    pub(crate) fn agent_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn AgentProvider> {
        match backends.agent {
            AgentBackend::Builtin => self.agent_builtin.clone(),
            AgentBackend::Remote => self.agent_remote(),
            AgentBackend::Directory => self.agent_directory_slot(backends),
            AgentBackend::None => self.agent_none.clone(),
        }
    }

    pub fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider> {
        self.agent_for_plugin_backends(&PluginBackends {
            agent: b,
            ..Default::default()
        })
    }

    pub fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        self.agent_builtin.recent_traces()
    }

    pub fn clear_agent_traces(&self) {
        self.agent_builtin.clear_traces();
    }

    #[must_use]
    pub fn agent_mcp_bridge(&self) -> Arc<dyn McpBridgePort> {
        self.agent_mcp_bridge.clone()
    }
}
