//! E05：上一轮内置复杂情感的 `narrative_hint` 注入下一轮主对话 Prompt（共景路径）。

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
async fn prior_narrative_hint_injected_into_second_turn_main_prompt() {
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
            user_message: "随便啦都行".to_string(),
            scene_id: None,
            session_id: None,
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
            session_id: None,
        },
    )
    .await
    .expect("turn2");

    let hint_snippet = "用户可能缺乏兴致";
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
