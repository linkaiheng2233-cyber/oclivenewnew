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

impl From<&EffectiveReplyPostProcessorConfig> for ReplyPostProcessorEffectiveConfig {
    fn from(eff: &EffectiveReplyPostProcessorConfig) -> Self {
        Self {
            backend: eff.backend,
            builtin: eff.builtin.clone(),
            remote: eff.remote.clone(),
            directory: eff.directory.clone(),
        }
    }
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
    let wire: &dyn ReplyPostProcessorResolver = state.reply_post_processor_resolver.as_ref();
    let wire_cfg = ReplyPostProcessorEffectiveConfig::from(&eff);
    match eff.backend {
        ReplyPostProcessorBackendKind::Builtin => resolve_builtin(&eff),
        ReplyPostProcessorBackendKind::Remote => wire.resolve_remote(&wire_cfg),
        ReplyPostProcessorBackendKind::Directory => wire.resolve_directory(&wire_cfg),
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
