//! HostProfile `[user_identity].default_id` applies when session has no explicit identity.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::host_profile::load_host_profile_file;
use oclive_kernel_host::domain::reply_post_processor::apply_effective_post_processor_config;
use oclive_kernel_host::domain::user_identity_loader::resolve_active_user_identity;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::models::{
    RolePackBuiltinReplyPostProcessorConfig, RolePackReplyPostProcessorConfig,
};
use oclivenewnew_tauri::api::role::load_role_impl;
use oclivenewnew_tauri::error::Result;
use std::io::Write;
use std::sync::Arc;
use tempfile::TempDir;

struct StubLlm;

#[async_trait]
impl LlmClient for StubLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("ok".to_string())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn host_profile_default_identity_used_when_db_empty() {
    let dir = TempDir::new().unwrap();
    let profile_file = dir.path().join("distro.oclive.toml");
    let mut f = std::fs::File::create(&profile_file).unwrap();
    f.write_all(
        br#"
distro_id = "test"
[user_identity]
default_id = "classmate"
"#,
    )
    .unwrap();
    let host = load_host_profile_file(&profile_file).expect("profile");
    assert_eq!(host.user_identity.default_id.as_deref(), Some("classmate"));

    let roles_dir = common::roles_dir();
    let llm: Arc<dyn LlmClient> = Arc::new(StubLlm);
    let state = AppStateBuilder::in_memory_test(llm, roles_dir, None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");

    load_role_impl(&state, "mumu", true).await.expect("load");
    state
        .db_manager
        .set_use_manifest_default_identity("mumu", false)
        .await
        .expect("manifest off");
    let role = state.load_role_cached_async("mumu").await.expect("role");
    let resolved = resolve_active_user_identity(&state, &role, "mumu", None)
        .await
        .expect("resolve");
    assert_eq!(resolved.identity_id, "classmate");
}

#[test]
fn minimal_post_process_chain_merge() {
    use oclive_kernel_host::domain::host_profile::{
        HostProfile, PostProcessChain, PostProcessChainProfile, UserIdentityProfile,
    };

    let host = HostProfile {
        post_process: PostProcessChainProfile {
            chain: PostProcessChain::Minimal,
        },
        user_identity: UserIdentityProfile::default(),
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
