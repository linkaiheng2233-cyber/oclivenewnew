use crate::domain::agent::{AgentProvider, BuiltinReActAgent};
use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
    ComplexEmotionProvider,
};
use crate::domain::event_estimator::{BuiltinEventEstimator, EventEstimator};
use crate::domain::local_plugin_bridge::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LocalPluginRegistry,
};
use crate::domain::memory_retrieval::{BuiltinMemoryRetrieval, MemoryRetrieval};
use crate::domain::noop_slot_backends::{
    NoopAgentProvider, NoopEventEstimator, NoopLlmClient, NoopMemoryRetrieval, NoopPromptAssembler,
    NoopUserEmotionAnalyzer,
};
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::{BuiltinPromptAssembler, PromptAssembler};
use crate::domain::user_emotion_analyzer::{BuiltinUserEmotionAnalyzer, UserEmotionAnalyzer};
use crate::infrastructure::agent_mcp_bridge::AgentMcpBridge;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::function_call_parser::BuiltinFunctionCallingParser;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::mcp_client::McpClient;
use crate::infrastructure::remote_plugin::{self, PluginRemoteGroup, RemoteComplexEmotionHttp};
use crate::models::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use async_trait::async_trait;
use oclive_kernel_contracts::{
    AgentMcpRegistryPort, LocalPluginRegistryPort, McpBridgePort, MemoryBackendPort,
    SlotBackendFactoryPort,
};
use oclive_kernel_types::{AgentDebugTrace, AgentToolResult, McpServerInfo, McpToolInfo};
use oclive_validation::SlotRegistryEntry;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use crate::domain::plugin_host::PluginHostError;

mod agent;
mod directory;
mod mcp;
mod slots;

struct BuiltinComplexEmotionArc;

impl ComplexEmotionProvider for BuiltinComplexEmotionArc {
    fn resolve_turn(
        &self,
        input: &ComplexEmotionInput,
    ) -> crate::error::Result<ComplexEmotionOutput> {
        Ok(BuiltinKeywordComplexEmotionProvider.resolve_turn_inner(input))
    }
}

struct RemoteComplexEmotionArc(Arc<RemoteComplexEmotionHttp>);

impl ComplexEmotionProvider for RemoteComplexEmotionArc {
    fn resolve_turn(
        &self,
        input: &ComplexEmotionInput,
    ) -> crate::error::Result<ComplexEmotionOutput> {
        self.0.resolve_turn(input)
    }
}

struct NoopComplexEmotionArc;

impl ComplexEmotionProvider for NoopComplexEmotionArc {
    fn resolve_turn(
        &self,
        _input: &ComplexEmotionInput,
    ) -> crate::error::Result<ComplexEmotionOutput> {
        Ok(ComplexEmotionOutput {
            source: "none".into(),
            narrative_hint: String::new(),
            labels: vec![],
            pattern: None,
            confidence: 0.0,
            intensity: 0.0,
            dissonance_score: 0.0,
            degraded_to_builtin: false,
            extension: None,
        })
    }
}

/// Backend registry: manages builtin / remote slots and provides a scaffold for local provider registration.
pub struct BackendRegistry {
    memory_builtin: Arc<dyn MemoryRetrieval>,
    memory_remote: OnceLock<Arc<dyn MemoryRetrieval>>,
    emotion_builtin: Arc<dyn UserEmotionAnalyzer>,
    emotion_remote: OnceLock<Arc<dyn UserEmotionAnalyzer>>,
    event_builtin: Arc<dyn EventEstimator>,
    event_remote: OnceLock<Arc<dyn EventEstimator>>,
    prompt_builtin: Arc<dyn PromptAssembler>,
    prompt_remote: OnceLock<Arc<dyn PromptAssembler>>,
    llm_remote: OnceLock<Arc<dyn LlmClient>>,
    llm_ollama: Arc<dyn LlmClient>,
    agent_builtin: Arc<BuiltinReActAgent>,
    agent_mcp_bridge: Arc<dyn McpBridgePort>,
    agent_remote: OnceLock<Arc<dyn AgentProvider>>,
    agent_none: Arc<dyn AgentProvider>,
    memory_none: Arc<dyn MemoryRetrieval>,
    emotion_none: Arc<dyn UserEmotionAnalyzer>,
    event_none: Arc<dyn EventEstimator>,
    prompt_none: Arc<dyn PromptAssembler>,
    llm_none: Arc<dyn LlmClient>,
    remote_plugin_group: OnceLock<PluginRemoteGroup>,
    local_plugins: RwLock<LocalPluginRegistry>,
    directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
    remote_http_client: Arc<reqwest::Client>,
    directory_memory_cache: RwLock<BTreeMap<String, Arc<dyn MemoryRetrieval>>>,
    directory_emotion_cache: RwLock<BTreeMap<String, Arc<dyn UserEmotionAnalyzer>>>,
    directory_event_cache: RwLock<BTreeMap<String, Arc<dyn EventEstimator>>>,
    directory_prompt_cache: RwLock<BTreeMap<String, Arc<dyn PromptAssembler>>>,
    directory_llm_cache: RwLock<BTreeMap<String, Arc<dyn LlmClient>>>,
    directory_agent_cache: RwLock<BTreeMap<String, Arc<dyn AgentProvider>>>,
}

impl BackendRegistry {
    fn remote_plugin_group(&self) -> &PluginRemoteGroup {
        self.remote_plugin_group.get_or_init(|| {
            remote_plugin::plugin_remote_group(
                self.remote_http_client.clone(),
                self.remote_fallback_allowed.clone(),
                self.high_risk_grants.clone(),
            )
        })
    }

    pub fn from_runtime(
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
        app_data_dir: PathBuf,
        high_risk_grants: Arc<HighRiskGrantStore>,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> Self {
        let llm_ollama = llm;
        let mcp = Arc::new(McpClient::new(app_data_dir, high_risk_grants.clone()));
        let agent_mcp_bridge: Arc<dyn McpBridgePort> = Arc::new(AgentMcpBridge::new(mcp));
        let parser: Arc<dyn oclive_kernel_contracts::FunctionCallingParserPort> =
            Arc::new(BuiltinFunctionCallingParser);
        let agent_builtin = Arc::new(BuiltinReActAgent::new(
            llm_ollama.clone(),
            agent_mcp_bridge.clone(),
            parser,
        ));
        let agent_none: Arc<dyn AgentProvider> = Arc::new(NoopAgentProvider);
        let memory_none: Arc<dyn MemoryRetrieval> = Arc::new(NoopMemoryRetrieval);
        let emotion_none: Arc<dyn UserEmotionAnalyzer> = Arc::new(NoopUserEmotionAnalyzer);
        let event_none: Arc<dyn EventEstimator> = Arc::new(NoopEventEstimator);
        let prompt_none: Arc<dyn PromptAssembler> = Arc::new(NoopPromptAssembler);
        let llm_none: Arc<dyn LlmClient> = Arc::new(NoopLlmClient);
        let remote_http_client = remote_plugin::build_shared_remote_http_client();
        Self {
            memory_builtin: Arc::new(BuiltinMemoryRetrieval),
            memory_remote: OnceLock::new(),
            emotion_builtin: Arc::new(BuiltinUserEmotionAnalyzer),
            emotion_remote: OnceLock::new(),
            event_builtin: Arc::new(BuiltinEventEstimator),
            event_remote: OnceLock::new(),
            prompt_builtin: Arc::new(BuiltinPromptAssembler),
            prompt_remote: OnceLock::new(),
            llm_remote: OnceLock::new(),
            llm_ollama,
            agent_builtin,
            agent_mcp_bridge,
            agent_remote: OnceLock::new(),
            agent_none,
            memory_none,
            emotion_none,
            event_none,
            prompt_none,
            llm_none,
            remote_plugin_group: OnceLock::new(),
            local_plugins: RwLock::new(LocalPluginRegistry::default()),
            directory_runtime,
            remote_fallback_allowed,
            high_risk_grants,
            remote_http_client,
            directory_memory_cache: RwLock::new(BTreeMap::new()),
            directory_emotion_cache: RwLock::new(BTreeMap::new()),
            directory_event_cache: RwLock::new(BTreeMap::new()),
            directory_prompt_cache: RwLock::new(BTreeMap::new()),
            directory_llm_cache: RwLock::new(BTreeMap::new()),
            directory_agent_cache: RwLock::new(BTreeMap::new()),
        }
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), PluginHostError> {
        crate::map_plugin_err!(self.local_plugins.write().register_provider(descriptor))
    }

    #[must_use]
    pub fn local_providers_for(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.local_plugins
            .read()
            .providers_for_capability(capability)
    }

    #[must_use]
    pub fn local_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        self.local_plugins.read().all_providers()
    }

    #[must_use]
    pub fn remote_fallback_allowed(&self) -> Arc<AtomicBool> {
        self.remote_fallback_allowed.clone()
    }

    #[must_use]
    pub fn high_risk_grants(&self) -> Arc<HighRiskGrantStore> {
        self.high_risk_grants.clone()
    }
}

impl MemoryBackendPort for BackendRegistry {
    fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        BackendRegistry::memory_retrieval_for_plugin_backends(self, backends)
    }

    fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
        BackendRegistry::memory_retrieval(self, b)
    }
}

impl SlotBackendFactoryPort for BackendRegistry {
    fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        BackendRegistry::agent_for_plugin_backends(self, backends)
    }

    fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider> {
        BackendRegistry::agent_for(self, b)
    }

    fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        BackendRegistry::user_emotion_analyzer_for_backends(self, backends)
    }

    fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        BackendRegistry::user_emotion_analyzer(self, b)
    }

    fn event_estimator_for_backends(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator> {
        BackendRegistry::event_estimator_for_backends(self, backends)
    }

    fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        BackendRegistry::event_estimator(self, b)
    }

    fn prompt_assembler_for_backends(&self, backends: &PluginBackends) -> Arc<dyn PromptAssembler> {
        BackendRegistry::prompt_assembler_for_backends(self, backends)
    }

    fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        BackendRegistry::prompt_assembler(self, b)
    }

    fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        BackendRegistry::llm_for_plugin_backends(self, backends)
    }

    fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        BackendRegistry::llm_for(self, b)
    }

    fn pick_complex_emotion_winner(
        &self,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn ComplexEmotionProvider> {
        BackendRegistry::pick_complex_emotion_winner(self, slot_registry)
    }

    fn pick_complex_emotion_for_entry(
        &self,
        entry: &SlotRegistryEntry,
    ) -> Arc<dyn ComplexEmotionProvider> {
        BackendRegistry::pick_complex_emotion_for_entry(self, entry)
    }
}

impl LocalPluginRegistryPort for BackendRegistry {
    fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String> {
        BackendRegistry::register_local_provider(self, descriptor).map_err(|e| e.to_string())
    }

    fn local_providers_for(
        &self,
        capability: LocalPluginCapability,
    ) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        BackendRegistry::local_providers_for(self, capability)
    }

    fn local_all_providers(&self) -> Vec<Arc<LocalPluginProviderDescriptor>> {
        BackendRegistry::local_all_providers(self)
    }
}

#[async_trait]
impl AgentMcpRegistryPort for BackendRegistry {
    fn list_mcp_servers(&self) -> Vec<McpServerInfo> {
        BackendRegistry::list_mcp_servers(self)
    }

    async fn list_mcp_tools(&self, server_id: &str) -> Result<Vec<McpToolInfo>, String> {
        BackendRegistry::list_mcp_tools(self, server_id).await
    }

    async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<AgentToolResult, String> {
        BackendRegistry::call_mcp_tool(self, server_id, tool_name, params).await
    }

    fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        BackendRegistry::recent_agent_traces(self)
    }

    fn clear_agent_traces(&self) {
        BackendRegistry::clear_agent_traces(self);
    }

    fn agent_mcp_bridge(&self) -> Arc<dyn McpBridgePort> {
        BackendRegistry::agent_mcp_bridge(self)
    }

    fn remote_fallback_allowed(&self) -> Arc<AtomicBool> {
        BackendRegistry::remote_fallback_allowed(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_complex_emotion_returns_zero_signal() {
        let provider = NoopComplexEmotionArc;
        let input = ComplexEmotionInput {
            role_id: "role".into(),
            scene_id: "scene".into(),
            user_message: "hi".into(),
            bot_reply: "hi".into(),
            recent_dialogue_summary: None,
            previous_narrative_hint: String::new(),
            user_valence: None,
            user_dominance: None,
            previous_user_message: None,
        };
        let out = provider.resolve_turn(&input).unwrap();
        assert_eq!(out.source, "none");
        assert_eq!(out.intensity, 0.0);
        assert!(out.narrative_hint.is_empty());
        assert!(out.labels.is_empty());
        assert!(!out.degraded_to_builtin);
    }
}
