//! AB1：`narrative_hint` 全链路边界与契约审计（见 `creator-docs/testing/NARRATIVE_HINT_CONTRACT.md`）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclivenewnew_tauri::error::Result;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_host::state::AppState;
use parking_lot::Mutex;
use std::sync::Arc;

const SECTION: &str = "【复杂情感叙事提示】";
const TURN1_HINT: &str = "用户可能缺乏兴致";

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

async fn run_turn(state: &AppState, user_message: &str) {
    process_message(
        state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: user_message.to_string(),
            scene_id: None,
            session_id: Some("contract-audit".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("process_message");
}

fn main_prompt_for_user(prompts: &[String], user_line: &str) -> Option<String> {
    prompts
        .iter()
        .find(|p| p.contains(&format!("用户说: {user_line}")))
        .cloned()
}

#[tokio::test]
async fn first_turn_main_prompt_omits_narrative_section() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    run_turn(&state, "随便啦都行").await;

    let guard = prompts.lock();
    let p1 = main_prompt_for_user(&guard, "随便啦都行").expect("turn1 main prompt");
    assert!(
        !p1.contains(SECTION),
        "first turn must not inject prior narrative section"
    );
}

#[tokio::test]
async fn second_turn_injects_prior_turn_hint() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    run_turn(&state, "随便啦都行").await;
    run_turn(&state, "接着说正事").await;

    let guard = prompts.lock();
    let p2 = main_prompt_for_user(&guard, "接着说正事").expect("turn2 main prompt");
    assert!(p2.contains(SECTION));
    assert!(
        p2.contains(TURN1_HINT),
        "turn2 should carry turn1 narrative_hint"
    );
}

#[tokio::test]
async fn third_turn_prompt_includes_narrative_section_after_chain() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    run_turn(&state, "随便啦都行").await;
    run_turn(&state, "嗯").await;
    run_turn(&state, "第三轮继续聊").await;

    let guard = prompts.lock();
    let p3 = main_prompt_for_user(&guard, "第三轮继续聊").expect("turn3 main prompt");
    assert!(
        p3.contains(SECTION),
        "after three turns, turn3 prompt must include narrative section (from turn2 stored hint)"
    );
    assert!(!p3.trim().is_empty(), "turn3 prompt should be non-empty");
}
