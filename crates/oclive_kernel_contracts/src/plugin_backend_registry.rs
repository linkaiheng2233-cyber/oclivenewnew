//! Plugin backend registry port: resolves `plugin_backends` → executable trait handles.

use async_trait::async_trait;
use oclive_kernel_types::{
    AgentBackend, AgentDebugTrace, AgentToolResult, EmotionBackend, EventBackend, LlmBackend,
    LocalPluginCapability, LocalPluginProviderDescriptor, McpServerInfo, McpToolInfo,
    MemoryBackend, PluginBackends, PromptBackend, SlotRegistryEntry,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::{
    AgentProvider, ComplexEmotionProvider, EventEstimator, LlmClient, McpBridgePort,
    MemoryRetrieval, PromptAssembler, UserEmotionAnalyzer,
};

/// Factory + resolver for six-slot plugin backends (builtin / remote / directory / local).
#[async_trait]
pub trait PluginBackendRegistryPort: Send + Sync {
    fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider>;
    fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider>;
    fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval>;
    fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval>;
    fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer>;
    fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer>;
    fn event_estimator_for_backends(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator>;
    fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator>;
    fn prompt_assembler_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler>;
    fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler>;
    fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient>;
    fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient>;

    fn resolve_complex_emotion_winner(
        &self,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn ComplexEmotionProvider>;

    fn resolve_complex_emotion_for_entry(
        &self,
        entry: &SlotRegistryEntry,
    ) -> Arc<dyn ComplexEmotionProvider>;

    /// Register a local plugin provider descriptor.
    ///
    /// # Errors
    ///
    /// Returns an error string when the descriptor is invalid or already registered.
    fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String>;

    fn local_providers_for(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>>;

    fn local_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>>;

    fn list_mcp_servers(&self) -> Vec<McpServerInfo>;

    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>, String>;

    async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<AgentToolResult, String>;

    fn recent_agent_traces(&self) -> Vec<AgentDebugTrace>;

    fn clear_agent_traces(&self);

    fn agent_mcp_bridge(&self) -> Arc<dyn McpBridgePort>;

    fn remote_fallback_allowed(&self) -> Arc<AtomicBool>;
}
