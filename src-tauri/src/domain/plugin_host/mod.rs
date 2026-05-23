//! # 插件装配中心（`PluginHost`）
//!
//! **角色**：把角色包配置（`plugin_backends`、`slot_registry`、目录插件 manifest）解析为可执行的 **`Arc<dyn …>`** 句柄集合（`ResolvedRolePlugins`），供编排层通过 [`PluginHostPort`](crate::domain::ports::PluginHostPort) 消费。
//!
//! **上游**：`RoleStorage` 加载的 `Role`；会话覆盖来自 `AppState`；`BackendRegistry` 缓存 builtin / remote / directory 构造器。
//! **下游**：`chat_engine`、`SlotResolver::resolve`；实现 [`PluginHostPort`](crate::domain::ports::PluginHostPort) 以解耦具体 `PluginHost` 类型。
//!
//! **关键决策**：编排只依赖 **trait 对象**，桌面 / 无头 / 测试可替换宿主；Remote 未配置 env 时**降级 + 日志**，避免静默失败。契约见 `creator-docs/plugin-and-architecture/PLUGIN_V1.md`。
//!
//! **Clone 策略（审计 2026-05）**
//!
//! - **`Arc::clone` / `Option<Arc<_>>::clone`**：仅增减引用计数，resolve 热路径上的 backend 句柄均属此类。
//! - **`PluginBackends` 结构体 clone**：仅在会话覆盖 [`PluginBackendsOverride::apply_to`] 时分配；无覆盖时借用包内默认配置（见 [`PluginResolver::resolve`]）。
//! - **`provider_id` 字符串**：local memory 路径用 [`pick_local_memory_provider_refs`]，仅克隆最终选中的 id。
//! - **`directory_slot_id`**：目录插件 id 需 owned `String` 供 RPC 启动，无法避免单次分配。

use crate::domain::agent::{AgentDebugTrace, AgentProvider};
use crate::domain::complex_emotion::ComplexEmotionProvider;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::local_plugin_bridge::{
    LocalPluginCapability, LocalPluginProviderDescriptor,
};
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::slot_resolver::ResolvedRoleSlots;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::domain::ports::LlmClient;
use crate::infrastructure::mcp_client::{McpServerManifest, McpToolCallResult};
use crate::models::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend,
    PluginBackends, PluginBackendsOverride, PromptBackend, Role,
};
use oclive_validation::SlotRegistryEntry;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thiserror::Error;

/// [`PluginHost`] / [`BackendRegistry`] 在解析与注册本地 provider 时的错误。
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

/// 已按 `role.plugin_backends` 解析的实现句柄；单次 `send_message` 内应只解析一次并复用。
#[derive(Clone)]
pub struct ResolvedRolePlugins {
    pub memory: Arc<dyn MemoryRetrieval>,
    pub emotion: Arc<dyn UserEmotionAnalyzer>,
    pub event: Arc<dyn EventEstimator>,
    pub prompt: Arc<dyn PromptAssembler>,
    pub llm: Arc<dyn LlmClient>,
    pub agent: Arc<dyn AgentProvider>,
    /// 蓝图 `complex_emotion` 槽 last-wins 解析（无 registry 时为 builtin）。
    pub complex_emotion: Arc<dyn ComplexEmotionProvider>,
    /// 按实例解析的多槽视图（P3；P4 编排串行合并用）。
    pub slots: Option<ResolvedRoleSlots>,
    /// 多 `agent` directory 槽合并的插件 id（观测 / P4）。
    pub merged_agent_directory_plugin_ids: Vec<String>,
}


mod registry;
mod resolver;

pub use registry::BackendRegistry;
pub use resolver::PluginResolver;

/// 编译期插件实现集合（[`PluginHost::resolve_for_role`] 按枚举克隆 `Arc`）。
pub struct PluginHost {
    registry: BackendRegistry,
}

impl PluginHost {
    /// 构造宿主注册表。
    ///
    /// - `llm`：进程内默认 LLM 句柄（`plugin_backends.llm = ollama` 等会复用或包装该实现）。
    /// - `directory_runtime`：目录插件懒启动运行时；无目录插件需求时可传 `None`。
    /// - `app_data_dir`：应用数据根目录（生产环境为 Tauri app data）。当前用于初始化
    ///   [`McpClient`](crate::infrastructure::mcp_client::McpClient)（扫描 `{app_data_dir}/mcp-servers/*.json`）。
    ///   集成测试可传 `std::env::temp_dir()`。
    /// - `high_risk_grants`：MCP 传输与目录插件子进程等显式授权（`{app_data}/high_risk_grants.json`）。
    pub fn new(
        llm: Arc<dyn LlmClient>,
        directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
        app_data_dir: PathBuf,
        high_risk_grants: Arc<HighRiskGrantStore>,
        remote_fallback_allowed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            registry: BackendRegistry::from_runtime(
                llm,
                directory_runtime,
                app_data_dir,
                high_risk_grants,
                remote_fallback_allowed,
            ),
        }
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn register_local_provider(
        &self,
        descriptor: LocalPluginProviderDescriptor,
    ) -> Result<(), PluginHostError> {
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
    pub fn list_mcp_servers(&self) -> Vec<McpServerManifest> {
        self.registry.list_mcp_servers()
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    pub fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
        self.registry.list_mcp_tools(server_id)
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
        self.registry.call_mcp_tool(server_id, tool_name, params)
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
        PluginResolver::resolve(
            &self.registry,
            &role.plugin_backends,
            None,
            role.slot_registry.as_ref(),
        )
    }

    /// 解析角色默认后端 + 会话级覆盖（覆盖为空时等价于 [`Self::resolve_for_role`]）。
    pub fn resolve_for_role_with_override(
        &self,
        role: &Role,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            &self.registry,
            &role.plugin_backends,
            session_override,
            role.slot_registry.as_ref(),
        )
    }

    /// 有效六槽 + 蓝图 registry + 可选六槽会话覆盖（v2 热路径）。
    pub fn resolve_for_effective_backends(
        &self,
        effective_backends: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        session_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            &self.registry,
            effective_backends,
            session_override,
            slot_registry,
        )
    }
}

impl ResolvedRolePlugins {
    /// 与 `role.plugin_backends` 一致，便于日志/测试断言（只读借用，避免热路径 clone）。
    #[must_use]
    pub fn backends_snapshot(role: &Role) -> &PluginBackends {
        &role.plugin_backends
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::llm::MockLlmClient;
    use crate::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
    use crate::models::{
        EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
        PluginBackendsOverride, PromptBackend,
    };
    use std::sync::Arc;

    fn host() -> PluginHost {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: String::new(),
        });
        let tmp = std::env::temp_dir();
        let grants = HighRiskGrantStore::load(tmp.clone(), false);
        let remote_fb = new_remote_fallback_switch(true);
        PluginHost::new(llm, None, tmp, grants, remote_fb)
    }

    #[test]
    fn resolve_matches_role_plugin_backends_default() {
        let role = Role::default();
        assert_eq!(
            ResolvedRolePlugins::backends_snapshot(&role),
            &role.plugin_backends
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
        assert!(
            h.register_local_provider(LocalPluginProviderDescriptor {
                provider_id: "local.demo".to_string(),
                schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Prompt],
            })
            .is_ok(),
            "register local provider"
        );
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
        assert!(
            h.register_local_provider(LocalPluginProviderDescriptor {
                provider_id: "mem.local.one".to_string(),
                schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
                min_runtime_version: None,
                capabilities: vec![LocalPluginCapability::Memory],
            })
            .is_ok(),
            "register mem.local.one"
        );
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
            .expect("rank")
            .into_iter()
            .map(|m| m.id)
            .collect();
        let b: Vec<_> = v2
            .rank_memories(mk())
            .expect("rank")
            .into_iter()
            .map(|m| m.id)
            .collect();
        assert_eq!(a, b);
    }

    #[test]
    fn memory_local_hint_selects_named_provider() {
        let h = host();
        for id in ["mem.a", "mem.z"] {
            assert!(
                h.register_local_provider(LocalPluginProviderDescriptor {
                    provider_id: id.to_string(),
                    schema_version: crate::domain::LOCAL_PLUGIN_SCHEMA_VERSION,
                    min_runtime_version: None,
                    capabilities: vec![LocalPluginCapability::Memory],
                })
                .is_ok(),
                "register {}",
                id
            );
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
