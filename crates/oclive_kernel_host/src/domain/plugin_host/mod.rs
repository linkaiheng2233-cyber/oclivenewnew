//! # Plugin assembly hub (`PluginHost`)
//!
//! **Role**: Parses role pack configuration (`plugin_backends`, `slot_registry`, directory plugin manifests)
//! into executable **`Arc<dyn …>`** handle sets (`ResolvedRolePlugins`) for the orchestration layer via
//! [`PluginHostPort`](crate::domain::ports::PluginHostPort).
//!
//! Construction with infrastructure dependencies: [`crate::infrastructure::plugin_wiring::build_plugin_host`].

use crate::domain::agent::{AgentDebugTrace, AgentProvider};
use crate::domain::complex_emotion::ComplexEmotionProvider;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::local_plugin_bridge::{LocalPluginCapability, LocalPluginProviderDescriptor};
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::slot_resolver::ResolvedRoleSlots;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::models::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PluginBackendsOverride, PromptBackend, Role,
};
use oclive_kernel_contracts::{McpBridgePort, PluginBackendRegistryPort};
use oclive_kernel_types::{AgentToolResult, McpServerInfo, McpToolInfo};
use oclive_validation::SlotRegistryEntry;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors from [`PluginHost`] during resolve and local provider registration.
#[derive(Debug, Error)]
pub enum PluginHostError {
    #[error("local plugin provider 注册失败: {0}")]
    LocalProviderRegistration(String),
}

impl From<String> for PluginHostError {
    fn from(msg: String) -> Self {
        PluginHostError::LocalProviderRegistration(msg)
    }
}

/// Implementation handles resolved from `role.plugin_backends`; resolve once per `send_message` and reuse.
#[derive(Clone)]
pub struct ResolvedRolePlugins {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
    pub llm: Arc<dyn LlmClient>,
    pub agent: Arc<dyn AgentProvider>,
    /// Blueprint `complex_emotion` slot last-wins resolve (builtin when no registry).
    pub complex_emotion: Arc<dyn ComplexEmotionProvider>,
    /// Per-instance multi-slot view (P3; P4 orchestration serial merge).
    pub slots: Option<ResolvedRoleSlots>,
    /// Merged plugin ids from multiple `agent` directory slots (observability / P4).
    pub merged_agent_directory_plugin_ids: Vec<String>,
}

mod resolver;

pub use resolver::PluginResolver;

/// Compile-time plugin implementation set ([`PluginHost::resolve_for_role`] clones `Arc` per enum variant).
pub struct PluginHost {
    registry: Arc<dyn PluginBackendRegistryPort>,
}

impl PluginHost {
    /// Wraps a pre-built backend registry port (see [`crate::infrastructure::plugin_wiring`]).
    #[must_use]
    pub fn from_registry(registry: Arc<dyn PluginBackendRegistryPort>) -> Self {
        Self { registry }
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), PluginHostError> {
        self.registry
            .register_local_provider(descriptor)
            .map_err(PluginHostError::LocalProviderRegistration)
    }

    #[must_use]
    pub fn local_providers_for(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.registry.local_providers_for(capability)
    }

    #[must_use]
    pub fn local_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.registry.local_all_providers()
    }

    #[must_use]
    pub fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        self.registry.llm_for(b)
    }

    #[must_use]
    pub fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        self.registry.llm_for_plugin_backends(backends)
    }

    #[must_use]
    pub fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider> {
        self.registry.agent_for(b)
    }

    #[must_use]
    pub fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        self.registry.agent_for_plugin_backends(backends)
    }

    #[must_use]
    pub fn agent_mcp_bridge(&self) -> Arc<dyn McpBridgePort> {
        self.registry.agent_mcp_bridge()
    }

    #[must_use]
    pub fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        self.registry.memory_retrieval_for_plugin_backends(backends)
    }

    #[must_use]
    pub fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
        self.registry.memory_retrieval(b)
    }

    #[must_use]
    pub fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        self.registry.user_emotion_analyzer(b)
    }

    #[must_use]
    pub fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        self.registry.user_emotion_analyzer_for_backends(backends)
    }

    #[must_use]
    pub fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        self.registry.event_estimator(b)
    }

    #[must_use]
    pub fn event_estimator_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn EventEstimator> {
        self.registry.event_estimator_for_backends(backends)
    }

    #[must_use]
    pub fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        self.registry.prompt_assembler(b)
    }

    #[must_use]
    pub fn prompt_assembler_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler> {
        self.registry.prompt_assembler_for_backends(backends)
    }

    #[must_use]
    pub fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        self.registry.list_mcp_servers()
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<McpToolInfo>, String> {
        self.registry.list_mcp_tools(server_id).await
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> crate::error::Result<AgentToolResult> {
        self.registry
            .call_mcp_tool(server_id, tool_name, params)
            .await
            .map_err(|e| crate::error::AppError::Unknown(e))
    }

    #[must_use]
    pub fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        self.registry.recent_agent_traces()
    }

    pub fn clear_agent_traces(&self) {
        self.registry.clear_agent_traces();
    }

    /// Resolves all backends declared by the current role pack (one clone of five `Arc`s, reused for the whole conversation).
    #[must_use]
    pub fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            self.registry.as_ref(),
            &role.plugin_backends,
            None,
            role.slot_registry.as_ref(),
        )
    }

    /// Resolves role default backends plus session-level override (equivalent to [`Self::resolve_for_role`] when override is empty).
    #[must_use]
    pub fn resolve_for_role_with_override(
        &self,
        role: &Role,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            self.registry.as_ref(),
            &role.plugin_backends,
            session_override,
            role.slot_registry.as_ref(),
        )
    }

    /// Effective six slots + blueprint registry + optional six-slot session override (v2 hot path).
    #[must_use]
    pub fn resolve_for_effective_backends(
        &self,
        effective_backends: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            self.registry.as_ref(),
            effective_backends,
            session_override,
            slot_registry,
        )
    }
}

impl ResolvedRolePlugins {
    /// Matches `role.plugin_backends` for logging / test assertions (read-only borrow, avoids hot-path clone).
    #[must_use]
    pub fn backends_snapshot(role: &Role) -> &PluginBackends {
        role.plugin_backends.as_ref()
    }
}
