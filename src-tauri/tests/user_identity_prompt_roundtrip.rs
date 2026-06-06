//! User Identity Prompt Template injected into main dialogue prompt when `user_identities/` is present.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::error::Result;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
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
async fn user_identity_template_injected_into_main_prompt() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "??????".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("turn1");

    let p1 = prompts.lock().last().expect("prompt captured").clone();
    assert!(
        p1.contains("??????"),
        "prompt should include user identity section"
    );
    assert!(
        p1.contains("????"),
        "prompt should include template body from user_identities/classmate.md"
    );
}
