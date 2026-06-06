//! Resolve [`ReplyPostProcessor`] from role pack `config.json` merged with [`HostProfile`] (independent of six-slot `PluginHost`).

use crate::domain::builtin_reply_post_processor::{
    BuiltinReplyPostProcessor, PassthroughReplyPostProcessor,
};
use crate::domain::host_profile::{HostProfile, PostProcessChain};
use crate::infrastructure::remote_plugin::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::{
    DirectoryReplyPostProcessor, RemoteReplyPostProcessorHttp,
};
use crate::models::role::Role;
use crate::models::ReplyPostProcessorBackendKind;
use crate::state::AppState;
use oclive_kernel_contracts::reply_post_processor::ReplyPostProcessor;
use oclive_kernel_types::models::{
    RolePackBuiltinReplyPostProcessorConfig, RolePackDirectoryReplyPostProcessorConfig,
    RolePackRemoteReplyPostProcessorConfig, RolePackReplyPostProcessorConfig,
};
use std::sync::Arc;

/// Role pack config after distro `[post_process].chain` merge.
#[derive(Debug, Clone)]
pub struct EffectiveReplyPostProcessorConfig {
    pub enabled: bool,
    pub backend: ReplyPostProcessorBackendKind,
    pub builtin: RolePackBuiltinReplyPostProcessorConfig,
    pub remote: RolePackRemoteReplyPostProcessorConfig,
    pub directory: RolePackDirectoryReplyPostProcessorConfig,
}

/// Merge role pack `reply_post_processor` with host profile post-process chain policy.
#[must_use]
pub fn apply_effective_post_processor_config(
    host: &HostProfile,
    role_cfg: &RolePackReplyPostProcessorConfig,
) -> EffectiveReplyPostProcessorConfig {
    let mut cfg = EffectiveReplyPostProcessorConfig {
        enabled: role_cfg.enabled,
        backend: role_cfg.backend,
        builtin: role_cfg.builtin.clone(),
        remote: role_cfg.remote.clone(),
        directory: role_cfg.directory.clone(),
    };
    if host.post_process.chain == PostProcessChain::Minimal {
        cfg.builtin.profile = "minimal".to_string();
    }
    cfg
}

fn resolve_builtin(eff: &EffectiveReplyPostProcessorConfig) -> Arc<dyn ReplyPostProcessor> {
    Arc::new(BuiltinReplyPostProcessor::new(eff.builtin.clone()))
}

fn resolve_remote(
    state: &AppState,
    eff: &EffectiveReplyPostProcessorConfig,
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
        Arc::clone(&state.remote_fallback_allowed),
        Arc::clone(&state.high_risk_grants),
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
    state: &AppState,
    eff: &EffectiveReplyPostProcessorConfig,
) -> Arc<dyn ReplyPostProcessor> {
    let pid = eff.directory.plugin_id.trim();
    if pid.is_empty() {
        tracing::warn!(
            target: "oclive_reply_post_processor",
            "reply_post_processor backend=directory but directory.plugin_id empty; builtin fallback"
        );
        return resolve_builtin(eff);
    }
    let rt = &state.directory_plugins;
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
                Arc::clone(&state.remote_fallback_allowed),
                Arc::clone(&state.high_risk_grants),
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

/// Factory: `enabled = false` → pass-through; merges host profile chain before backend resolution.
#[must_use]
pub fn resolve_reply_post_processor(state: &AppState, role: &Role) -> Arc<dyn ReplyPostProcessor> {
    let eff = apply_effective_post_processor_config(
        state.host_profile.as_ref(),
        &role.pack_reply_post_processor_config,
    );
    if !eff.enabled {
        return Arc::new(PassthroughReplyPostProcessor);
    }
    match eff.backend {
        ReplyPostProcessorBackendKind::Builtin => resolve_builtin(&eff),
        ReplyPostProcessorBackendKind::Remote => resolve_remote(state, &eff),
        ReplyPostProcessorBackendKind::Directory => resolve_directory(state, &eff),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::{PostProcessChainProfile, UserIdentityProfile};

    #[test]
    fn minimal_chain_forces_builtin_profile() {
        let host = HostProfile {
            post_process: PostProcessChainProfile {
                chain: PostProcessChain::Minimal,
            },
            ..HostProfile::default()
        };
        let role_cfg = RolePackReplyPostProcessorConfig {
            enabled: true,
            builtin: RolePackBuiltinReplyPostProcessorConfig {
                profile: "standard".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let eff = apply_effective_post_processor_config(&host, &role_cfg);
        assert_eq!(eff.builtin.profile, "minimal");
        assert!(eff.enabled);
    }

    #[test]
    fn standard_chain_respects_role_pack() {
        let host = HostProfile {
            user_identity: UserIdentityProfile::default(),
            ..HostProfile::default()
        };
        let role_cfg = RolePackReplyPostProcessorConfig {
            enabled: true,
            builtin: RolePackBuiltinReplyPostProcessorConfig {
                profile: "standard".to_string(),
                max_chars: Some(10),
                ..Default::default()
            },
            ..Default::default()
        };
        let eff = apply_effective_post_processor_config(&host, &role_cfg);
        assert_eq!(eff.builtin.profile, "standard");
        assert_eq!(eff.builtin.max_chars, Some(10));
    }

    #[test]
    fn minimal_chain_does_not_enable_disabled_role_config() {
        let host = HostProfile {
            post_process: PostProcessChainProfile {
                chain: PostProcessChain::Minimal,
            },
            ..HostProfile::default()
        };
        let role_cfg = RolePackReplyPostProcessorConfig::default();
        let eff = apply_effective_post_processor_config(&host, &role_cfg);
        assert!(!eff.enabled);
    }
}
