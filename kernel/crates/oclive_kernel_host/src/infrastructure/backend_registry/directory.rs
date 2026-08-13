//! Split from backend_registry.rs (zero semantic change, facade in mod.rs).

use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::remote_plugin::{
    DirectoryComplexEmotionHttp, RemoteComplexEmotionHttp, RemotePluginHttpConfig,
};
use oclive_kernel_runtime::domain::complex_emotion::ComplexEmotionProvider;
use oclive_validation::{slot_registry_instances_sorted, SlotRegistryEntry};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use super::BackendRegistry;
use super::{BuiltinComplexEmotionArc, NoopComplexEmotionArc, RemoteComplexEmotionArc};

impl BackendRegistry {
    #[must_use]
    pub fn directory_runtime(&self) -> Option<Arc<DirectoryPluginRuntime>> {
        self.directory_runtime.clone()
    }

    pub(in crate::infrastructure) fn directory_runtime_for_slots(
        &self,
    ) -> Option<Arc<DirectoryPluginRuntime>> {
        self.directory_runtime.clone()
    }

    /// Same-type **last-wins** (`position` max) complex emotion implementation.
    #[must_use]
    pub fn pick_complex_emotion_winner(
        &self,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Arc<dyn ComplexEmotionProvider> {
        slot_registry_instances_sorted(slot_registry, "complex_emotion")
            .last()
            .map(|(_, e)| self.pick_complex_emotion_for_entry(e))
            .unwrap_or_else(|| Arc::new(NoopComplexEmotionArc))
    }

    pub fn pick_complex_emotion_for_entry(
        &self,
        entry: &SlotRegistryEntry,
    ) -> Arc<dyn ComplexEmotionProvider> {
        let remote_fb = self.remote_fallback_allowed.clone();
        match entry.backend.trim() {
            "builtin" => Arc::new(BuiltinComplexEmotionArc),
            "none" => Arc::new(NoopComplexEmotionArc),
            "remote" => {
                let cfg = entry
                    .url
                    .as_ref()
                    .map(|u| u.trim())
                    .filter(|u| !u.is_empty())
                    .map(|endpoint| RemotePluginHttpConfig {
                        endpoint: endpoint.to_string(),
                        timeout: Duration::from_millis(8_000),
                        bearer_token: None,
                    })
                    .or_else(RemotePluginHttpConfig::from_env_complex_emotion);
                let Some(cfg) = cfg else {
                    tracing::warn!(
                        target: "oclive_plugin",
                        "complex_emotion backend=remote but no url/env; using builtin"
                    );
                    return Arc::new(BuiltinComplexEmotionArc);
                };
                match RemoteComplexEmotionHttp::new(cfg, remote_fb, self.high_risk_grants.clone()) {
                    Ok(http) => Arc::new(RemoteComplexEmotionArc(Arc::new(http))),
                    Err(e) => {
                        tracing::error!(
                            target: "oclive_plugin",
                            "complex_emotion remote client build failed: {}; using builtin",
                            e
                        );
                        Arc::new(BuiltinComplexEmotionArc)
                    }
                }
            }
            "directory" => self.pick_complex_emotion_directory(entry, remote_fb),
            _ => Arc::new(BuiltinComplexEmotionArc),
        }
    }

    pub(super) fn pick_complex_emotion_directory(
        &self,
        entry: &SlotRegistryEntry,
        remote_fb: Arc<AtomicBool>,
    ) -> Arc<dyn ComplexEmotionProvider> {
        let Some(rt) = self.directory_runtime.as_ref() else {
            tracing::warn!(
                target: "oclive_plugin",
                "complex_emotion backend=directory but runtime disabled; using builtin"
            );
            return Arc::new(BuiltinComplexEmotionArc);
        };
        let plugin_id = entry
            .plugin
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let Some(pid) = plugin_id else {
            tracing::warn!(
                target: "oclive_plugin",
                "complex_emotion backend=directory but plugin id missing; using builtin"
            );
            return Arc::new(BuiltinComplexEmotionArc);
        };
        if !rt.manifest_provides_capability(&pid, "complex_emotion") {
            tracing::warn!(
                target: "oclive_plugin",
                "directory plugin_id={} missing provides complex_emotion; using builtin",
                pid
            );
            return Arc::new(BuiltinComplexEmotionArc);
        }
        match rt.ensure_rpc_url(pid.as_str()) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, false);
                match DirectoryComplexEmotionHttp::new(cfg, remote_fb) {
                    Ok(http) => Arc::new(http),
                    Err(e) => {
                        tracing::error!(
                            target: "oclive_plugin",
                            "complex_emotion directory plugin_id={} client failed: {}; using builtin",
                            pid,
                            e
                        );
                        Arc::new(BuiltinComplexEmotionArc)
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    target: "oclive_plugin",
                    "complex_emotion directory plugin_id={} spawn failed: {}; using builtin",
                    pid,
                    e
                );
                Arc::new(BuiltinComplexEmotionArc)
            }
        }
    }
}
