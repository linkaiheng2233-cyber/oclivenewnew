use crate::domain::agent::{AgentDebugTrace, AgentProvider, BuiltinReActAgent};
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
use crate::domain::prompt_assembler::{
    BuiltinPromptAssembler, BuiltinPromptAssemblerV2, PromptAssembler,
};
use crate::domain::user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, BuiltinUserEmotionAnalyzerV2, UserEmotionAnalyzer,
};
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::domain::ports::LlmClient;
use crate::infrastructure::mcp_client::{McpClient, McpServerManifest, McpToolCallResult};
use crate::infrastructure::remote_plugin::{
    self, PluginRemoteGroup, RemoteEventEstimatorHttp, RemoteLlmHttp, RemoteMemoryRetrievalHttp,
    RemotePluginHttpConfig, RemotePromptAssemblerHttp, RemoteUserEmotionAnalyzerHttp,
};
use crate::models::{
    AgentBackend, DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackends, PromptBackend,
};
use parking_lot::RwLock;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};

use super::PluginHostError;

/// 后端注册表：管理 builtin / remote 插槽，并预留本地 provider 注册骨架。
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
    llm_ollama: Arc<dyn LlmClient>,
    llm_remote: OnceLock<Arc<dyn LlmClient>>,
    agent_builtin: Arc<BuiltinReActAgent>,
    agent_remote: OnceLock<Arc<dyn AgentProvider>>,
    agent_directory: Arc<dyn AgentProvider>,
    remote_plugin_group: OnceLock<PluginRemoteGroup>,
    local_plugins: RwLock<LocalPluginRegistry>,
    directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
    remote_http_client: Arc<reqwest::Client>,
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
            .get_or_init(|| self.agent_builtin.clone())
            .clone()
    }

    fn resolve_directory_slot<T, Pick, Build>(
        &self,
        module: &'static str,
        backends: &PluginBackends,
        pick: Pick,
        fallback: T,
        build: Build,
    ) -> T
    where
        Pick: FnOnce(&DirectoryPluginSlots) -> &Option<String>,
        Build: FnOnce(&Self, &str, &str) -> T,
        T: Clone,
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
        match _rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => build(self, pid.as_str(), url.as_str()),
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
            AgentBackend::Directory => self.agent_directory.clone(),
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
    pub fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
        crate::map_frontend_err!(self.agent_builtin.list_mcp_tools(server_id))
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> std::result::Result<McpToolCallResult, String> {
        crate::map_frontend_err!(self.agent_builtin.call_tool_direct(server_id, tool_name, params))
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
        let agent_builtin = Arc::new(BuiltinReActAgent::new(llm_ollama.clone(), mcp));
        let agent_directory: Arc<dyn AgentProvider> = agent_builtin.clone();
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
            llm_ollama,
            llm_remote: OnceLock::new(),
            agent_builtin,
            agent_remote: OnceLock::new(),
            agent_directory,
            remote_plugin_group: OnceLock::new(),
            local_plugins: RwLock::new(LocalPluginRegistry::default()),
            directory_runtime,
            remote_fallback_allowed,
            high_risk_grants,
            remote_http_client,
        }
    }

    pub(crate) fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        match backends.llm {
            LlmBackend::Ollama => self.llm_ollama.clone(),
            LlmBackend::Remote => self.llm_remote(),
            LlmBackend::Directory => self.llm_directory_slot(backends),
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
        let ids: Vec<&str> = providers
            .iter()
            .map(|p| p.provider_id.as_str())
            .collect();
        let pick = pick_local_memory_provider_refs(ids, backends.local_memory_provider_id.as_deref());
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
