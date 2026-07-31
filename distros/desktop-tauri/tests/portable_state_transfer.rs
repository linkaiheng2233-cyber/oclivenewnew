mod common;

use oclive_kernel_host::infrastructure::llm::MockLlmClient;
use oclive_kernel_host::service::{
    export_portable_memory_impl, export_portable_persona_impl, import_portable_memory_impl,
    import_portable_persona_impl,
};
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::{PortableStateImportRequest, PortableStateRequest};
use std::sync::Arc;

async fn state() -> AppState {
    AppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient { reply: "ok".into() }),
        common::roles_dir(),
    )
    .await
    .expect("state")
}

#[tokio::test]
async fn persona_roundtrip_restores_mutable_but_not_core() {
    let state = state().await;
    state.db_manager.ensure_role_runtime("mumu").await.unwrap();
    state
        .db_manager
        .set_mutable_personality("mumu", "更信任用户。")
        .await
        .unwrap();
    let exported = export_portable_persona_impl(
        &state,
        &PortableStateRequest {
            role_id: "mumu".into(),
            session_id: None,
        },
    )
    .await
    .unwrap();

    state
        .db_manager
        .set_mutable_personality("mumu", "")
        .await
        .unwrap();
    let result = import_portable_persona_impl(
        &state,
        &PortableStateImportRequest {
            role_id: "mumu".into(),
            session_id: None,
            content: exported.content,
        },
    )
    .await
    .unwrap();
    assert!(result.mutable_profile_restored);
    assert_eq!(
        state
            .db_manager
            .get_mutable_personality("mumu")
            .await
            .unwrap(),
        "更信任用户。"
    );
}

#[tokio::test]
async fn memory_roundtrip_excludes_short_term_and_ephemeral_state() {
    let state = state().await;
    state.db_manager.ensure_role_runtime("mumu").await.unwrap();
    state
        .db_manager
        .save_memory_merged("mumu", "用户喜欢雨天。", 0.8, 0.6, "default")
        .await
        .unwrap();
    let exported = export_portable_memory_impl(
        &state,
        &PortableStateRequest {
            role_id: "mumu".into(),
            session_id: None,
        },
    )
    .await
    .unwrap();
    assert!(!exported.content.contains("short_term"));
    assert!(!exported.content.contains("ephemeral"));

    let before = state.db_manager.count_memories("mumu").await.unwrap();
    let result = import_portable_memory_impl(
        &state,
        &PortableStateImportRequest {
            role_id: "mumu".into(),
            session_id: None,
            content: exported.content,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.imported_long_term, 1);
    assert_eq!(
        state.db_manager.count_memories("mumu").await.unwrap(),
        before
    );
}
