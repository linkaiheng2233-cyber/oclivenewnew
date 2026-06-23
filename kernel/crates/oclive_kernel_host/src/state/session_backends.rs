//! Session-scoped plugin backend / slot override helpers on [`super::AppState`].

use super::{AppState, EffectiveSessionConfig};
use crate::models::{PluginBackends, PluginBackendsOverride, PluginBackendsSourceMap, Role};
use std::collections::BTreeMap;
use std::sync::Arc;

impl AppState {
    #[must_use]
    pub fn effective_session_config_for(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> Arc<EffectiveSessionConfig> {
        EffectiveSessionConfig::compute(role, session_namespace, self)
    }

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
        self.session_cache.touch_session(session_namespace);
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
        self.session_cache.touch_session(session_namespace);
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
        self.effective_session_config_for(role, session_namespace)
            .slot_registry
            .clone()
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
        Arc::clone(
            &self
                .effective_session_config_for(role, session_namespace)
                .backends,
        )
    }

    #[must_use]
    pub fn effective_plugin_backend_sources_for_session(
        &self,
        role: &Role,
        session_namespace: &str,
    ) -> PluginBackendsSourceMap {
        self.effective_session_config_for(role, session_namespace)
            .sources
            .clone()
    }
}
