//! K-PLATFORM-01a Full: single-command invoke-shaped smoke for `list_roles`.
//!
//! Windows mock `WebviewWindow` IPC (`tauri::test::get_ipc_response`) can fail to start
//! with `STATUS_ENTRYPOINT_NOT_FOUND` on some toolchains; this smoke instead exercises the
//! same command body (`list_roles_impl`) plus IPC error serialization contract.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::command_error::CommandError;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::error::AppError;
use oclivenewnew_tauri::api::role::list_roles_impl;
use std::sync::Arc;

#[tokio::test]
async fn list_roles_invoke_smoke() {
    let llm = Arc::new(MockLlmClient {
        reply: "unused".into(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("state");

    let roles = list_roles_impl(&state).await.expect("list_roles");
    assert!(
        roles.iter().any(|r| r.id == "mumu"),
        "expected mumu role; got {:?}",
        roles.iter().map(|r| r.id.as_str()).collect::<Vec<_>>()
    );
    let landlady = roles
        .iter()
        .find(|role| role.id == "gentle-landlady")
        .expect("gentle-landlady must be visible in the normal Chat Pro role list");
    assert_eq!(
        landlady.name, "邻居阿姨",
        "gentle-landlady must use the user-facing role name"
    );
    assert!(
        landlady.adult_extension_available,
        "gentle-landlady must advertise its optional adult extension"
    );

    // IPC Err payload: CommandError serializes as kernel JSON string (Tauri 2 replaces InvokeError).
    let err = CommandError::from(AppError::RoleNotFound("missing".into()));
    let payload = serde_json::to_string(&err).expect("serialize CommandError");
    assert!(
        payload.contains("ROLE_NOT_FOUND") || payload.contains("role"),
        "unexpected IPC error payload: {payload}"
    );
}
