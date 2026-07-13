//! Per-turn session plugin backend / slot registry resolution (computed once per turn).

use crate::infrastructure::storage::pick_llm_backend_env_override;
use crate::models::{PluginBackends, PluginBackendsSourceMap, Role};
use oclive_kernel_runtime::domain::plugin_resolution::{
    remote_llm_url_token_configured, resolve_session_plugin_backends, SessionPluginResolutionInput,
};
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

        let host_ceiling = oclive_kernel_runtime::domain::plugin_resolution::HostBackendCeiling {
            skip_agent: state.host_profile.skip_agent,
            backends_ceiling: state.host_profile.backends_ceiling.clone(),
        };
        let resolved = resolve_session_plugin_backends(&SessionPluginResolutionInput {
            pack_plugin_backends: (*role.plugin_backends).clone(),
            pack_slot_registry: role.slot_registry.clone(),
            session_slot_overrides: slot_overrides,
            user_llm_provider: state.user_llm_provider.read().trim().to_string(),
            llm_env_override: pick_llm_backend_env_override(),
            remote_llm_url_token_configured: remote_llm_url_token_configured(),
            host_ceiling,
        });
        for msg in &resolved.warnings {
            tracing::warn!(target: "oclive_plugin", "session={session_namespace} {msg}");
        }

        Arc::new(Self {
            backends: Arc::new(resolved.backends),
            sources: resolved.sources,
            slot_registry,
        })
    }
}
