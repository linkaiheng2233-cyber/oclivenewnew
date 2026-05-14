//! 主仓最小烟测：`AppState` + `process_message` 编排链路（不经 Tauri IPC）。

use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
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
            session_id: None,
        },
    )
    .await
    .expect("process_message");

    assert_eq!(res.reply, "烟测回复");
}
