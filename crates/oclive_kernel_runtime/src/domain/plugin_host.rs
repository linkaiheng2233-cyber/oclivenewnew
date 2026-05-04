//! 编译期可替换子系统宿主：按角色包 [`PluginBackends`](crate::models::PluginBackends) 选择具体实现。
//!
//! 与仓库 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 契约一致；`Remote` 在设置 `OCLIVE_REMOTE_*` 时走 HTTP JSON-RPC，否则回退内置。

#[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
use crate::domain::agent::BuiltinReActAgent;
#[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
use crate::domain::agent::McpShellAgent;
#[cfg(not(feature = "kernel-agent"))]
use crate::domain::agent::NoopAgent;
use crate::domain::agent::{AgentDebugTrace, AgentProvider, DisabledAgentProvider};
use crate::domain::complex_emotion::{
    default_complex_emotion_keyword_arc, ComplexEmotionProvider,
    DegradedToBuiltinComplexEmotionProvider, NoneComplexEmotionProvider,
};
use crate::domain::disabled_default_providers::{
    DisabledEventEstimator, DisabledUserEmotionAnalyzer, NoneMemoryRetrieval, NonePromptAssembler,
};
use crate::domain::event_estimator::{
    default_event_slot_v1, default_event_slot_v2, EventEstimator, RemoteEventEstimatorPlaceholder,
};
use crate::domain::local_plugin_bridge::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LocalPluginRegistry,
};
use crate::domain::local_plugin_memory_pick::pick_local_memory_provider;
use crate::domain::memory_retrieval::{
    default_memory_slot_v1, default_memory_slot_v2, LocalPluginMemoryRetrieval, MemoryRetrieval,
    RemoteMemoryRetrievalPlaceholder,
};
use crate::domain::prompt_assembler::{
    default_prompt_slot_v1, default_prompt_slot_v2, PromptAssembler,
    RemotePromptAssemblerPlaceholder,
};
use crate::domain::user_emotion_analyzer::{
    default_user_emotion_slot_v1, default_user_emotion_slot_v2,
    RemoteUserEmotionAnalyzerPlaceholder, UserEmotionAnalyzer,
};
use crate::infrastructure::cloud_llm::CloudLlmConfig;
use crate::infrastructure::db::DbManager;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::llm::{none_llm_client_arc, LlmClient, RemoteLlmPlaceholder};
#[cfg(feature = "kernel-agent")]
use crate::infrastructure::mcp_client::McpClient;
use crate::infrastructure::mcp_client::{McpServerManifest, McpToolCallResult};
use crate::infrastructure::remote_plugin::RemoteComplexEmotionHttp;
use crate::infrastructure::remote_plugin::{
    self, agent_remote_backend, PluginJsonRpcLlm, RemoteEventEstimatorHttp,
    RemoteMemoryRetrievalHttp, RemotePluginHttpConfig, RemotePromptAssemblerHttp,
    RemoteUserEmotionAnalyzerHttp,
};
use crate::models::{
    AgentBackend, ComplexEmotionBackend, DirectoryPluginSlots, EmotionBackend, EventBackend,
    LlmBackend, MemoryBackend, PluginBackends, PluginBackendsOverride, PromptBackend, Role,
};
#[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
use oclive_kernel_core::mcp::McpInvoke;
use parking_lot::RwLock;
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

/// 已按 `role.plugin_backends` 解析的实现句柄；单次 `send_message` 内应只解析一次并复用。
#[derive(Clone)]
pub struct ResolvedRolePlugins {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
    pub llm: Arc<dyn LlmClient>,
    pub agent: Arc<dyn AgentProvider>,
    pub complex_emotion: Arc<dyn ComplexEmotionProvider>,
}

/// 后端注册表：管理 builtin / remote 插槽，并预留本地 provider 注册骨架。
pub struct BackendRegistry {
    db_manager: Arc<DbManager>,
    memory_builtin: Arc<dyn MemoryRetrieval>,
    memory_builtin_v2: Arc<dyn MemoryRetrieval>,
    memory_remote: Arc<dyn MemoryRetrieval>,
    emotion_builtin: Arc<dyn UserEmotionAnalyzer>,
    emotion_builtin_v2: Arc<dyn UserEmotionAnalyzer>,
    emotion_remote: Arc<dyn UserEmotionAnalyzer>,
    event_builtin: Arc<dyn EventEstimator>,
    event_builtin_v2: Arc<dyn EventEstimator>,
    event_remote: Arc<dyn EventEstimator>,
    prompt_builtin: Arc<dyn PromptAssembler>,
    prompt_builtin_v2: Arc<dyn PromptAssembler>,
    prompt_remote: Arc<dyn PromptAssembler>,
    llm_ollama: Arc<dyn LlmClient>,
    llm_remote: Arc<dyn LlmClient>,
    llm_none: Arc<dyn LlmClient>,
    memory_none: Arc<dyn MemoryRetrieval>,
    emotion_none: Arc<dyn UserEmotionAnalyzer>,
    event_none: Arc<dyn EventEstimator>,
    prompt_none: Arc<dyn PromptAssembler>,
    agent_builtin: Arc<dyn AgentProvider>,
    #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
    agent_react: Arc<BuiltinReActAgent>,
    #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
    agent_mcp: Arc<McpShellAgent>,
    agent_remote: Arc<dyn AgentProvider>,
    agent_none: Arc<dyn AgentProvider>,
    complex_emotion_builtin: Arc<dyn ComplexEmotionProvider>,
    complex_emotion_remote: Arc<dyn ComplexEmotionProvider>,
    complex_emotion_none: Arc<dyn ComplexEmotionProvider>,
    local_plugins: RwLock<LocalPluginRegistry>,
    directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
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
    const REMOTE_PROVIDER_PLUGIN: &'static str = "system:remote_plugin_http";
    const REMOTE_PROVIDER_LLM: &'static str = "system:remote_llm_http";
    const REMOTE_PROVIDER_AGENT: &'static str = "system:remote_agent_http";
    const REMOTE_PROVIDER_COMPLEX_EMOTION: &'static str = "system:remote_complex_emotion_http";

    /// 将异步 DB 调用桥接到**同步**权限钩子（目录插件 manifest / Tauri 侧同步路径）。
    ///
    /// 使用 `block_in_place` 避免在异步 worker 上饿死同线程其它任务；**不宜**用于可改为 async API 的新代码。
    /// HTTP 出站见 `infrastructure::blocking_http`；市场索引 HTTP 已迁至原生 async。
    fn block_on<T>(&self, fut: impl Future<Output = T>) -> T {
        if let Ok(h) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| h.block_on(fut))
        } else {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build tokio runtime")
                .block_on(fut)
        }
    }

    fn check_directory_plugin_permission(&self, plugin_id: &str, permission: &str) -> bool {
        let ok = self.block_on(async {
            self.db_manager
                .is_plugin_permission_granted(plugin_id, permission)
                .await
                .unwrap_or(false)
        });
        let _ = self.block_on(async {
            self.db_manager
                .insert_plugin_audit_log(plugin_id, "permission.check", Some(permission), ok, "{}")
                .await
        });
        ok
    }

    fn check_remote_http_permission(&self, system_provider_id: &str) -> bool {
        let ok = self.block_on(async {
            self.db_manager
                .is_plugin_permission_granted(system_provider_id, "network:*")
                .await
                .unwrap_or(false)
        });
        let _ = self.block_on(async {
            self.db_manager
                .insert_plugin_audit_log(
                    system_provider_id,
                    "network.http",
                    Some("network:*"),
                    ok,
                    "{}",
                )
                .await
        });
        ok
    }

    fn complex_emotion_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn ComplexEmotionProvider> {
        match backends.complex_emotion {
            ComplexEmotionBackend::Builtin => self.complex_emotion_builtin.clone(),
            ComplexEmotionBackend::Remote => self.complex_emotion_remote.clone(),
            ComplexEmotionBackend::Directory => self.complex_emotion_directory_slot(backends),
            ComplexEmotionBackend::None => self.complex_emotion_none.clone(),
        }
    }

    fn complex_emotion_directory_slot(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn ComplexEmotionProvider> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.complex_emotion=directory but directory plugin runtime disabled; using builtin"
            );
            return self.complex_emotion_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.complex_emotion)
        else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.complex_emotion=directory but directory_plugins.complex_emotion missing; using builtin"
            );
            return self.complex_emotion_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.complex_emotion=directory but permission process:spawn not granted; using builtin"
            );
            return self.complex_emotion_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => Arc::new(RemoteComplexEmotionHttp::new(
                RemotePluginHttpConfig::for_directory_plugin_rpc(url, false),
            )),
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory complex_emotion plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.complex_emotion_builtin.clone()
            }
        }
    }

    fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        match backends.agent {
            AgentBackend::Builtin => self.agent_builtin.clone(),
            AgentBackend::Remote => self.agent_remote.clone(),
            AgentBackend::Directory => self.agent_directory_slot(backends),
            AgentBackend::None => self.agent_none.clone(),
        }
    }

    fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider> {
        self.agent_for_plugin_backends(&PluginBackends {
            agent: b,
            ..Default::default()
        })
    }

    fn agent_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.agent=directory but directory plugin runtime disabled"
            );
            return self.agent_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.agent) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.agent=directory but directory_plugins.agent missing"
            );
            return self.agent_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.agent=directory but permission process:spawn not granted; using builtin"
            );
            return self.agent_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, false);
                #[cfg(feature = "kernel-agent")]
                {
                    Arc::new(remote_plugin::RemoteAgentHttp::new(cfg))
                }
                #[cfg(not(feature = "kernel-agent"))]
                {
                    let _ = cfg;
                    self.agent_builtin.clone()
                }
            }
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory agent plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.agent_builtin.clone()
            }
        }
    }

    async fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        {
            self.agent_react.list_mcp_servers().await
        }
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        {
            self.agent_mcp.list_mcp_servers().await
        }
        #[cfg(not(feature = "kernel-agent"))]
        {
            Vec::new()
        }
    }

    async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        {
            self.agent_react
                .list_mcp_tools(server_id)
                .await
                .map_err(|e| e.to_frontend_error())
        }
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        {
            self.agent_mcp
                .list_mcp_tools(server_id)
                .await
                .map_err(|e| e.to_frontend_error())
        }
        #[cfg(not(feature = "kernel-agent"))]
        {
            let _ = server_id;
            Err("kernel-agent feature disabled".to_string())
        }
    }

    async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> std::result::Result<McpToolCallResult, String> {
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        {
            let sid = server_id.trim();
            if sid.is_empty() {
                return Err("server_id required".to_string());
            }
            let mut required_perm = "network:*";
            for s in self.list_mcp_servers().await {
                if s.id.trim() == sid {
                    if s.transport.trim().eq_ignore_ascii_case("stdio") {
                        required_perm = "process:spawn";
                    }
                    break;
                }
            }
            let mcp_provider_id = format!("system:mcp_server:{}", sid);
            if !self.check_directory_plugin_permission(mcp_provider_id.as_str(), required_perm) {
                return Err(format!(
                    "mcp server {} missing required permission {}",
                    sid, required_perm
                ));
            }
            self.agent_react
                .call_tool_direct(server_id, tool_name, params)
                .await
                .map_err(|e| e.to_frontend_error())
        }
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        {
            let sid = server_id.trim();
            if sid.is_empty() {
                return Err("server_id required".to_string());
            }
            let mut required_perm = "network:*";
            for s in self.list_mcp_servers().await {
                if s.id.trim() == sid {
                    if s.transport.trim().eq_ignore_ascii_case("stdio") {
                        required_perm = "process:spawn";
                    }
                    break;
                }
            }
            let mcp_provider_id = format!("system:mcp_server:{}", sid);
            if !self.check_directory_plugin_permission(mcp_provider_id.as_str(), required_perm) {
                return Err(format!(
                    "mcp server {} missing required permission {}",
                    sid, required_perm
                ));
            }
            self.agent_mcp
                .call_tool_direct(server_id, tool_name, params)
                .await
                .map_err(|e| e.to_frontend_error())
        }
        #[cfg(not(feature = "kernel-agent"))]
        {
            let _ = (server_id, tool_name, params);
            Err("kernel-agent feature disabled".to_string())
        }
    }

    fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        {
            self.agent_react.recent_traces()
        }
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        {
            self.agent_mcp.recent_traces()
        }
        #[cfg(not(feature = "kernel-agent"))]
        {
            Vec::new()
        }
    }

    fn clear_agent_traces(&self) {
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        {
            self.agent_react.clear_traces();
        }
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        {
            self.agent_mcp.clear_traces();
        }
    }

    fn from_runtime(
        db_manager: Arc<DbManager>,
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
        app_data_dir: PathBuf,
        cloud_llm_user: Arc<RwLock<Option<CloudLlmConfig>>>,
    ) -> Self {
        let llm_ollama = llm.clone();
        #[cfg(not(feature = "kernel-agent"))]
        let _ = &app_data_dir;
        #[cfg(feature = "kernel-agent")]
        let mcp = Arc::new(McpClient::new(app_data_dir));
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        let mcp_port: Arc<dyn McpInvoke> = mcp.clone();
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        let agent_react = Arc::new(BuiltinReActAgent::new(llm_ollama.clone(), mcp_port));
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        let agent_shell = Arc::new(McpShellAgent::new(mcp.clone()));
        #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
        let agent_builtin: Arc<dyn AgentProvider> = agent_react.clone();
        #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
        let agent_builtin: Arc<dyn AgentProvider> = agent_shell.clone();
        #[cfg(not(feature = "kernel-agent"))]
        let agent_builtin: Arc<dyn AgentProvider> = Arc::new(NoopAgent);

        let memory_none: Arc<dyn MemoryRetrieval> = Arc::new(NoneMemoryRetrieval);
        let emotion_none: Arc<dyn UserEmotionAnalyzer> = Arc::new(DisabledUserEmotionAnalyzer);
        let event_none: Arc<dyn EventEstimator> = Arc::new(DisabledEventEstimator);
        let prompt_none: Arc<dyn PromptAssembler> = Arc::new(NonePromptAssembler);
        let llm_none: Arc<dyn LlmClient> = none_llm_client_arc();

        // Remote HTTP sidecars are treated as "system providers". They require an explicit
        // permission grant (`network:*`) before they are even selected as providers.
        // If not granted, we fall back to builtin/placeholder providers, and keep behavior deterministic.
        let tmp = Self {
            db_manager: db_manager.clone(),
            memory_builtin: default_memory_slot_v1(),
            memory_builtin_v2: default_memory_slot_v2(),
            memory_remote: Arc::new(RemoteMemoryRetrievalPlaceholder::new()),
            emotion_builtin: default_user_emotion_slot_v1(),
            emotion_builtin_v2: default_user_emotion_slot_v2(),
            emotion_remote: Arc::new(RemoteUserEmotionAnalyzerPlaceholder::new()),
            event_builtin: default_event_slot_v1(),
            event_builtin_v2: default_event_slot_v2(),
            event_remote: Arc::new(RemoteEventEstimatorPlaceholder::new()),
            prompt_builtin: default_prompt_slot_v1(),
            prompt_builtin_v2: default_prompt_slot_v2(),
            prompt_remote: Arc::new(RemotePromptAssemblerPlaceholder::new()),
            llm_ollama: llm_ollama.clone(),
            llm_remote: Arc::new(RemoteLlmPlaceholder::new(llm_ollama.clone())),
            llm_none: llm_none.clone(),
            memory_none: memory_none.clone(),
            emotion_none: emotion_none.clone(),
            event_none: event_none.clone(),
            prompt_none: prompt_none.clone(),
            agent_builtin: agent_builtin.clone(),
            #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
            agent_react: agent_react.clone(),
            #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
            agent_mcp: agent_shell.clone(),
            agent_remote: agent_builtin.clone(),
            agent_none: Arc::new(DisabledAgentProvider),
            complex_emotion_builtin: default_complex_emotion_keyword_arc(),
            complex_emotion_remote: Arc::new(DegradedToBuiltinComplexEmotionProvider::new(
                "complex_emotion backend Remote is not connected; using builtin complex emotion",
            )),
            complex_emotion_none: Arc::new(NoneComplexEmotionProvider),
            local_plugins: RwLock::new(LocalPluginRegistry::default()),
            directory_runtime,
        };

        let rem = if tmp.check_remote_http_permission(Self::REMOTE_PROVIDER_PLUGIN) {
            remote_plugin::plugin_remote_group()
        } else {
            log::warn!(
                target: "oclive_plugin",
                "remote plugin HTTP configured but permission network:* not granted (provider_id={}); using placeholders",
                Self::REMOTE_PROVIDER_PLUGIN
            );
            remote_plugin::PluginRemoteGroup {
                memory: Arc::new(RemoteMemoryRetrievalPlaceholder::new()),
                emotion: Arc::new(RemoteUserEmotionAnalyzerPlaceholder::new()),
                event: Arc::new(RemoteEventEstimatorPlaceholder::new()),
                prompt: Arc::new(RemotePromptAssemblerPlaceholder::new()),
            }
        };

        let llm_remote: Arc<dyn LlmClient> = if tmp
            .check_remote_http_permission(Self::REMOTE_PROVIDER_LLM)
        {
            remote_plugin::llm_remote_backend(llm, cloud_llm_user)
        } else {
            log::warn!(
                target: "oclive_plugin",
                "remote LLM configured but permission network:* not granted (provider_id={}); using placeholder",
                Self::REMOTE_PROVIDER_LLM
            );
            Arc::new(RemoteLlmPlaceholder::new(llm_ollama.clone()))
        };

        let agent_remote: Arc<dyn AgentProvider> = if tmp
            .check_remote_http_permission(Self::REMOTE_PROVIDER_AGENT)
        {
            #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
            {
                agent_remote_backend(agent_react.clone() as Arc<dyn AgentProvider>)
            }
            #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
            {
                agent_remote_backend(agent_shell.clone())
            }
            #[cfg(not(feature = "kernel-agent"))]
            {
                agent_remote_backend(agent_builtin.clone())
            }
        } else {
            log::warn!(
                target: "oclive_plugin",
                "remote Agent configured but permission network:* not granted (provider_id={}); using builtin",
                Self::REMOTE_PROVIDER_AGENT
            );
            agent_builtin.clone()
        };

        let complex_emotion_remote: Arc<dyn ComplexEmotionProvider> = if tmp
            .check_remote_http_permission(Self::REMOTE_PROVIDER_COMPLEX_EMOTION)
        {
            remote_plugin::complex_emotion_remote_backend()
        } else {
            log::warn!(
                target: "oclive_plugin",
                "remote complex_emotion configured but permission network:* not granted (provider_id={}); using degraded builtin",
                Self::REMOTE_PROVIDER_COMPLEX_EMOTION
            );
            Arc::new(DegradedToBuiltinComplexEmotionProvider::new(
                "complex_emotion backend Remote is not connected; using builtin complex emotion",
            ))
        };

        let complex_emotion_builtin: Arc<dyn ComplexEmotionProvider> =
            default_complex_emotion_keyword_arc();
        let complex_emotion_none: Arc<dyn ComplexEmotionProvider> =
            Arc::new(NoneComplexEmotionProvider);
        Self {
            db_manager,
            memory_builtin: default_memory_slot_v1(),
            memory_builtin_v2: default_memory_slot_v2(),
            memory_remote: rem.memory,
            emotion_builtin: default_user_emotion_slot_v1(),
            emotion_builtin_v2: default_user_emotion_slot_v2(),
            emotion_remote: rem.emotion,
            event_builtin: default_event_slot_v1(),
            event_builtin_v2: default_event_slot_v2(),
            event_remote: rem.event,
            prompt_builtin: default_prompt_slot_v1(),
            prompt_builtin_v2: default_prompt_slot_v2(),
            prompt_remote: rem.prompt,
            llm_ollama,
            llm_remote,
            llm_none,
            memory_none,
            emotion_none,
            event_none,
            prompt_none,
            agent_builtin,
            #[cfg(all(feature = "kernel-agent", feature = "default-agent-providers"))]
            agent_react,
            #[cfg(all(feature = "kernel-agent", not(feature = "default-agent-providers")))]
            agent_mcp: agent_shell,
            agent_remote,
            agent_none: Arc::new(DisabledAgentProvider),
            complex_emotion_builtin,
            complex_emotion_remote,
            complex_emotion_none,
            local_plugins: RwLock::new(LocalPluginRegistry::default()),
            directory_runtime: tmp.directory_runtime,
        }
    }

    fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        match backends.llm {
            LlmBackend::Ollama => self.llm_ollama.clone(),
            LlmBackend::Remote => self.llm_remote.clone(),
            LlmBackend::Directory => self.llm_directory_slot(backends),
            LlmBackend::None => self.llm_none.clone(),
        }
    }

    fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        self.llm_for_plugin_backends(&PluginBackends {
            llm: b,
            ..Default::default()
        })
    }

    fn llm_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.llm=directory but directory plugin runtime disabled"
            );
            return self.llm_ollama.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.llm) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.llm=directory but directory_plugins.llm missing"
            );
            return self.llm_ollama.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.llm=directory but permission process:spawn not granted; using ollama"
            );
            return self.llm_ollama.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, true);
                Arc::new(PluginJsonRpcLlm::new(cfg))
            }
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory llm plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.llm_ollama.clone()
            }
        }
    }

    fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        match backends.memory {
            MemoryBackend::Builtin => self.memory_builtin.clone(),
            MemoryBackend::BuiltinV2 => self.memory_builtin_v2.clone(),
            MemoryBackend::Remote => self.memory_remote.clone(),
            MemoryBackend::Local => self.memory_local_slot_for(backends),
            MemoryBackend::Directory => self.memory_directory_slot(backends),
            MemoryBackend::None => self.memory_none.clone(),
        }
    }

    fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
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
        let ids: Vec<String> = providers.iter().map(|p| p.provider_id.clone()).collect();
        let pick = pick_local_memory_provider(ids, backends.local_memory_provider_id.as_deref());
        if pick.provider_id.is_none() {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=local but no registered local memory provider; ranking uses builtin_v2"
            );
        } else if pick.hint_missed {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.local_memory_provider_id={:?} not found among memory providers; using provider_id={}",
                backends.local_memory_provider_id,
                pick.provider_id.as_deref().unwrap_or("")
            );
        } else if pick.ambiguous_lexicographic {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=local with multiple memory providers; set plugin_backends.local_memory_provider_id; picked provider_id={}",
                pick.provider_id.as_deref().unwrap_or("")
            );
        }
        Arc::new(LocalPluginMemoryRetrieval::new(
            self.memory_builtin_v2.clone(),
            pick.provider_id,
        ))
    }

    fn memory_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn MemoryRetrieval> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=directory but directory plugin runtime disabled; using builtin"
            );
            return self.memory_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.memory) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=directory but directory_plugins.memory missing; using builtin"
            );
            return self.memory_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.memory=directory but permission process:spawn not granted; using builtin"
            );
            return self.memory_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => Arc::new(RemoteMemoryRetrievalHttp::new(
                RemotePluginHttpConfig::for_directory_plugin_rpc(url, false),
            )),
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory memory plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.memory_builtin.clone()
            }
        }
    }

    fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        match backends.emotion {
            EmotionBackend::Builtin => self.emotion_builtin.clone(),
            EmotionBackend::BuiltinV2 => self.emotion_builtin_v2.clone(),
            EmotionBackend::Remote => self.emotion_remote.clone(),
            EmotionBackend::Directory => self.emotion_directory_slot(backends),
            EmotionBackend::None => self.emotion_none.clone(),
        }
    }

    fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        self.user_emotion_analyzer_for_backends(&PluginBackends {
            emotion: b,
            ..Default::default()
        })
    }

    fn emotion_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn UserEmotionAnalyzer> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.emotion=directory but directory plugin runtime disabled; using builtin"
            );
            return self.emotion_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.emotion) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.emotion=directory but directory_plugins.emotion missing; using builtin"
            );
            return self.emotion_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.emotion=directory but permission process:spawn not granted; using builtin"
            );
            return self.emotion_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => Arc::new(RemoteUserEmotionAnalyzerHttp::new(
                RemotePluginHttpConfig::for_directory_plugin_rpc(url, false),
            )),
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory emotion plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.emotion_builtin.clone()
            }
        }
    }

    fn event_estimator_for_backends(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator> {
        match backends.event {
            EventBackend::Builtin => self.event_builtin.clone(),
            EventBackend::BuiltinV2 => self.event_builtin_v2.clone(),
            EventBackend::Remote => self.event_remote.clone(),
            EventBackend::Directory => self.event_directory_slot(backends),
            EventBackend::None => self.event_none.clone(),
        }
    }

    fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        self.event_estimator_for_backends(&PluginBackends {
            event: b,
            ..Default::default()
        })
    }

    fn event_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn EventEstimator> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.event=directory but directory plugin runtime disabled; using builtin"
            );
            return self.event_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.event) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.event=directory but directory_plugins.event missing; using builtin"
            );
            return self.event_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.event=directory but permission process:spawn not granted; using builtin"
            );
            return self.event_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => Arc::new(RemoteEventEstimatorHttp::new(
                RemotePluginHttpConfig::for_directory_plugin_rpc(url, false),
            )),
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory event plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.event_builtin.clone()
            }
        }
    }

    fn prompt_assembler_for_backends(&self, backends: &PluginBackends) -> Arc<dyn PromptAssembler> {
        match backends.prompt {
            PromptBackend::Builtin => self.prompt_builtin.clone(),
            PromptBackend::BuiltinV2 => self.prompt_builtin_v2.clone(),
            PromptBackend::Remote => self.prompt_remote.clone(),
            PromptBackend::Directory => self.prompt_directory_slot(backends),
            PromptBackend::None => self.prompt_none.clone(),
        }
    }

    fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        self.prompt_assembler_for_backends(&PluginBackends {
            prompt: b,
            ..Default::default()
        })
    }

    fn prompt_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn PromptAssembler> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.prompt=directory but directory plugin runtime disabled; using builtin"
            );
            return self.prompt_builtin.clone();
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, |s| &s.prompt) else {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.prompt=directory but directory_plugins.prompt missing; using builtin"
            );
            return self.prompt_builtin.clone();
        };
        if !self.check_directory_plugin_permission(pid.as_str(), "process:spawn") {
            log::warn!(
                target: "oclive_plugin",
                "plugin_backends.prompt=directory but permission process:spawn not granted; using builtin"
            );
            return self.prompt_builtin.clone();
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => Arc::new(RemotePromptAssemblerHttp::new(
                RemotePluginHttpConfig::for_directory_plugin_rpc(url, false),
            )),
            Err(e) => {
                log::error!(
                    target: "oclive_plugin",
                    "directory prompt plugin_id={} spawn failed: {}",
                    pid,
                    e
                );
                self.prompt_builtin.clone()
            }
        }
    }

    pub fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String> {
        self.local_plugins.write().register_provider(descriptor)
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
}

/// 解析层：将角色包默认后端 + 可选会话覆盖合成为有效后端并绑定实现。
pub struct PluginResolver;

impl PluginResolver {
    fn resolve(
        registry: &BackendRegistry,
        role_backends: &PluginBackends,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        let effective = match session_override {
            Some(ov) => ov.apply_to(role_backends),
            None => role_backends.clone(),
        };
        ResolvedRolePlugins {
            memory: registry.memory_retrieval_for_plugin_backends(&effective),
            emotion: registry.user_emotion_analyzer_for_backends(&effective),
            event: registry.event_estimator_for_backends(&effective),
            prompt: registry.prompt_assembler_for_backends(&effective),
            llm: registry.llm_for_plugin_backends(&effective),
            agent: registry.agent_for_plugin_backends(&effective),
            complex_emotion: registry.complex_emotion_for_plugin_backends(&effective),
        }
    }
}

/// 编译期插件实现集合（[`PluginHost::resolve_for_role`] 按枚举克隆 `Arc`）。
pub struct PluginHost {
    registry: BackendRegistry,
}

impl PluginHost {
    pub fn new(
        db_manager: Arc<DbManager>,
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
        app_data_dir: PathBuf,
        cloud_llm_user: Arc<RwLock<Option<CloudLlmConfig>>>,
    ) -> Self {
        Self {
            registry: BackendRegistry::from_runtime(
                db_manager,
                llm,
                directory_runtime,
                app_data_dir,
                cloud_llm_user,
            ),
        }
    }

    pub fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), String> {
        self.registry.register_local_provider(descriptor)
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

    pub fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        self.registry.llm_for(b)
    }

    pub fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        self.registry.llm_for_plugin_backends(backends)
    }

    pub fn agent_for(&self, b: AgentBackend) -> Arc<dyn AgentProvider> {
        self.registry.agent_for(b)
    }

    pub fn agent_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn AgentProvider> {
        self.registry.agent_for_plugin_backends(backends)
    }

    pub fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        self.registry.memory_retrieval_for_plugin_backends(backends)
    }

    pub fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
        self.registry.memory_retrieval(b)
    }

    pub fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        self.registry.user_emotion_analyzer(b)
    }

    pub fn user_emotion_analyzer_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        self.registry.user_emotion_analyzer_for_backends(backends)
    }

    pub fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        self.registry.event_estimator(b)
    }

    pub fn event_estimator_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn EventEstimator> {
        self.registry.event_estimator_for_backends(backends)
    }

    pub fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        self.registry.prompt_assembler(b)
    }

    pub fn prompt_assembler_for_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler> {
        self.registry.prompt_assembler_for_backends(backends)
    }

    #[must_use]
    pub async fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        self.registry.list_mcp_servers().await
    }

    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
        self.registry.list_mcp_tools(server_id).await
    }

    pub async fn call_mcp_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> std::result::Result<McpToolCallResult, String> {
        self.registry
            .call_mcp_tool(server_id, tool_name, params)
            .await
    }

    #[must_use]
    pub fn recent_agent_traces(&self) -> Vec<AgentDebugTrace> {
        self.registry.recent_agent_traces()
    }

    pub fn clear_agent_traces(&self) {
        self.registry.clear_agent_traces();
    }

    /// 解析当前角色包声明的全部后端（一次克隆五套 `Arc`，供整段对话复用）。
    pub fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins {
        PluginResolver::resolve(&self.registry, &role.plugin_backends, None)
    }

    /// 解析角色默认后端 + 会话级覆盖（覆盖为空时等价于 [`Self::resolve_for_role`]）。
    pub fn resolve_for_role_with_override(
        &self,
        role: &Role,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginResolver::resolve(&self.registry, &role.plugin_backends, session_override)
    }
}

impl ResolvedRolePlugins {
    /// 与 `role.plugin_backends` 一致，便于日志/测试断言。
    pub fn backends_snapshot(role: &Role) -> PluginBackends {
        role.plugin_backends.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::DbManager;
    use crate::infrastructure::llm::MockLlmClient;
    use crate::models::{
        EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
        PluginBackendsOverride, PromptBackend,
    };
    use std::sync::Arc;

    fn test_db_manager() -> Arc<DbManager> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool =
            rt.block_on(async { sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap() });
        rt.block_on(async {
            let _ = sqlx::query(
                "CREATE TABLE IF NOT EXISTS plugin_permission_grants (
                    plugin_id TEXT NOT NULL,
                    permission TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 0,
                    granted_by TEXT NOT NULL DEFAULT 'test',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    PRIMARY KEY (plugin_id, permission)
                );",
            )
            .execute(&pool)
            .await;
            let _ = sqlx::query(
                "CREATE TABLE IF NOT EXISTS plugin_audit_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    plugin_id TEXT NOT NULL,
                    action TEXT NOT NULL,
                    permission TEXT,
                    allowed INTEGER NOT NULL,
                    meta_json TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );",
            )
            .execute(&pool)
            .await;
        });
        Arc::new(DbManager::new(pool))
    }

    fn host() -> PluginHost {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        let db = test_db_manager();
        PluginHost::new(
            db,
            llm,
            None,
            std::env::temp_dir(),
            Arc::new(RwLock::new(None)),
        )
    }

    #[test]
    fn resolve_matches_role_plugin_backends_default() {
        let role = Role::default();
        assert_eq!(
            ResolvedRolePlugins::backends_snapshot(&role),
            role.plugin_backends
        );
        host().resolve_for_role(&role);
    }

    #[test]
    fn resolve_selects_memory_v2_when_configured() {
        let role = Role {
            plugin_backends: PluginBackends {
                memory: MemoryBackend::BuiltinV2,
                emotion: EmotionBackend::Builtin,
                event: EventBackend::Builtin,
                prompt: PromptBackend::Builtin,
                llm: LlmBackend::Ollama,
                ..Default::default()
            },
            ..Default::default()
        };
        let h = host();
        let pl = h.resolve_for_role(&role);
        let same_again = h.memory_retrieval(MemoryBackend::BuiltinV2);
        // 同一 `PluginHost` 内：resolve 与显式取槽应为同一 `Arc` 指针
        assert!(Arc::ptr_eq(&pl.memory, &same_again));
    }

    #[test]
    fn resolve_selects_emotion_v2_when_configured() {
        let role = Role {
            plugin_backends: PluginBackends {
                emotion: EmotionBackend::BuiltinV2,
                ..Default::default()
            },
            ..Default::default()
        };
        let h = host();
        let pl = h.resolve_for_role(&role);
        let slot = h.user_emotion_analyzer(EmotionBackend::BuiltinV2);
        assert!(Arc::ptr_eq(&pl.emotion, &slot));
    }

    #[test]
    fn resolve_with_override_prefers_session_backend() {
        let role = Role::default();
        let override_backends = PluginBackendsOverride {
            memory: Some(MemoryBackend::BuiltinV2),
            llm: Some(LlmBackend::Remote),
            ..Default::default()
        };
        let h = host();
        let pl = h.resolve_for_role_with_override(&role, Some(&override_backends));
        let mem_slot = h.memory_retrieval(MemoryBackend::BuiltinV2);
        let llm_slot = h.llm_for(LlmBackend::Remote);
        assert!(Arc::ptr_eq(&pl.memory, &mem_slot));
        assert!(Arc::ptr_eq(&pl.llm, &llm_slot));
    }

    #[test]
    fn register_local_provider_tracks_capability() {
        let h = host();
        h.register_local_provider(LocalPluginProviderDescriptor {
            provider_id: "local.demo".to_string(),
            schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
            min_runtime_version: None,
            capabilities: vec![LocalPluginCapability::Prompt],
        })
        .expect("register local provider");
        assert_eq!(
            h.local_providers_for(LocalPluginCapability::Prompt).len(),
            1
        );
        assert_eq!(
            h.local_providers_for(LocalPluginCapability::Memory).len(),
            0
        );
    }

    #[test]
    fn memory_local_resolves_and_ranks_like_v2_when_provider_registered() {
        let h = host();
        h.register_local_provider(LocalPluginProviderDescriptor {
            provider_id: "mem.local.one".to_string(),
            schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
            min_runtime_version: None,
            capabilities: vec![LocalPluginCapability::Memory],
        })
        .expect("register");
        let role = Role {
            plugin_backends: PluginBackends {
                memory: MemoryBackend::Local,
                ..Default::default()
            },
            ..Default::default()
        };
        let pl = h.resolve_for_role(&role);
        let v2 = h.memory_retrieval(MemoryBackend::BuiltinV2);
        use crate::domain::memory_retrieval::MemoryRetrievalInput;
        use crate::models::Memory;
        use chrono::Utc;
        let t = Utc::now();
        let m = Memory {
            id: "x".into(),
            role_id: "r".into(),
            content: "hello".into(),
            importance: 1.0,
            weight: 1.0,
            created_at: t,
            scene_id: None,
        };
        let slice = &[m];
        let mk = || MemoryRetrievalInput {
            memories: slice,
            user_query: "hello",
            scene_id: None,
            limit: 3,
        };
        assert_eq!(
            pl.memory.diagnostic_local_provider_id(),
            Some("mem.local.one")
        );
        let a: Vec<_> = pl
            .memory
            .rank_memories(mk())
            .into_iter()
            .map(|m| m.id)
            .collect();
        let b: Vec<_> = v2.rank_memories(mk()).into_iter().map(|m| m.id).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn memory_local_hint_selects_named_provider() {
        let h = host();
        for id in ["mem.a", "mem.z"] {
            h.register_local_provider(LocalPluginProviderDescriptor {
                provider_id: id.to_string(),
                schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .expect("register");
        }
        let role = Role {
            plugin_backends: PluginBackends {
                memory: MemoryBackend::Local,
                local_memory_provider_id: Some("mem.z".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let pl = h.resolve_for_role(&role);
        assert_eq!(pl.memory.diagnostic_local_provider_id(), Some("mem.z"));
    }
}
