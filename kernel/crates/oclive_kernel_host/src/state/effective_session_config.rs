//! Per-turn session plugin backend / slot registry resolution (computed once per turn).

use crate::infrastructure::storage::pick_llm_backend_env_override;
use crate::models::{PluginBackendSource, PluginBackends, PluginBackendsSourceMap, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Session-effective backends, source map, and merged slot registry — one merge per turn.
#[derive(Debug)]
pub struct EffectiveSessionConfig {
    pub backends: Arc<PluginBackends>,
    pub sources: PluginBackendsSourceMap,
    pub slot_registry: Option<BTreeMap<String, SlotRegistryEntry>>,
}

impl EffectiveSessionConfig {
    #[must_use]
    pub fn compute(role: &Role, session_namespace: &str, state: &super::AppState) -> Arc<Self> {
        let slot_overrides = state.session_slot_overrides(session_namespace);
        let slot_registry = role
            .slot_registry
            .as_ref()
            .map(|pack| oclive_validation::effective_slot_registry(pack, &slot_overrides));

        let mut backends = if let Some(ref eff) = slot_registry {
            oclive_validation::slot_registry_to_plugin_backends(eff)
        } else {
            (*role.plugin_backends).clone()
        };

        let provider = state.user_llm_provider.read().trim().to_ascii_lowercase();
        if provider == "cloud" {
            backends.llm = crate::models::plugin_backends::LlmBackend::Remote;
        } else if provider == "local" {
            backends.llm = crate::models::plugin_backends::LlmBackend::Ollama;
        } else if let Some(llm) = pick_llm_backend_env_override() {
            backends.llm = llm;
        } else if std::env::var("OCLIVE_REMOTE_LLM_URL")
            .ok()
            .is_some_and(|u| !u.trim().is_empty())
            && std::env::var("OCLIVE_REMOTE_LLM_TOKEN")
                .ok()
                .is_some_and(|t| !t.trim().is_empty())
        {
            backends.llm = crate::models::plugin_backends::LlmBackend::Remote;
        }
        let backends =
            super::host_backends::apply_host_ceiling(&backends, state.host_profile.as_ref());
        let sanitized = oclive_validation::sanitize_unimplemented_agent_backend(backends);
        for msg in &sanitized.warnings {
            tracing::warn!(target: "oclive_plugin", "session={session_namespace} {msg}");
        }

        let mut sources = PluginBackendsSourceMap::default();
        if let Some(reg) = role.slot_registry.as_ref() {
            for key in slot_overrides.keys() {
                let Some(entry) = reg.get(key) else {
                    continue;
                };
                match entry.slot_type.as_str() {
                    "memory" => sources.memory = PluginBackendSource::SessionOverride,
                    "emotion" => sources.emotion = PluginBackendSource::SessionOverride,
                    "event" => sources.event = PluginBackendSource::SessionOverride,
                    "prompt" => sources.prompt = PluginBackendSource::SessionOverride,
                    "llm" => sources.llm = PluginBackendSource::SessionOverride,
                    "agent" => sources.agent = PluginBackendSource::SessionOverride,
                    _ => {}
                }
            }
        }
        if sources.llm == PluginBackendSource::PackDefault
            && pick_llm_backend_env_override().is_some()
        {
            sources.llm = PluginBackendSource::EnvOverride;
        }

        Arc::new(Self {
            backends: Arc::new(sanitized.backends),
            sources,
            slot_registry,
        })
    }
}
