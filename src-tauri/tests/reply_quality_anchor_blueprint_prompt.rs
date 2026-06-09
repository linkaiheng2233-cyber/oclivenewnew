//! Pack-level `meta.reply_quality_anchor` flows into main dialogue Prompt (blueprint v2 load path).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::prompt_builder::{
    effective_reply_quality_anchor, DEFAULT_REPLY_QUALITY_ANCHOR, KERNEL_DIALOGUE_GUARDRAILS,
};
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::infrastructure::storage::RoleStorage;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_host::state::AppState;
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
async fn shimeng_pack_anchor_replaces_default_in_main_prompt() {
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let storage = RoleStorage::new(&roles_dir);
    let role = storage
        .load_role_from_dir(&roles_dir.join("shimeng"))
        .expect("load shimeng");
    let anchor = effective_reply_quality_anchor(&role);
    assert!(anchor.contains("诗梦"));
    assert!(!anchor.contains(DEFAULT_REPLY_QUALITY_ANCHOR.trim()));

    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "shimeng".to_string(),
            user_message: "你好".to_string(),
            scene_id: None,
            ..Default::default()
        },
    )
    .await
    .expect("turn1");

    let p1 = {
        let guard = prompts.lock();
        guard
            .iter()
            .find(|p| p.contains("用户说: 你好"))
            .expect("main prompt for turn1 should include user line")
            .clone()
    };
    assert!(p1.contains("诗梦"));
    assert!(p1.contains("【对话硬约束】"));
    assert!(p1.contains("禁止复读开场"));
    assert!(KERNEL_DIALOGUE_GUARDRAILS.contains("禁止复读开场"));
}

#[tokio::test]
async fn mumu_without_pack_anchor_uses_kernel_default_in_prompt() {
    let roles_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles");
    let storage = RoleStorage::new(&roles_dir);
    let role = storage
        .load_role_from_dir(&roles_dir.join("mumu"))
        .expect("load mumu");
    assert!(role.reply_quality_anchor.as_deref().unwrap_or("").trim().is_empty());
    let anchor = effective_reply_quality_anchor(&role);
    assert!(anchor.contains("【回复质量锚点】（每轮须遵守）"));

    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm: Arc<dyn LlmClient> = Arc::new(CapturePromptLlm {
        prompts: prompts.clone(),
        reply: "mock".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .expect("state");

    process_message(
        &state,
        &SendMessageRequest {
            role_id: "mumu".to_string(),
            user_message: "嗯".to_string(),
            scene_id: None,
            ..Default::default()
        },
    )
    .await
    .expect("turn1");

    let p1 = {
        let guard = prompts.lock();
        guard
            .iter()
            .find(|p| p.contains("用户说: 嗯"))
            .expect("main prompt for turn1 should include user line")
            .clone()
    };
    assert!(p1.contains("禁止复述用户"));
    assert!(p1.contains("【对话硬约束】"));
}
