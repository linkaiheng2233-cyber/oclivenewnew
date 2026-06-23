//! Multi-turn relation transition hint injection and decay in main dialogue Prompt.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::prompt_builder::relation_transition_duration;
use oclive_kernel_host::domain::relation_transition::maybe_start_relation_transition;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_types::models::PersonalitySource;
use oclivenewnew_tauri::error::Result;
use parking_lot::Mutex;
use std::sync::Arc;

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

fn latest_main_prompt(prompts: &Mutex<Vec<String>>, user_line: &str) -> Option<String> {
    prompts
        .lock()
        .iter()
        .rev()
        .find(|p| p.contains(user_line))
        .cloned()
}

#[tokio::test]
async fn relation_transition_hint_persists_then_expires() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "mumu".to_string(),
        user_message: "seed".to_string(),
        scene_id: None,
        ..Default::default()
    };
    process_message(&state, &req).await.expect("seed turn");

    let role = state.storage.load_role("mumu").expect("mumu role");
    let srid = "mumu";

    maybe_start_relation_transition(
        &state.session_cache,
        state.db_manager.as_ref(),
        &role,
        srid,
        "Acquaintance",
        "Friend",
        5.0,
    )
    .await
    .expect("start transition");

    let remaining = relation_transition_duration(1, 5.0);
    assert!(remaining >= 2);

    for i in 0..remaining {
        let user_line = format!("过渡轮{i}");
        process_message(
            &state,
            &SendMessageRequest {
                role_id: "mumu".to_string(),
                user_message: user_line.clone(),
                scene_id: None,
                ..Default::default()
            },
        )
        .await
        .expect("transition turn");

        let prompt = latest_main_prompt(&prompts, &user_line).expect("prompt captured");
        assert!(
            prompt.contains("【关系过渡】"),
            "turn {i} should include transition section"
        );
        assert!(
            prompt.contains("Acquaintance") && prompt.contains("Friend"),
            "turn {i} should include transition relation labels"
        );
    }

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "after transition".to_string(),
            scene_id: None,
            ..Default::default()
        },
    )
    .await
    .expect("post transition turn");

    let after = latest_main_prompt(&prompts, "after transition").expect("after prompt");
    assert!(
        !after.contains("【关系过渡】"),
        "transition section should disappear after buffer expires"
    );
}

#[tokio::test]
async fn vector_mode_transition_does_not_write_mutable_profile() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    let role = state.storage.load_role("mumu").expect("role");
    assert_eq!(
        role.evolution_config.personality_source,
        PersonalitySource::Vector
    );

    let srid = "mumu";

    maybe_start_relation_transition(
        &state.session_cache,
        state.db_manager.as_ref(),
        &role,
        srid,
        "Stranger",
        "Acquaintance",
        4.0,
    )
    .await
    .expect("start");

    let mutable = state
        .db_manager
        .get_mutable_personality(srid)
        .await
        .expect("mutable");
    assert!(
        mutable.trim().is_empty(),
        "vector mode must not persist ## 关系过渡 to mutable profile"
    );

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "vector transition".to_string(),
            scene_id: None,
            ..Default::default()
        },
    )
    .await
    .expect("turn");

    let prompt = latest_main_prompt(&prompts, "vector transition").expect("prompt");
    assert!(prompt.contains("【关系过渡】"));
}
