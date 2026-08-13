//! Split from backend_registry.rs (zero semantic change, facade in mod.rs).

use crate::domain::event_estimator::EventEstimator;
use crate::domain::ports::LlmClient;
use crate::infrastructure::remote_plugin::{
    self, RemoteEventEstimatorHttp, RemoteLlmHttp, RemoteMemoryRetrievalHttp,
    RemotePluginHttpConfig, RemotePromptAssemblerHttp, RemoteUserEmotionAnalyzerHttp,
    METHOD_LLM_GENERATE_STREAM,
};
use crate::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend,
};
use oclive_kernel_runtime::domain::local_plugin_bridge::LocalPluginCapability;
use oclive_kernel_runtime::domain::local_plugin_memory_pick::pick_local_memory_provider_refs;
use oclive_kernel_runtime::domain::memory_retrieval::{
    LocalPluginMemoryRetrieval, MemoryRetrieval,
};
use oclive_kernel_runtime::domain::prompt_assembler::PromptAssembler;
use oclive_kernel_runtime::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use std::sync::Arc;

use super::BackendRegistry;

impl BackendRegistry {
    pub(super) fn memory_remote(&self) -> Arc<dyn MemoryRetrieval> {
        self.memory_remote
            .get_or_init(|| self.remote_plugin_group().memory.clone())
            .clone()
    }

    pub(super) fn emotion_remote(&self) -> Arc<dyn UserEmotionAnalyzer> {
        self.emotion_remote
            .get_or_init(|| self.remote_plugin_group().emotion.clone())
            .clone()
    }

    pub(super) fn event_remote(&self) -> Arc<dyn EventEstimator> {
        self.event_remote
            .get_or_init(|| self.remote_plugin_group().event.clone())
            .clone()
    }

    pub(super) fn prompt_remote(&self) -> Arc<dyn PromptAssembler> {
        self.prompt_remote
            .get_or_init(|| self.remote_plugin_group().prompt.clone())
            .clone()
    }

    pub(super) fn llm_remote(&self) -> Arc<dyn LlmClient> {
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

    pub(super) fn llm_directory_slot(&self, backends: &PluginBackends) -> Arc<dyn LlmClient> {
        self.pick_directory_slot(
            "llm",
            backends,
            &self.directory_llm_cache,
            |s| &s.llm,
            self.llm_ollama.clone(),
            |reg, pid, url| {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url.to_string(), true);
                let native_stream = reg.directory_runtime_for_slots().is_some_and(|runtime| {
                    runtime.manifest_declares_rpc_method(pid, METHOD_LLM_GENERATE_STREAM)
                });
                Arc::new(
                    RemoteLlmHttp::new(
                        reg.remote_http_client.clone(),
                        cfg,
                        reg.high_risk_grants.clone(),
                        None,
                    )
                    .with_native_stream(native_stream),
                )
            },
        )
    }

    pub(crate) fn memory_retrieval_for_plugin_backends(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        match backends.memory {
            MemoryBackend::Builtin => self.memory_builtin.clone(),
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

    pub(super) fn memory_local_slot_for(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
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
                "plugin_backends.memory=local but no registered local memory provider; ranking uses builtin"
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
            self.memory_builtin.clone(),
            pick.provider_id,
        ))
    }

    pub(super) fn memory_directory_slot(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn MemoryRetrieval> {
        self.pick_directory_slot(
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

    pub(super) fn emotion_directory_slot(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn UserEmotionAnalyzer> {
        self.pick_directory_slot(
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

    pub(super) fn event_directory_slot(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn EventEstimator> {
        self.pick_directory_slot(
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

    pub(super) fn prompt_directory_slot(
        &self,
        backends: &PluginBackends,
    ) -> Arc<dyn PromptAssembler> {
        self.pick_directory_slot(
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
}
