//! Directory plugin slot id resolution and cache-backed RPC wiring (D-READ-05 split).

use super::backend_registry::BackendRegistry;
use crate::models::{DirectoryPluginSlots, PluginBackends};
use parking_lot::RwLock;
use std::collections::BTreeMap;

pub(super) fn directory_slot_id(
    slots: &DirectoryPluginSlots,
    pick: impl FnOnce(&DirectoryPluginSlots) -> &Option<String>,
) -> Option<String> {
    pick(slots)
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

impl BackendRegistry {
    pub(super) fn pick_directory_slot<T, Pick, Build>(
        &self,
        module: &'static str,
        backends: &PluginBackends,
        cache: &RwLock<BTreeMap<String, T>>,
        pick: Pick,
        fallback: T,
        build: Build,
    ) -> T
    where
        Pick: FnOnce(&DirectoryPluginSlots) -> &Option<String>,
        Build: FnOnce(&Self, &str, &str) -> T,
        T: Clone + Send + Sync + 'static,
    {
        let Some(_rt) = self.directory_runtime_for_slots() else {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.{module}=directory but directory plugin runtime disabled; using fallback"
            );
            return fallback;
        };
        let Some(pid) = directory_slot_id(&backends.directory_plugins, pick) else {
            tracing::warn!(
                target: "oclive_plugin",
                "plugin_backends.{module}=directory but directory_plugins.{module} missing; using fallback"
            );
            return fallback;
        };
        if let Some(cached) = cache.read().get(&pid).cloned() {
            return cached;
        }
        match _rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let built = build(self, pid.as_str(), url.as_str());
                cache.write().insert(pid, built.clone());
                built
            }
            Err(e) => {
                tracing::error!(
                    target: "oclive_plugin",
                    "directory {module} plugin_id={pid} spawn failed: {e}"
                );
                fallback
            }
        }
    }
}
