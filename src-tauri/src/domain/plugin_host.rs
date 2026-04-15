//! 编译期可替换子系统宿主：按角色包 [`PluginBackends`](crate::models::PluginBackends) 选择具体实现。
//!
//! 与仓库 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 契约一致；`Remote` 在设置 `OCLIVE_REMOTE_*` 时走 HTTP JSON-RPC，否则回退内置。

use crate::domain::event_estimator::{
    BuiltinEventEstimator, BuiltinEventEstimatorV2, EventEstimator,
};
use crate::domain::local_plugin_bridge::{
    LocalPluginCapability, LocalPluginProviderDescriptor, LocalPluginRegistry,
};
use crate::domain::local_plugin_memory_pick::pick_local_memory_provider;
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
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_plugin::{
    self, RemoteEventEstimatorHttp, RemoteLlmHttp, RemoteMemoryRetrievalHttp,
    RemotePluginHttpConfig, RemotePromptAssemblerHttp, RemoteUserEmotionAnalyzerHttp,
};
use crate::models::{
    DirectoryPluginSlots, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PluginBackendsOverride, PromptBackend, Role,
};
use parking_lot::RwLock;
use std::sync::Arc;

/// 已按 `role.plugin_backends` 解析的实现句柄；单次 `send_message` 内应只解析一次并复用。
#[derive(Clone)]
pub struct ResolvedRolePlugins {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
    pub llm: Arc<dyn LlmClient>,
}

/// 后端注册表：管理 builtin / remote 插槽，并预留本地 provider 注册骨架。
pub struct BackendRegistry {
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
    fn from_runtime(
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
    ) -> Self {
        let llm_ollama = llm.clone();
        let llm_remote = remote_plugin::llm_remote_backend(llm);
        let rem = remote_plugin::plugin_remote_group();
        Self {
            memory_builtin: Arc::new(BuiltinMemoryRetrieval),
            memory_builtin_v2: Arc::new(BuiltinMemoryRetrievalV2),
            memory_remote: rem.memory,
            emotion_builtin: Arc::new(BuiltinUserEmotionAnalyzer),
            emotion_builtin_v2: Arc::new(BuiltinUserEmotionAnalyzerV2),
            emotion_remote: rem.emotion,
            event_builtin: Arc::new(BuiltinEventEstimator),
            event_builtin_v2: Arc::new(BuiltinEventEstimatorV2),
            event_remote: rem.event,
            prompt_builtin: Arc::new(BuiltinPromptAssembler),
            prompt_builtin_v2: Arc::new(BuiltinPromptAssemblerV2),
            prompt_remote: rem.prompt,
            llm_ollama,
            llm_remote,
            local_plugins: RwLock::new(LocalPluginRegistry::default()),
            directory_runtime,
        }
    }

    fn llm_for_plugin_backends(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        match backends.llm {
            LlmBackend::Ollama => self.llm_ollama.clone(),
            LlmBackend::Remote => self.llm_remote.clone(),
            LlmBackend::Directory => self.llm_directory_slot(backends),
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
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, true);
                Arc::new(RemoteLlmHttp::new(cfg))
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
        }
    }
}

/// 编译期插件实现集合（[`PluginHost::resolve_for_role`] 按枚举克隆 `Arc`）。
pub struct PluginHost {
    registry: BackendRegistry,
}

impl PluginHost {
    pub fn new(
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
    ) -> Self {
        Self {
            registry: BackendRegistry::from_runtime(llm, directory_runtime),
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
    use crate::infrastructure::llm::MockLlmClient;
    use crate::models::{
        EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
        PluginBackendsOverride, PromptBackend,
    };
    use std::sync::Arc;

    fn host() -> PluginHost {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        PluginHost::new(llm, None)
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
