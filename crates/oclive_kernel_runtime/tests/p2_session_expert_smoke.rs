//! P2 补强：专家模型参数校验、`process_message` 场景字段与 `plugin_state` 异步读盘烟测。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::expert_models_admin::expert_models_set_session_override;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::infrastructure::plugin_state::PluginStateStore;
use oclive_kernel_runtime::models::dto::{ExpertModelsSetSessionOverrideRequest, SendMessageRequest};
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "p2_ok".to_string(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_models_set_session_override_rejects_empty_role_id() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "need roles/shimeng"
    );
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let err = expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "   ".into(),
            session_id: None,
            graph: Default::default(),
            prompt_style: None,
        },
    )
    .await
    .expect_err("empty role_id");

    assert!(
        err.contains("role_id"),
        "expected role_id validation, got {:?}",
        err
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_echoes_requested_scene_id() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "shimeng".to_string(),
        user_message: "scene probe".into(),
        scene_id: Some("default".into()),
        session_id: Some("p2_scene_sess".into()),
    };
    let res = process_message(&state, &req).await.expect("process_message");
    assert_eq!(res.scene_id, "default");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plugin_state_store_load_async_missing_file_is_default() {
    let dir = tempdir().expect("tmp");
    let p = dir.path().join("no_such_plugin_state.json");
    let s = PluginStateStore::load_async(&p).await;
    assert_eq!(s.schema_version, 3);
    assert!(s.roles.is_empty());
}
