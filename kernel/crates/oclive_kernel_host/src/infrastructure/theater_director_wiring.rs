//! Directory theater director wiring (JSON-RPC client, grants, directory spawn).

use crate::domain::builtin_theater_director::BuiltinTheaterDirector;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::{DirectoryTheaterDirector, RemotePluginHttpConfig};
use oclive_kernel_contracts::{
    TheaterDirectorEffectiveConfig, TheaterDirectorPromptProvider, TheaterDirectorResolver,
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Host infrastructure resolver for directory theater director backend.
pub struct HostTheaterDirectorResolver {
    directory_plugins: Arc<DirectoryPluginRuntime>,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
}

impl HostTheaterDirectorResolver {
    #[must_use]
    pub fn new(
        directory_plugins: Arc<DirectoryPluginRuntime>,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
    ) -> Self {
        Self {
            directory_plugins,
            remote_fallback_allowed,
            high_risk_grants,
        }
    }
}

impl TheaterDirectorResolver for HostTheaterDirectorResolver {
    fn resolve_directory(
        &self,
        eff: &TheaterDirectorEffectiveConfig,
    ) -> Arc<dyn TheaterDirectorPromptProvider> {
        let pid = eff.directory_plugin_id.trim();
        if pid.is_empty() {
            tracing::warn!(
                target: "oclive_theater",
                "theater director backend=directory but plugin_id empty; builtin fallback"
            );
            return Arc::new(BuiltinTheaterDirector);
        }
        let rt = &self.directory_plugins;
        if !rt.manifest_provides_capability(pid, "theater_director") {
            tracing::warn!(
                target: "oclive_theater",
                plugin_id = %pid,
                "directory plugin missing provides theater_director; builtin fallback"
            );
            return Arc::new(BuiltinTheaterDirector);
        }
        match rt.ensure_rpc_url(pid) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, false);
                match DirectoryTheaterDirector::new(
                    cfg,
                    Arc::clone(&self.remote_fallback_allowed),
                    Arc::clone(&self.high_risk_grants),
                ) {
                    Ok(http) => Arc::new(http),
                    Err(e) => {
                        tracing::warn!(
                            target: "oclive_theater",
                            plugin_id = %pid,
                            error = %e,
                            "directory theater director client build failed; builtin fallback"
                        );
                        Arc::new(BuiltinTheaterDirector)
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_theater",
                    plugin_id = %pid,
                    error = %e,
                    "directory theater director spawn failed; builtin fallback"
                );
                Arc::new(BuiltinTheaterDirector)
            }
        }
    }
}
