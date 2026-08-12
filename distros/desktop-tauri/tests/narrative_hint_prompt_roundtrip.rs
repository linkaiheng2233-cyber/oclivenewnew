//! E05：上一轮 [EMO] 派生的 `narrative_hint` 注入下一轮主对话 Prompt（共景路径）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::models::dto::SendMessageRequest;
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

#[tokio::test]
async fn prior_narrative_hint_injected_into_second_turn_main_prompt() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    // B M1 producer: the hint comes from the main LLM reply [EMO] marker.
    let hint_snippet = "用户可能缺乏兴致";
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: format!(
            "好呀\n\n[EMO]{{\"labels\":[\"neutral\"],\"intensity\":0.4,\"narrative_hint\":\"{}\"}}[/EMO]",
            hint_snippet
        ),
    });
    let mut host = HostProfile::default();
    host.turn_thinking.fast_skip_complex_emotion = false;
    let state = AppStateBuilder::in_memory_test(llm, common::roles_dir(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "随便啦都行".to_string(),
            scene_id: None,
            session_id: Some("prompt-roundtrip".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("turn1");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "接着说正事".to_string(),
            scene_id: None,
            session_id: Some("prompt-roundtrip".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("turn2");

    let p2_owned = {
        let guard = prompts.lock();
        guard
            .iter()
            .find(|p| p.contains("用户说: 接着说正事"))
            .expect("main prompt for turn2 should include user line")
            .clone()
    };

    assert!(
        p2_owned.contains(hint_snippet),
        "expected prior narrative hint in main prompt; excerpt={}",
        &p2_owned[p2_owned.len().saturating_sub(800)..]
    );
    assert!(
        p2_owned.contains("【复杂情感叙事提示】"),
        "expected narrative hint section heading"
    );
}
