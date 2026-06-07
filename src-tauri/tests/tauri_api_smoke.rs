//! 主仓最小烟测：`AppState` + `process_message` 编排链路（不经 Tauri IPC）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_host::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[tokio::test]
async fn process_message_smoke_mock_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: "烟测回复".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let res = process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "你好".to_string(),
            scene_id: None,
            ..Default::default()
        },
    )
    .await
    .expect("process_message");

    assert_eq!(res.reply, "烟测回复");
}
