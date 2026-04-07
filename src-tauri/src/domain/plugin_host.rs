//! 编译期可替换子系统宿主：按角色包 [`PluginBackends`](crate::models::PluginBackends) 选择具体实现。
//!
//! 与仓库 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 契约一致；`Remote` 在设置 `OCLIVE_REMOTE_*` 时走 HTTP JSON-RPC，否则回退内置。

use crate::domain::event_estimator::{
    BuiltinEventEstimator, BuiltinEventEstimatorV2, EventEstimator,
};
use crate::domain::memory_retrieval::{
    BuiltinMemoryRetrieval, BuiltinMemoryRetrievalV2, MemoryRetrieval,
};
use crate::domain::prompt_assembler::{
    BuiltinPromptAssembler, BuiltinPromptAssemblerV2, PromptAssembler,
};
use crate::domain::user_emotion_analyzer::{
    BuiltinUserEmotionAnalyzer, BuiltinUserEmotionAnalyzerV2, UserEmotionAnalyzer,
};
use crate::infrastructure::llm::LlmClient;
use crate::infrastructure::remote_plugin;
use crate::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend, Role,
};
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

/// 编译期插件实现集合（[`PluginHost::resolve_for_role`] 按枚举克隆 `Arc`）。
pub struct PluginHost {
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
}

impl PluginHost {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
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
        }
    }

    pub fn llm_for(&self, b: LlmBackend) -> Arc<dyn LlmClient> {
        match b {
            LlmBackend::Ollama => self.llm_ollama.clone(),
            LlmBackend::Remote => self.llm_remote.clone(),
        }
    }

    pub fn memory_retrieval(&self, b: MemoryBackend) -> Arc<dyn MemoryRetrieval> {
        match b {
            MemoryBackend::Builtin => self.memory_builtin.clone(),
            MemoryBackend::BuiltinV2 => self.memory_builtin_v2.clone(),
            MemoryBackend::Remote => self.memory_remote.clone(),
        }
    }

    pub fn user_emotion_analyzer(&self, b: EmotionBackend) -> Arc<dyn UserEmotionAnalyzer> {
        match b {
            EmotionBackend::Builtin => self.emotion_builtin.clone(),
            EmotionBackend::BuiltinV2 => self.emotion_builtin_v2.clone(),
            EmotionBackend::Remote => self.emotion_remote.clone(),
        }
    }

    pub fn event_estimator(&self, b: EventBackend) -> Arc<dyn EventEstimator> {
        match b {
            EventBackend::Builtin => self.event_builtin.clone(),
            EventBackend::BuiltinV2 => self.event_builtin_v2.clone(),
            EventBackend::Remote => self.event_remote.clone(),
        }
    }

    pub fn prompt_assembler(&self, b: PromptBackend) -> Arc<dyn PromptAssembler> {
        match b {
            PromptBackend::Builtin => self.prompt_builtin.clone(),
            PromptBackend::BuiltinV2 => self.prompt_builtin_v2.clone(),
            PromptBackend::Remote => self.prompt_remote.clone(),
        }
    }

    /// 解析当前角色包声明的全部后端（一次克隆五套 `Arc`，供整段对话复用）。
    pub fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins {
        let b = &role.plugin_backends;
        ResolvedRolePlugins {
            memory: self.memory_retrieval(b.memory),
            emotion: self.user_emotion_analyzer(b.emotion),
            event: self.event_estimator(b.event),
            prompt: self.prompt_assembler(b.prompt),
            llm: self.llm_for(b.llm),
        }
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
        EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend,
    };
    use std::sync::Arc;

    fn host() -> PluginHost {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        PluginHost::new(llm)
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
}
