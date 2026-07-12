//! K-MEM-01：回合写入 → STM/LTM 门控 merge → 再次读取（集成契约）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::host_profile::{
    FastPersistenceMode, HostProfile, TurnThinkingProfile,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::models::dto::{QueryMemoriesRequest, SendMessageRequest};
use oclivenewnew_tauri::api::memory::query_memories_impl;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

fn write_minimal_role(roles_root: &TempDir, role_id: &str) {
    let role_dir = roles_root.path().join(role_id);
    fs::create_dir_all(role_dir.join("scenes/default")).unwrap();
    fs::write(
        role_dir.join("manifest.json"),
        format!(
            r#"{{"id":"{role_id}","name":"MemTest","version":"0.1.0","author":"t","description":"t","scenes":["default"],"default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"user_relations":{{"friend":{{"initial_favorability":50.0,"favor_multiplier":1.0}}}},"default_relation":"friend"}}"#
        ),
    )
    .unwrap();
    fs::write(
        role_dir.join("core_personality.txt"),
        "memory lifecycle fixture",
    )
    .unwrap();
    fs::write(
        role_dir.join("settings.json"),
        r#"{"plugin_backends":{"memory":"builtin","llm":"ollama","emotion":"builtin","event":"builtin","prompt":"builtin","agent":"none"}}"#,
    )
    .unwrap();
    fs::write(
        role_dir.join("scenes/default/scene.json"),
        r#"{"id":"default","label":"default"}"#,
    )
    .unwrap();
}

async fn state_for_role(roles_dir: &std::path::Path, host: HostProfile) -> AppState {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟回复".to_string(),
    });
    AppStateBuilder::in_memory_test(llm, roles_dir, None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state")
}

async fn run_turn(state: &AppState, role_id: &str, session_id: &str, user_message: &str) {
    process_message(
        state,
        &SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: user_message.to_string(),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("turn");
}

#[tokio::test]
async fn legacy_persistence_writes_stm_and_ltm() {
    let tmp = TempDir::new().unwrap();
    let role_id = "mem_legacy_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-legacy-sess";
    let mut host = HostProfile::default();
    host.turn_thinking.fast_persistence = FastPersistenceMode::Legacy;
    let state = state_for_role(tmp.path(), host).await;

    run_turn(&state, role_id, session_id, "记住我喜欢蓝色").await;

    let srid = conversation_state_role_id(role_id, Some(session_id));
    let stm = state
        .db_manager
        .list_short_term_recent_turns(&srid, 10)
        .await
        .expect("stm");
    assert_eq!(stm.len(), 1, "STM should record the co-present turn");
    assert!(stm[0].0.contains("蓝色"));

    let memories = query_memories_impl(
        &state,
        &QueryMemoriesRequest {
            role_id: srid.clone(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("ltm query");
    assert!(
        !memories.is_empty(),
        "legacy Fast persistence should write LTM for casual turn"
    );
}

#[tokio::test]
async fn strong_only_fast_skips_ltm_but_keeps_stm() {
    let tmp = TempDir::new().unwrap();
    let role_id = "mem_strong_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-strong-sess";
    let host = HostProfile {
        turn_thinking: TurnThinkingProfile {
            fast_persistence: FastPersistenceMode::StrongOnly,
            ..TurnThinkingProfile::default()
        },
        ..HostProfile::default()
    };
    let state = state_for_role(tmp.path(), host).await;

    run_turn(&state, role_id, session_id, "今天天气不错").await;

    let srid = conversation_state_role_id(role_id, Some(session_id));
    let stm = state
        .db_manager
        .list_short_term_recent_turns(&srid, 10)
        .await
        .expect("stm");
    assert_eq!(
        stm.len(),
        1,
        "STM must still record the turn under strong_only"
    );

    let memories = query_memories_impl(
        &state,
        &QueryMemoriesRequest {
            role_id: srid.clone(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("ltm query");
    assert!(
        memories.is_empty(),
        "strong_only Fast casual turn must not insert LTM rows"
    );
}

#[tokio::test]
async fn second_turn_reads_prior_stm_turn_pair() {
    let tmp = TempDir::new().unwrap();
    let role_id = "mem_chain_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-chain-sess";
    let mut host = HostProfile::default();
    host.turn_thinking.fast_persistence = FastPersistenceMode::Legacy;
    let state = state_for_role(tmp.path(), host).await;

    run_turn(&state, role_id, session_id, "第一轮话题").await;
    run_turn(&state, role_id, session_id, "第二轮跟进").await;

    let srid = conversation_state_role_id(role_id, Some(session_id));
    let stm = state
        .db_manager
        .list_short_term_recent_turns(&srid, 10)
        .await
        .expect("stm");
    assert_eq!(stm.len(), 2, "two turns should yield two STM rows");
    assert!(stm[0].0.contains("第一轮") || stm[1].0.contains("第一轮"));
    assert!(stm[0].0.contains("第二轮") || stm[1].0.contains("第二轮"));
}
