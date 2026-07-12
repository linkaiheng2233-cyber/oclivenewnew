//! AB1：`narrative_hint` 全链路边界与契约审计（见 `creator-docs/testing/NARRATIVE_HINT_CONTRACT.md`）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::complex_emotion_store::load_stored_narrative_hint;
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::error::Result;
use parking_lot::Mutex;
use std::sync::Arc;

const SECTION: &str = "【复杂情感叙事提示】";
const TURN1_HINT: &str = "用户可能缺乏兴致";
const WANING_HINT: &str = "对话热度下降";

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

async fn capture_state_async(fast_skip_ce: bool) -> (AppState, Arc<Mutex<Vec<String>>>) {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let mut host = HostProfile::default();
    host.turn_thinking.fast_skip_complex_emotion = fast_skip_ce;
    let state = AppStateBuilder::in_memory_test(llm, common::roles_dir(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");
    (state, prompts)
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
    let (state, prompts) = capture_state_async(false).await;
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
    let (state, prompts) = capture_state_async(false).await;

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
async fn third_turn_prompt_includes_updated_hint_not_stale_turn1() {
    let (state, prompts) = capture_state_async(false).await;

    run_turn(&state, "好").await;
    run_turn(&state, "嗯").await;
    run_turn(&state, "第三轮继续聊").await;

    let guard = prompts.lock();
    let p3 = main_prompt_for_user(&guard, "第三轮继续聊").expect("turn3 main prompt");
    assert!(p3.contains(SECTION));
    assert!(
        p3.contains(WANING_HINT),
        "turn3 should inject turn2 waning_engagement hint, got: {p3}"
    );
    assert!(
        !p3.contains(TURN1_HINT),
        "turn3 must not still carry turn1 disengagement hint after turn2 update"
    );
}

#[tokio::test]
async fn fast_skip_does_not_persist_or_inject_narrative_hint() {
    let (state, prompts) = capture_state_async(true).await;
    run_turn(&state, "随便啦都行").await;

    let srid = conversation_state_role_id("mumu", Some("contract-audit"));
    let stored = load_stored_narrative_hint(&state, &srid)
        .await
        .expect("load hint");
    assert!(
        stored.trim().is_empty(),
        "fast_skip must not persist a new narrative_hint on casual Fast turn"
    );

    run_turn(&state, "第二轮").await;
    let guard = prompts.lock();
    let p2 = main_prompt_for_user(&guard, "第二轮").expect("turn2 main prompt");
    assert!(
        !p2.contains(SECTION),
        "with fast_skip and no prior persist, turn2 must omit narrative section"
    );
}
