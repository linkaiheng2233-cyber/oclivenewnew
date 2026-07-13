//! Multi-turn relation transition buffer lifecycle (SessionCache + Profile archive).
//!
//! `relation_transition_hint` is passed through orchestration but is **not** injected as a
//! dedicated `【关系过渡】` prompt section (see `prompt_builder::tests::relation_transition_hint_not_injected_into_prompt`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::prompt_builder::relation_transition_duration;
use oclive_kernel_host::domain::relation_transition::maybe_start_relation_transition;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_types::models::PersonalitySource;
use oclivenewnew_tauri::error::Result;
use parking_lot::Mutex;
use std::sync::Arc;

const SESSION: &str = "relation-transition-test";

struct CapturePromptLlm {
    prompts: Arc<Mutex<Vec<String>>>,
    reply: String,
}

#[async_trait]
impl LlmClient for CapturePromptLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts.lock().push(prompt.to_string());
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn srid() -> String {
    conversation_state_role_id("mumu", Some(SESSION))
}

fn latest_main_prompt(prompts: &Mutex<Vec<String>>, user_line: &str) -> Option<String> {
    prompts
        .lock()
        .iter()
        .rev()
        .find(|p| p.contains(user_line))
        .cloned()
}

async fn capture_state() -> (oclive_kernel_host::state::AppState, Arc<Mutex<Vec<String>>>) {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let state = AppStateBuilder::in_memory_test(llm, common::roles_dir(), None)
        .build()
        .await
        .expect("state");
    (state, prompts)
}

async fn run_turn(state: &oclive_kernel_host::state::AppState, user_message: &str) {
    process_message(
        state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: user_message.to_string(),
            scene_id: None,
            session_id: Some(SESSION.to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("process_message");
}

#[tokio::test]
async fn relation_transition_hint_persists_then_expires() {
    let (state, prompts) = capture_state().await;

    run_turn(&state, "seed").await;

    let role = state.storage.load_role("mumu").expect("mumu role");
    let srid = srid();

    maybe_start_relation_transition(
        &state.session_cache,
        state.db_manager.as_ref(),
        &role,
        &srid,
        "Acquaintance",
        "Friend",
        5.0,
    )
    .await
    .expect("start transition");

    let remaining = relation_transition_duration(1, 5.0);
    assert!(remaining >= 2);
    assert!(state.session_cache.has_relation_transition(&srid));

    for _ in 0..remaining {
        let consumed = state
            .session_cache
            .consume_relation_transition(&srid)
            .expect("consume transition turn");
        assert!(
            !consumed.hint.is_empty(),
            "each buffered turn should carry a non-empty hint"
        );
    }

    assert!(
        !state.session_cache.has_relation_transition(&srid),
        "transition buffer should expire after {remaining} consumes"
    );

    run_turn(&state, "transition turn").await;
    let prompt = latest_main_prompt(&prompts, "transition turn").expect("prompt captured");
    assert!(
        !prompt.contains("【关系过渡】"),
        "main prompt must not inject a dedicated transition section"
    );

    run_turn(&state, "after transition").await;
    let after = latest_main_prompt(&prompts, "after transition").expect("after prompt");
    assert!(!after.contains("【关系过渡】"));
}

#[tokio::test]
async fn vector_mode_transition_does_not_write_mutable_profile() {
    let (state, prompts) = capture_state().await;

    let role = state.storage.load_role("mumu").expect("role");
    assert_eq!(
        role.evolution_config.personality_source,
        PersonalitySource::Vector
    );

    let srid = srid();

    maybe_start_relation_transition(
        &state.session_cache,
        state.db_manager.as_ref(),
        &role,
        &srid,
        "Stranger",
        "Acquaintance",
        4.0,
    )
    .await
    .expect("start");

    let mutable = state
        .db_manager
        .get_mutable_personality(&srid)
        .await
        .expect("mutable");
    assert!(
        mutable.trim().is_empty(),
        "vector mode must not persist ## 关系过渡 to mutable profile"
    );

    run_turn(&state, "vector transition").await;

    let mutable_after = state
        .db_manager
        .get_mutable_personality(&srid)
        .await
        .expect("mutable after turn");
    assert!(
        mutable_after.trim().is_empty(),
        "vector mode must not write mutable profile on transition turns"
    );

    let prompt = latest_main_prompt(&prompts, "vector transition").expect("prompt");
    assert!(
        !prompt.contains("【关系过渡】"),
        "vector mode transition hint is not injected as a dedicated prompt section"
    );
}
