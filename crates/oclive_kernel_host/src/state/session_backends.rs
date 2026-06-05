//! Session-scoped plugin backend / slot override helpers on [`super::AppState`].

use super::AppState;
use crate::infrastructure::storage::resolve_llm_backend_env_override;
use crate::models::{
    PluginBackendSource, PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap, Role,
};
use std::collections::BTreeMap;
use std::sync::Arc;

impl AppState {
    #[must_use]
    pub fn session_backend_override(
        &self,
        session_namespace: &str,
    ) -> Option<PluginBackendsOverride> {
        self.session_cache
            .session_plugin_overrides()
            .read()
            .get(session_namespace)
            .cloned()
    }

    pub fn set_session_backend_override(
        &self,
        session_namespace: &str,
        override_backends: PluginBackendsOverride,
    ) {
        if override_backends.is_empty() {
            self.session_cache
                .session_plugin_overrides()
                .write()
                .remove(session_namespace);
            return;
        }
        self.session_cache
            .session_plugin_overrides()
            .write()
            .insert(session_namespace.to_string(), override_backends);
    }

    pub fn clear_session_backend_override(&self, session_namespace: &str) {
        self.session_cache
            .session_plugin_overrides()
            .write()
            .remove(session_namespace);
    }

    #[must_use]
    pub fn session_slot_overrides(
        &self,
        session_namespace: &str,
    ) -> BTreeMap<String, oclive_validation::SlotOverridePatch> {
        self.session_cache
            .session_slot_overrides()
            .read()
            .get(session_namespace)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_session_slot_override(
        &self,
        session_namespace: &str,
        slot_key: &str,
        patch: oclive_validation::SlotOverridePatch,
    ) {
        let key = slot_key.trim();
        if key.is_empty() {
            return;
        }
        if patch.is_empty() {
            let mut map = self.session_cache.session_slot_overrides().write();
            if let Some(m) = map.get_mut(session_namespace) {
                m.remove(key);
                if m.is_empty() {
                    map.remove(session_namespace);
                }
            }
            return;
        }
        let mut map = self.session_cache.session_slot_overrides().write();
        let entry = map.entry(session_namespace.to_string()).or_default();
        let mut merged = {
            let mut base = entry.get(key).cloned().unwrap_or_default();
            patch.merge_into(&mut base);
            base
        };
        if let Some(ref id) = merged.local_memory_provider_id {
            let t = id.trim();
            merged.local_memory_provider_id = if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            };
        }
        entry.insert(key.to_string(), merged);
    }

    pub fn clear_session_slot_override(&self, session_namespace: &str, slot_key: &str) {
        let key = slot_key.trim();
        if key.is_empty() {
            return;
        }
        let mut map = self.session_cache.session_slot_overrides().write();
        if let Some(m) = map.get_mut(session_namespace) {
            m.remove(key);
            if m.is_empty() {
                map.remove(session_namespace);
            }
        }
    }

    pub fn clear_all_session_slot_overrides(&self, session_namespace: &str) {
        self.session_cache
            .session_slot_overrides()
            .write()
            .remove(session_namespace);
    }

    #[must_use]
    pub fn effective_slot_registry_for_session(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> Option<BTreeMap<String, oclive_validation::SlotRegistryEntry>> {
        let pack = role.slot_registry.as_ref()?;
        let ov = self.session_slot_overrides(session_namespace);
        Some(oclive_validation::effective_slot_registry(pack, &ov))
    }

    #[must_use]
    pub fn slot_session_overridden_keys(&self, session_namespace: &str) -> Vec<String> {
        self.session_slot_overrides(session_namespace)
            .keys()
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn effective_plugin_backends_for_session(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> Arc<PluginBackends> {
        let mut backends =
            if let Some(eff) = self.effective_slot_registry_for_session(role, session_namespace) {
                oclive_validation::slot_registry_to_plugin_backends(&eff)
            } else {
                (*role.plugin_backends).clone()
            };
        let provider = self.user_llm_provider.read().trim().to_ascii_lowercase();
        if provider == "cloud" {
            backends.llm = crate::models::plugin_backends::LlmBackend::Remote;
        } else if provider == "local" {
            backends.llm = crate::models::plugin_backends::LlmBackend::Ollama;
        } else if let Some(llm) = resolve_llm_backend_env_override() {
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
            super::host_backends::apply_host_ceiling(&backends, self.host_profile.as_ref());
        Arc::new(backends)
    }

    #[must_use]
    pub fn effective_plugin_backend_sources_for_session(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> PluginBackendsSourceMap {
        let mut out = PluginBackendsSourceMap::default();
        if let Some(reg) = role.slot_registry.as_ref() {
            for (key, _) in self.session_slot_overrides(session_namespace) {
                let Some(entry) = reg.get(&key) else {
                    continue;
                };
                match entry.slot_type.as_str() {
                    "memory" => out.memory = PluginBackendSource::SessionOverride,
                    "emotion" => out.emotion = PluginBackendSource::SessionOverride,
                    "event" => out.event = PluginBackendSource::SessionOverride,
                    "prompt" => out.prompt = PluginBackendSource::SessionOverride,
                    "llm" => out.llm = PluginBackendSource::SessionOverride,
                    "agent" => out.agent = PluginBackendSource::SessionOverride,
                    _ => {}
                }
            }
        }
        if out.llm == PluginBackendSource::PackDefault
            && resolve_llm_backend_env_override().is_some()
        {
            out.llm = PluginBackendSource::EnvOverride;
        }
        out
    }
}
