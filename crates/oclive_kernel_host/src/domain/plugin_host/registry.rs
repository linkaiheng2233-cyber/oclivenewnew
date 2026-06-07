use crate::domain::agent::{AgentDebugTrace, AgentProvider, BuiltinReActAgent};
use crate::domain::agent_mcp_bridge::AgentMcpBridge;
use crate::domain::fallback_agent::FallbackAgentProvider;
use crate::domain::noop_slot_backends::{
    NoopAgentProvider, NoopEventEstimator, NoopLlmClient, NoopMemoryRetrieval,
    NoopPromptAssembler, NoopUserEmotionAnalyzer,
};
use crate::domain::event_estimator::{
    BuiltinEventEstimator, BuiltinEventEstimatorV2, EventEstimator,
};
use crate::domain::local_plugin_bridge::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LocalPluginRegistry,
};
use crate::domain::local_plugin_memory_pick::pick_local_memory_provider_refs;
use crate::domain::memory_retrieval::{
    BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2, LocalPluginMemoryRetrieval, MemoryRetrieval,
};
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::{
    BuiltinPromptAssembler, BuiltinPromptAssemblerV2, PromptAssembler,
};
use crate::domain::user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, BuiltinUserEmotionAnalyzerV2, UserEmotionAnalyzer,
};
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use crate::infrastructure::remote_plugin::{
    self, agent_remote_backend, AgentRpcProvider, PluginRemoteGroup, RemoteEventEstimatorHttp,
    RemoteLlmHttp, RemoteMemoryRetrievalHttp, RemotePluginHttpConfig, RemotePromptAssemblerHttp,
    RemoteUserEmotionAnalyzerHttp,
};
use crate::models::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackends, PromptBackend,
};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use super::PluginHostError;

/// Backend registry: manages builtin / remote slots and provides a scaffold for local provider registration.
pub struct BackendRegistry {
    memory_builtin: Arc<dyn MemoryRetrieval>,
    memory_builtin_v2: OnceLock<Arc<dyn MemoryRetrieval>>,
    memory_remote: OnceLock<Arc<dyn MemoryRetrieval>>,
    emotion_builtin: Arc<dyn UserEmotionAnalyzer>,
    emotion_builtin_v2: OnceLock<Arc<dyn UserEmotionAnalyzer>>,
    emotion_remote: OnceLock<Arc<dyn UserEmotionAnalyzer>>,
    event_builtin: Arc<dyn EventEstimator>,
    event_builtin_v2: OnceLock<Arc<dyn EventEstimator>>,
    event_remote: OnceLock<Arc<dyn EventEstimator>>,
    prompt_builtin: Arc<dyn PromptAssembler>,
    prompt_builtin_v2: OnceLock<Arc<dyn PromptAssembler>>,
    prompt_remote: OnceLock<Arc<dyn PromptAssembler>>,
    llm_remote: OnceLock<Arc<dyn LlmClient>>,
    llm_ollama: Arc<dyn LlmClient>,
    agent_builtin: Arc<BuiltinReActAgent>,
    agent_mcp_bridge: Arc<AgentMcpBridge>,
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

fn directory_slot_id(
    slots: &DirectoryPluginSlots,
    pick: impl FnOnce(&DirectoryPluginSlots) -> &Option<String>,
) -> Option<String> {
    pick(slots)
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
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

    fn memory_builtin_v2(&self) -> Arc<dyn MemoryRetrieval> {
        self.memory_builtin_v2
            .get_or_init(|| Arc::new(BuiltinMemoryRetrievalV2))
            .clone()
    }

    fn memory_remote(&self) -> Arc<dyn MemoryRetrieval> {
        self.memory_remote
            .get_or_init(|| self.remote_plugin_group().memory.clone())
            .clone()
    }

    fn emotion_builtin_v2(&self) -> Arc<dyn UserEmotionAnalyzer> {
        self.emotion_builtin_v2
            .get_or_init(|| Arc::new(BuiltinUserEmotionAnalyzerV2))
            .clone()
    }

    fn emotion_remote(&self) -> Arc<dyn UserEmotionAnalyzer> {
        self.emotion_remote
            .get_or_init(|| self.remote_plugin_group().emotion.clone())
            .clone()
    }

    fn event_builtin_v2(&self) -> Arc<dyn EventEstimator> {
        self.event_builtin_v2
            .get_or_init(|| Arc::new(BuiltinEventEstimatorV2))
            .clone()
    }

    fn event_remote(&self) -> Arc<dyn EventEstimator> {
        self.event_remote
            .get_or_init(|| self.remote_plugin_group().event.clone())
            .clone()
    }

    fn prompt_builtin_v2(&self) -> Arc<dyn PromptAssembler> {
        self.prompt_builtin_v2
            .get_or_init(|| Arc::new(BuiltinPromptAssemblerV2))
            .clone()
    }

    fn prompt_remote(&self) -> Arc<dyn PromptAssembler> {
        self.prompt_remote
            .get_or_init(|| self.remote_plugin_group().prompt.clone())
            .clone()
    }

    fn llm_remote(&self) -> Arc<dyn LlmClient> {
        self.llm_remote
            .get_or_init(|| {
                remote_plugin::llm_remote_backend(
                    self.remote_http_client.clone(),
                    self.llm_ollama.clone(),
                    self.remote_fallback_allowed.clone(),
                    self.high_risk_grants.clone(),
                )
            })
            .clone()
    }

    fn agent_remote(&self) -> Arc<dyn AgentProvider> {
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

    fn agent_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        let builtin = self.agent_builtin.clone() as Arc<dyn AgentProvider>;
        self.resolve_directory_slot(
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
                FallbackAgentProvider::new(primary, reg.agent_builtin.clone() as Arc<dyn AgentProvider>, "directory")
            },
        )
    }

    fn resolve_directory_slot<T, Pick, Build>(
        &self,
        module: &'static str,
        backends: &PluginBackends,
        cache: &RwLock<BTreeMap<String, T>>,
        pick: Pick,
        fallback: T,
        build: Build,
    ) -> T
    where
        Pick: FnOnce(&DirectoryPluginSlots) -> &Option<String>,
        Build: FnOnce(&Self, &str, &str) -> T,
        T: Clone + Send + Sync + 'static,
    {
        let Some(_rt) = self.directory_runtime.as_ref() else {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.{module}=directory but directory plugin runtime disabled; using fallback"
            );
            return fallback;
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, pick) else {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.{module}=directory but directory_plugins.{module} missing; using fallback"
            );
            return fallback;
        };
        if let Some(cached) = cache.read().get(&pid).cloned() {
            return cached;
        }
        match _rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let built = build(self, pid.as_str(), url.as_str());
                cache.write().insert(pid, built.clone());
                built
            }
            Err(e) => {
                tracing::error!(
                    target: "oclive_plugin",
                    "directory {module} plugin_id={pid} spawn failed: {e}"
                );
                fallback
            }
        }
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

    pub fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        self.agent_builtin.list_mcp_servers()
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
        crate::map_frontend_err!(self.agent_builtin.list_mcp_tools(server_id).await)
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> std::result::Result<McpToolCallResult, String> {
        crate::map_frontend_err!(
            self.agent_builtin
                .call_tool_direct(server_id, tool_name, params)
                .await
        )
    }

    pub fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        self.agent_builtin.recent_traces()
    }

    pub fn clear_agent_traces(&self) {
        self.agent_builtin.clear_traces();
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
        let agent_mcp_bridge = Arc::new(AgentMcpBridge::new(mcp));
        let agent_builtin = Arc::new(BuiltinReActAgent::new(
            llm_ollama.clone(),
            agent_mcp_bridge.clone(),
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
            memory_builtin_v2: OnceLock::new(),
            memory_remote: OnceLock::new(),
            emotion_builtin: Arc::new(BuiltinUserEmotionAnalyzer),
            emotion_builtin_v2: OnceLock::new(),
            emotion_remote: OnceLock::new(),
            event_builtin: Arc::new(BuiltinEventEstimator),
            event_builtin_v2: OnceLock::new(),
            event_remote: OnceLock::new(),
            prompt_builtin: Arc::new(BuiltinPromptAssembler),
            prompt_builtin_v2: OnceLock::new(),
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

    #[must_use]
    pub fn agent_mcp_bridge(&self) -> Arc<AgentMcpBridge> {
        self.agent_mcp_bridge.clone()
    }

    pub(crate) fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        match backends.llm {
            LlmBackend::Ollama => self.llm_ollama.clone(),
            LlmBackend::Remote => self.llm_remote(),
            LlmBackend::Directory => self.llm_directory_slot(backends),
            LlmBackend::None => self.llm_none.clone(),
        }
    }

    pub fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        self.llm_for_plugin_backends(&PluginBackends {
            llm: b,
            ..Default::default()
        })
    }

    fn llm_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        self.resolve_directory_slot(
            "llm",
            backends,
            &self.directory_llm_cache,
            |s| &s.llm,
            self.llm_ollama.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), true);
                Arc::new(RemoteLlmHttp::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.high_risk_grants.clone(),
                    None,
                ))
            },
        )
    }

    pub(crate) fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        match backends.memory {
            MemoryBackend::Builtin => self.memory_builtin.clone(),
            MemoryBackend::BuiltinV2 => self.memory_builtin_v2(),
            MemoryBackend::Remote => self.memory_remote(),
            MemoryBackend::Local => self.memory_local_slot_for(backends),
            MemoryBackend::Directory => self.memory_directory_slot(backends),
            MemoryBackend::None => self.memory_none.clone(),
        }
    }

    pub fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
        self.memory_retrieval_for_plugin_backends(&PluginBackends {
            memory: b,
            ..Default::default()
        })
    }

    fn memory_local_slot_for(&self, backends: &PluginBackends) -> Arc<dyn MemoryRetrieval> {
        let providers = self
            .local_plugins
            .read()
            .providers_for_capability(LocalPluginCapability::Memory);
        let ids: Vec<&str> = providers.iter().map(|p| p.provider_id.as_str()).collect();
        let pick =
            pick_local_memory_provider_refs(ids, backends.local_memory_provider_id.as_deref());
        if pick.provider_id.is_none() {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=local but no registered local memory provider; ranking uses builtin_v2"
            );
        } else if pick.hint_missed {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.local_memory_provider_id={:?} not found among memory providers; using provider_id={}",
                backends.local_memory_provider_id,
                pick.provider_id.as_deref().unwrap_or("")
            );
        } else if pick.ambiguous_lexicographic {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=local with multiple memory providers; set plugin_backends.local_memory_provider_id; picked provider_id={}",
                pick.provider_id.as_deref().unwrap_or("")
            );
        }
        Arc::new(LocalPluginMemoryRetrieval::new(
            self.memory_builtin_v2(),
            pick.provider_id,
        ))
    }

    fn memory_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn MemoryRetrieval> {
        self.resolve_directory_slot(
            "memory",
            backends,
            &self.directory_memory_cache,
            |s| &s.memory,
            self.memory_builtin.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), false);
                Arc::new(RemoteMemoryRetrievalHttp::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.remote_fallback_allowed.clone(),
                    reg.high_risk_grants.clone(),
                    None,
                ))
            },
        )
    }

    pub(crate) fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        match backends.emotion {
            EmotionBackend::Builtin => self.emotion_builtin.clone(),
            EmotionBackend::BuiltinV2 => self.emotion_builtin_v2(),
            EmotionBackend::Remote => self.emotion_remote(),
            EmotionBackend::Directory => self.emotion_directory_slot(backends),
            EmotionBackend::None => self.emotion_none.clone(),
        }
    }

    pub fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        self.user_emotion_analyzer_for_backends(&PluginBackends {
            emotion: b,
            ..Default::default()
        })
    }

    fn emotion_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn UserEmotionAnalyzer> {
        self.resolve_directory_slot(
            "emotion",
            backends,
            &self.directory_emotion_cache,
            |s| &s.emotion,
            self.emotion_builtin.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), false);
                Arc::new(RemoteUserEmotionAnalyzerHttp::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.remote_fallback_allowed.clone(),
                    reg.high_risk_grants.clone(),
                    None,
                ))
            },
        )
    }

    pub(crate) fn event_estimator_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn EventEstimator> {
        match backends.event {
            EventBackend::Builtin => self.event_builtin.clone(),
            EventBackend::BuiltinV2 => self.event_builtin_v2(),
            EventBackend::Remote => self.event_remote(),
            EventBackend::Directory => self.event_directory_slot(backends),
            EventBackend::None => self.event_none.clone(),
        }
    }

    pub fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        self.event_estimator_for_backends(&PluginBackends {
            event: b,
            ..Default::default()
        })
    }

    fn event_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator> {
        self.resolve_directory_slot(
            "event",
            backends,
            &self.directory_event_cache,
            |s| &s.event,
            self.event_builtin.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), false);
                Arc::new(RemoteEventEstimatorHttp::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.remote_fallback_allowed.clone(),
                    reg.high_risk_grants.clone(),
                    None,
                ))
            },
        )
    }

    pub(crate) fn prompt_assembler_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler> {
        match backends.prompt {
            PromptBackend::Builtin => self.prompt_builtin.clone(),
            PromptBackend::BuiltinV2 => self.prompt_builtin_v2(),
            PromptBackend::Remote => self.prompt_remote(),
            PromptBackend::Directory => self.prompt_directory_slot(backends),
            PromptBackend::None => self.prompt_none.clone(),
        }
    }

    pub fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        self.prompt_assembler_for_backends(&PluginBackends {
            prompt: b,
            ..Default::default()
        })
    }

    fn prompt_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn PromptAssembler> {
        self.resolve_directory_slot(
            "prompt",
            backends,
            &self.directory_prompt_cache,
            |s| &s.prompt,
            self.prompt_builtin.clone(),
            |reg, _pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), false);
                Arc::new(RemotePromptAssemblerHttp::new(
                    reg.remote_http_client.clone(),
                    cfg,
                    reg.remote_fallback_allowed.clone(),
                    reg.high_risk_grants.clone(),
                    None,
                ))
            },
        )
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

    #[must_use]
    pub fn directory_runtime(&self) -> Option<Arc<DirectoryPluginRuntime>> {
        self.directory_runtime.clone()
    }
}
