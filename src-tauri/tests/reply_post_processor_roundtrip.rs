//! Reply Post-Processor: display reply normalization when enabled in role pack config.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::error::Result;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::sync::Arc;
use tempfile::TempDir;

struct FixedReplyLlm {
    reply: String,
}

#[async_trait]
impl LlmClient for FixedReplyLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn write_postproc_role(dir: &TempDir, enabled: bool, raw_reply: &str) -> String {
    let role_id = "pp.test";
    let role = dir.path().join(role_id);
    std::fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "PP",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "relations": { "friend": { "initial_favorability": 50, "favor_multiplier": 1.0, "prompt_hint": "朋友" } },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 1 }
        }
    });
    std::fs::write(role.join("pipeline.ocblueprint"), bp.to_string()).unwrap();
    let cfg = serde_json::json!({
        "reply_post_processor": {
            "enabled": enabled,
            "backend": "builtin",
            "builtin": { "profile": "standard", "max_chars": 4, "strip_leading_quote": false }
        }
    });
    std::fs::write(role.join("config.json"), cfg.to_string()).unwrap();
    let _ = raw_reply;
    role_id.to_string()
}

#[tokio::test]
async fn post_processor_truncates_display_reply_when_enabled() {
    let dir = TempDir::new().unwrap();
    let raw = "abcdefgh";
    let role_id = write_postproc_role(&dir, true, raw);
    let llm: Arc<dyn LlmClient> = Arc::new(FixedReplyLlm {
        reply: raw.to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dir.path().to_path_buf())
        .await
        .expect("state");
    let resp = process_message(
        &state,
        &SendMessageRequest {
            role_id,
            user_message: "hi".to_string(),
            scene_id: Some("default".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("chat");
    assert_eq!(
        resp.reply, "abcd",
        "max_chars=4 should truncate display reply"
    );
}

#[tokio::test]
async fn post_processor_disabled_passes_raw_reply() {
    let dir = TempDir::new().unwrap();
    let raw = "abcdefgh";
    let role_id = write_postproc_role(&dir, false, raw);
    let llm: Arc<dyn LlmClient> = Arc::new(FixedReplyLlm {
        reply: raw.to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dir.path().to_path_buf())
        .await
        .expect("state");
    let resp = process_message(
        &state,
        &SendMessageRequest {
            role_id,
            user_message: "hi".to_string(),
            scene_id: Some("default".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("chat");
    assert_eq!(resp.reply, raw);
}
