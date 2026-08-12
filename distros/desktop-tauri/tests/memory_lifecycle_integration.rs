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
        .list_short_term_recent_turns(&srid, 10, false)
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
            content_scope: None,
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
        .list_short_term_recent_turns(&srid, 10, false)
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
            content_scope: None,
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
        .list_short_term_recent_turns(&srid, 10, false)
        .await
        .expect("stm");
    assert_eq!(stm.len(), 2, "two turns should yield two STM rows");
    assert!(stm[0].0.contains("第一轮") || stm[1].0.contains("第一轮"));
    assert!(stm[0].0.contains("第二轮") || stm[1].0.contains("第二轮"));
}

#[tokio::test]
async fn ltm_merge_dedupes_similar_turn_memories() {
    let tmp = TempDir::new().unwrap();
    let role_id = "mem_dedup_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-dedup-sess";
    let mut host = HostProfile::default();
    host.turn_thinking.fast_persistence = FastPersistenceMode::Legacy;
    let state = state_for_role(tmp.path(), host).await;

    run_turn(&state, role_id, session_id, "记住我喜欢蓝色天空").await;
    run_turn(&state, role_id, session_id, "记住我还喜欢蓝色天空").await;

    let srid = conversation_state_role_id(role_id, Some(session_id));
    let count = state
        .db_manager
        .count_memories(&srid)
        .await
        .expect("ltm count");
    assert!(
        count <= 1,
        "similar LTM lines should merge/dedupe to one row, got {count}"
    );
}

#[tokio::test]
async fn strong_only_persistence_gate_allows_ltm_on_quarrel_event() {
    use oclive_kernel_host::domain::turn_thinking::{TurnThinkingMode, TurnThinkingPlan};
    use oclive_kernel_types::models::{Event, EventType, PersonalityVector};

    let tmp = TempDir::new().unwrap();
    let role_id = "mem_strong_event_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-strong-event-sess";
    let host = HostProfile {
        turn_thinking: TurnThinkingProfile {
            fast_persistence: FastPersistenceMode::StrongOnly,
            ..TurnThinkingProfile::default()
        },
        ..HostProfile::default()
    };
    let plan = TurnThinkingPlan {
        mode: TurnThinkingMode::Fast,
        reasons: vec![],
    };
    assert!(
        plan.applies_full_persistence(&host, &EventType::Quarrel),
        "strong_only Fast must treat Quarrel as strong persistence"
    );

    let state = state_for_role(tmp.path(), host).await;
    state
        .load_role_cached_async(role_id)
        .await
        .expect("load role");
    let srid = conversation_state_role_id(role_id, Some(session_id));
    state
        .db_manager
        .ensure_role_runtime(&srid)
        .await
        .expect("runtime");
    let personality = PersonalityVector {
        stubbornness: 0.5,
        clinginess: 0.5,
        sensitivity: 0.5,
        assertiveness: 0.5,
        forgiveness: 0.5,
        talkativeness: 0.5,
        warmth: 0.5,
    };
    let quarrel = Event {
        event_type: EventType::Quarrel,
        user_emotion: "angry".to_string(),
        bot_emotion: "sad".to_string(),
    };
    state
        .db_manager
        .apply_chat_turn_atomic(oclive_kernel_host::infrastructure::db::ChatTurnTxInput {
            role_id: &srid,
            personality: &personality,
            current_emotion: "Sad",
            relation_state: "Friend",
            user_relation_key: "friend",
            favor_delta: -0.1,
            memory_content: "quarrel memory line",
            memory_scope: "ordinary",
            memory_importance: plan.memory_importance_after_policy(
                &state.host_profile,
                &EventType::Quarrel,
                0.6,
            ),
            memory_fifo_limit: 500,
            memory_similarity_threshold: 0.6,
            event: &quarrel,
            user_message: "我们吵架了",
            bot_reply: "模拟回复",
            scene_id: "default",
        })
        .await
        .expect("atomic turn");
    let count = state
        .db_manager
        .count_memories(&srid)
        .await
        .expect("ltm count");
    assert_eq!(
        count, 1,
        "strong_only gate must allow LTM write when importance > 0 on Quarrel"
    );
    // M2 slice 0: the same turn must persist the six-slot emotion dimensions on
    // the events row — events stay six-slot, no complex-emotion leakage.
    let events = state
        .db_manager
        .get_events(&srid, 10)
        .await
        .expect("events list");
    assert_eq!(events.len(), 1, "single turn should persist one event row");
    assert_eq!(events[0].event_type, EventType::Quarrel);
    assert_eq!(events[0].user_emotion, "angry");
    assert_eq!(events[0].bot_emotion, "sad");
}

#[tokio::test]
async fn second_turn_prompt_reads_prior_ltm() {
    use async_trait::async_trait;
    use oclive_kernel_host::infrastructure::llm::LlmClient;
    use oclivenewnew_tauri::error::Result;
    use parking_lot::Mutex;
    use std::sync::Arc;

    struct CapturePromptLlm {
        prompts: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl LlmClient for CapturePromptLlm {
        async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
            self.prompts.lock().push(prompt.to_string());
            Ok("好的".to_string())
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            Ok("neutral".to_string())
        }

        async fn startup_probe(&self) -> Result<()> {
            Ok(())
        }
    }

    let tmp = TempDir::new().unwrap();
    let role_id = "mem_prompt_fixture";
    write_minimal_role(&tmp, role_id);
    let session_id = "mem-prompt-sess";
    let mut host = HostProfile::default();
    host.turn_thinking.fast_persistence = FastPersistenceMode::Legacy;
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
    });
    let state = AppStateBuilder::in_memory_test(llm, tmp.path(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");

    let marker = "蓝色风筝";
    run_turn(&state, role_id, session_id, &format!("记住我喜欢{marker}")).await;
    run_turn(&state, role_id, session_id, "接着聊别的").await;

    let guard = prompts.lock();
    let p2 = guard
        .iter()
        .find(|p| p.contains("接着聊别的"))
        .expect("turn2 main prompt");
    assert!(
        p2.contains(marker),
        "turn2 prompt should retrieve prior LTM content"
    );
}
