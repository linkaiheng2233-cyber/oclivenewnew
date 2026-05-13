//! P2：插件权限 DB 与命令层烟测 — 授予 / 撤销 / 查询一致性。

use oclive_kernel_runtime::domain::plugin_permission_commands::{
    get_plugin_permission_grants, set_plugin_permission_grant, SetPluginPermissionGrantRequest,
};
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "perm_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plugin_permission_grant_revoke_roundtrip() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "need roles/shimeng"
    );
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let pid = "p2_perm_test_plug";
    let perm = "rpc:invoke";

    set_plugin_permission_grant(
        &state,
        &SetPluginPermissionGrantRequest {
            plugin_id: pid.into(),
            permission: perm.into(),
            enabled: true,
        },
    )
    .await
    .expect("grant");

    assert!(
        state
            .db_manager
            .is_plugin_permission_granted(pid, perm)
            .await
            .unwrap_or(false),
        "DB should report granted after upsert enabled=true"
    );

    let grants = get_plugin_permission_grants(&state, pid)
        .await
        .expect("list grants");
    assert!(
        grants
            .grants
            .iter()
            .any(|g| g.permission == perm && g.enabled),
        "get_plugin_permission_grants should include enabled row"
    );

    set_plugin_permission_grant(
        &state,
        &SetPluginPermissionGrantRequest {
            plugin_id: pid.into(),
            permission: perm.into(),
            enabled: false,
        },
    )
    .await
    .expect("revoke");

    assert!(
        !state
            .db_manager
            .is_plugin_permission_granted(pid, perm)
            .await
            .unwrap_or(true),
        "DB should report not granted after upsert enabled=false"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plugin_permission_set_rejects_empty_ids() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let err = set_plugin_permission_grant(
        &state,
        &SetPluginPermissionGrantRequest {
            plugin_id: "  ".into(),
            permission: "rpc:invoke".into(),
            enabled: true,
        },
    )
    .await
    .expect_err("empty plugin_id");
    let msg = err.to_string();
    assert!(
        msg.contains("required") || msg.contains("plugin_id"),
        "unexpected err: {}",
        msg
    );
}
