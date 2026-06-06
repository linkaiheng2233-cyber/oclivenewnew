//! Per-session turn lock: concurrent messages on the same `srid` must both persist without loss.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclivenewnew_tauri::api::role::load_role_impl;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

async fn test_state() -> Arc<AppState> {
    let llm = Arc::new(MockLlmClient {
        reply: "并发锁测试回复".to_string(),
    });
    Arc::new(
        AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("AppState"),
    )
}

#[tokio::test]
async fn concurrent_turns_same_srid_both_persist_stm() {
    let state = test_state().await;
    let role_id = "mumu";
    load_role_impl(state.as_ref(), role_id, true)
        .await
        .expect("load_role");

    let before = state
        .db_manager
        .list_short_term_turns(role_id)
        .await
        .expect("stm before")
        .len();

    let req_a = SendMessageRequest {
        role_id: role_id.to_string(),
        user_message: "并发消息A".to_string(),
        scene_id: None,
        ..Default::default()
    };
    let req_b = SendMessageRequest {
        role_id: role_id.to_string(),
        user_message: "并发消息B".to_string(),
        scene_id: None,
        ..Default::default()
    };

    let state_a = Arc::clone(&state);
    let state_b = Arc::clone(&state);
    let (res_a, res_b) = tokio::join!(
        async move { process_message(state_a.as_ref(), &req_a).await },
        async move { process_message(state_b.as_ref(), &req_b).await },
    );

    res_a.expect("turn A");
    res_b.expect("turn B");

    let turns = state
        .db_manager
        .list_short_term_turns(role_id)
        .await
        .expect("stm after");
    assert!(
        turns.len() >= before + 2,
        "expected at least two new STM rows, got {} (before={before})",
        turns.len()
    );

    let inputs: Vec<&str> = turns.iter().map(|(u, _, _, _, _)| u.as_str()).collect();
    assert!(
        inputs.iter().any(|s| s.contains("并发消息A")),
        "missing user A in STM"
    );
    assert!(
        inputs.iter().any(|s| s.contains("并发消息B")),
        "missing user B in STM"
    );
}
