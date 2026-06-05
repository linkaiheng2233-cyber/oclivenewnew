//! # Plugin assembly hub (`PluginHost`)
//!
//! **Role**: Parses role pack configuration (`plugin_backends`, `slot_registry`, directory plugin manifests)
//! into executable **`Arc<dyn …>`** handle sets (`ResolvedRolePlugins`) for the orchestration layer via
//! [`PluginHostPort`](crate::domain::ports::PluginHostPort).
//!
//! **Upstream**: `Role` loaded by `RoleStorage`; session overrides from `AppState`; `BackendRegistry`
//! caches builtin / remote / directory constructors.
//! **Downstream**: `chat_engine`, `SlotResolver::resolve`; implements [`PluginHostPort`](crate::domain::ports::PluginHostPort)
//! to decouple from the concrete `PluginHost` type.
//!
//! **Key decisions**: Orchestration depends only on **trait objects** so desktop / headless / test hosts
//! are swappable; when Remote env is unconfigured, **graceful degradation + logging** avoids silent failure.
//! Contract: `creator-docs/plugin-and-architecture/PLUGIN_V1.md`.
//!
//! **Clone strategy (audit 2026-05)**
//!
//! - **`Arc::clone` / `Option<Arc<_>>::clone`**: Reference-count only; backend handles on the resolve hot path are all of this kind.
//! - **`PluginBackends` struct clone**: Allocates only when applying session override via [`PluginBackendsOverride::apply_to`];
//!   with no override, borrows the pack default (see [`PluginResolver::resolve`]).
//! - **`provider_id` strings**: Local memory path uses [`pick_local_memory_provider_refs`]; only the final selected id is cloned.
//! - **`directory_slot_id`**: Directory plugin id needs an owned `String` for RPC startup; one allocation is unavoidable.

use crate::domain::agent::{AgentDebugTrace, AgentProvider};
use crate::domain::complex_emotion::ComplexEmotionProvider;
use crate::domain::event_estimator::EventEstimator;
use crate::domain::local_plugin_bridge::{LocalPluginCapability, LocalPluginProviderDescriptor};
use crate::domain::memory_retrieval::MemoryRetrieval;
use crate::domain::ports::LlmClient;
use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::slot_resolver::ResolvedRoleSlots;
use crate::domain::user_emotion_analyzer::UserEmotionAnalyzer;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::mcp_client::{McpServerManifest, McpToolCallResult};
use crate::models::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PluginBackendsOverride, PromptBackend, Role,
};
use oclive_validation::SlotRegistryEntry;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thiserror::Error;

/// Errors from [`PluginHost`] / [`BackendRegistry`] during resolve and local provider registration.
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

mod registry;
mod resolver;

pub use registry::BackendRegistry;
pub use resolver::PluginResolver;

/// Compile-time plugin implementation set ([`PluginHost::resolve_for_role`] clones `Arc` per enum variant).
pub struct PluginHost {
    registry: BackendRegistry,
}

impl PluginHost {
    /// Constructs the host registry.
    ///
    /// - `llm`: In-process default LLM handle (`plugin_backends.llm = ollama` etc. reuse or wrap this impl).
    /// - `directory_runtime`: Lazy-start directory plugin runtime; pass `None` when no directory plugins are needed.
    /// - `app_data_dir`: Application data root (Tauri app data in production). Currently used to initialize
    ///   [`McpClient`](crate::infrastructure::mcp_client::McpClient) (scans `{app_data_dir}/mcp-servers/*.json`).
    ///   Integration tests may pass `std::env::temp_dir()`.
    /// - `high_risk_grants`: Explicit permission grants for MCP transport, directory plugin subprocesses, etc.
    ///   (`{app_data}/high_risk_grants.json`).
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
    pub async fn list_mcp_tools(
        &self,
        server_id: &str,
    ) -> std::result::Result<Vec<crate::infrastructure::mcp_client::McpToolManifest>, String> {
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

    /// Resolves all backends declared by the current role pack (one clone of five `Arc`s, reused for the whole conversation).
    pub fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins {
        PluginResolver::resolve(
            &self.registry,
            &role.plugin_backends,
            None,
            role.slot_registry.as_ref(),
        )
    }

    /// Resolves role default backends plus session-level override (equivalent to [`Self::resolve_for_role`] when override is empty).
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

    /// Effective six slots + blueprint registry + optional six-slot session override (v2 hot path).
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
    /// Matches `role.plugin_backends` for logging / test assertions (read-only borrow, avoids hot-path clone).
    #[must_use]
    pub fn backends_snapshot(role: &Role) -> &PluginBackends {
        role.plugin_backends.as_ref()
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
            role.plugin_backends.as_ref()
        );
        host().resolve_for_role(&role);
    }

    #[test]
    fn resolve_selects_memory_v2_when_configured() {
        let role = Role {
            plugin_backends: std::sync::Arc::new(PluginBackends {
                memory: MemoryBackend::BuiltinV2,
                emotion: EmotionBackend::Builtin,
                event: EventBackend::Builtin,
                prompt: PromptBackend::Builtin,
                llm: LlmBackend::Ollama,
                ..Default::default()
            }),
            ..Default::default()
        };
        let h = host();
        let pl = h.resolve_for_role(&role);
        let same_again = h.memory_retrieval(MemoryBackend::BuiltinV2);
        // Within the same `PluginHost`: resolve and explicit slot lookup must share the same `Arc` pointer
        assert!(Arc::ptr_eq(&pl.memory, &same_again));
    }

    #[test]
    fn resolve_selects_emotion_v2_when_configured() {
        let role = Role {
            plugin_backends: std::sync::Arc::new(PluginBackends {
                emotion: EmotionBackend::BuiltinV2,
                ..Default::default()
            }),
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
            plugin_backends: std::sync::Arc::new(PluginBackends {
                memory: MemoryBackend::Local,
                ..Default::default()
            }),
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
            mention_count: 1,
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
            plugin_backends: std::sync::Arc::new(PluginBackends {
                memory: MemoryBackend::Local,
                local_memory_provider_id: Some("mem.z".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let pl = h.resolve_for_role(&role);
        assert_eq!(pl.memory.diagnostic_local_provider_id(), Some("mem.z"));
    }
}
