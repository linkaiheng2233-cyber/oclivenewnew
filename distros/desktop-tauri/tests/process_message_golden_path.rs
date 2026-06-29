//! `process_message` 黄金路径：内存 DB + [`MockLlmClient`]，断言回复与 DTO 契约字段。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{SendMessageRequest, API_VERSION, SCHEMA_VERSION};
use std::sync::Arc;

#[tokio::test]
async fn process_message_golden_path_mock_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: "黄金路径回复".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("AppState");

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

    assert_eq!(res.api_version, API_VERSION);
    assert_eq!(res.schema, SCHEMA_VERSION);
    assert_eq!(res.reply, "黄金路径回复");
    assert!(!res.scene_id.is_empty());
    #[allow(deprecated)]
    {
        assert!(!res.relation_state.is_empty());
        assert!(res.favorability_current >= 0.0 && res.favorability_current <= 100.0);
    }
}
