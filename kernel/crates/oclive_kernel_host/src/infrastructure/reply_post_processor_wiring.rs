//! Remote/directory Reply Post-Processor wiring (HTTP clients, grants, directory spawn).

use crate::domain::reply_post_processor::resolve_builtin;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::{
    DirectoryReplyPostProcessor, RemoteReplyPostProcessorHttp,
};
use oclive_kernel_contracts::{
    ReplyPostProcessor, ReplyPostProcessorEffectiveConfig, ReplyPostProcessorResolver,
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Host infrastructure resolver for non-builtin reply post-processor backends.
pub struct HostReplyPostProcessorResolver {
    directory_plugins: Arc<DirectoryPluginRuntime>,
    remote_fallback_allowed: Arc<AtomicBool>,
    high_risk_grants: Arc<HighRiskGrantStore>,
}

impl HostReplyPostProcessorResolver {
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

impl ReplyPostProcessorResolver for HostReplyPostProcessorResolver {
    fn resolve_remote(
        &self,
        eff: &ReplyPostProcessorEffectiveConfig,
    ) -> Arc<dyn ReplyPostProcessor> {
        let Some(cfg) = RemotePluginHttpConfig::for_reply_post_processor_remote(
            eff.remote.url.as_str(),
            eff.remote.timeout_ms,
        ) else {
            tracing::warn!(
                target: "oclive_reply_post_processor",
                "reply_post_processor backend=remote but remote.url empty; builtin fallback"
            );
            return resolve_builtin(eff);
        };
        match RemoteReplyPostProcessorHttp::new(
            cfg,
            Arc::clone(&self.remote_fallback_allowed),
            Arc::clone(&self.high_risk_grants),
            eff.builtin.clone(),
        ) {
            Ok(http) => Arc::new(http),
            Err(e) => {
                tracing::warn!(
                    target: "oclive_reply_post_processor",
                    error = %e,
                    "remote reply post-processor client build failed; builtin fallback"
                );
                resolve_builtin(eff)
            }
        }
    }

    fn resolve_directory(
        &self,
        eff: &ReplyPostProcessorEffectiveConfig,
    ) -> Arc<dyn ReplyPostProcessor> {
        let pid = eff.directory.plugin_id.trim();
        if pid.is_empty() {
            tracing::warn!(
                target: "oclive_reply_post_processor",
                "reply_post_processor backend=directory but directory.plugin_id empty; builtin fallback"
            );
            return resolve_builtin(eff);
        }
        let rt = &self.directory_plugins;
        if !rt.manifest_provides_capability(pid, "reply_post_process") {
            tracing::warn!(
                target: "oclive_reply_post_processor",
                plugin_id = %pid,
                "directory plugin missing provides reply_post_process; builtin fallback"
            );
            return resolve_builtin(eff);
        }
        match rt.ensure_rpc_url(pid) {
            Ok(url) => {
                let cfg = RemotePluginHttpConfig::for_directory_plugin_rpc(url, false);
                match DirectoryReplyPostProcessor::new(
                    cfg,
                    Arc::clone(&self.remote_fallback_allowed),
                    Arc::clone(&self.high_risk_grants),
                    eff.builtin.clone(),
                ) {
                    Ok(http) => Arc::new(http),
                    Err(e) => {
                        tracing::warn!(
                            target: "oclive_reply_post_processor",
                            plugin_id = %pid,
                            error = %e,
                            "directory reply post-processor client build failed; builtin fallback"
                        );
                        resolve_builtin(eff)
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_reply_post_processor",
                    plugin_id = %pid,
                    error = %e,
                    "directory reply post-processor spawn failed; builtin fallback"
                );
                resolve_builtin(eff)
            }
        }
    }
}
