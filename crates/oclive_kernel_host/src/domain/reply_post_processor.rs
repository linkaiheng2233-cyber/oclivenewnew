//! Resolve [`ReplyPostProcessor`] from role pack `config.json` merged with [`HostProfile`] (independent of six-slot `PluginHost`).

use crate::domain::builtin_reply_post_processor::{
    BuiltinReplyPostProcessor, PassthroughReplyPostProcessor,
};
use crate::domain::host_profile::{HostProfile, PostProcessChain};
use crate::models::role::Role;
use crate::models::ReplyPostProcessorBackendKind;
use crate::state::AppState;
use oclive_kernel_contracts::{
    ReplyPostProcessor, ReplyPostProcessorEffectiveConfig, ReplyPostProcessorResolver,
};
use oclive_kernel_types::models::RolePackReplyPostProcessorConfig;
use std::sync::Arc;

/// Merge role pack `reply_post_processor` with host profile post-process chain policy.
#[must_use]
pub fn apply_effective_post_processor_config(
    host: &HostProfile,
    role_cfg: &RolePackReplyPostProcessorConfig,
) -> ReplyPostProcessorEffectiveConfig {
    let mut cfg = ReplyPostProcessorEffectiveConfig {
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

pub(crate) fn resolve_builtin(eff: &ReplyPostProcessorEffectiveConfig) -> Arc<dyn ReplyPostProcessor> {
    Arc::new(BuiltinReplyPostProcessor::new(eff.builtin.clone()))
}

/// Merges role pack + host profile chain into the effective reply post-processor backend (policy, not I/O).
#[must_use]
pub fn resolve_reply_post_processor(state: &AppState, role: &Role) -> Arc<dyn ReplyPostProcessor> {
    if !role.pack_reply_post_processor_config.enabled {
        return Arc::new(PassthroughReplyPostProcessor);
    }
    let eff = apply_effective_post_processor_config(
        state.host_profile.as_ref(),
        &role.pack_reply_post_processor_config,
    );
    let wire: &dyn ReplyPostProcessorResolver = state.reply_post_processor_resolver.as_ref();
    match eff.backend {
        ReplyPostProcessorBackendKind::Builtin => resolve_builtin(&eff),
        ReplyPostProcessorBackendKind::Remote => wire.resolve_remote(&eff),
        ReplyPostProcessorBackendKind::Directory => wire.resolve_directory(&eff),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::{PostProcessChainProfile, UserIdentityProfile};
    use oclive_kernel_types::models::RolePackBuiltinReplyPostProcessorConfig;

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
    fn disabled_role_config_stays_disabled() {
        let host = HostProfile {
            post_process: PostProcessChainProfile {
                chain: PostProcessChain::Minimal,
            },
            ..HostProfile::default()
        };
        let role_cfg = RolePackReplyPostProcessorConfig::default();
        assert!(!role_cfg.enabled);
        let eff = apply_effective_post_processor_config(&host, &role_cfg);
        assert_eq!(eff.builtin.profile, "minimal");
    }
}
