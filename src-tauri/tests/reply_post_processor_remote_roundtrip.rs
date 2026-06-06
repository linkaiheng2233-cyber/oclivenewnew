//! Remote Reply Post-Processor: unreachable URL falls back to builtin rules in `process_message`.

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

fn write_remote_postproc_role(dir: &TempDir, rpc_url: &str, max_chars: u32) -> String {
    let role_id = "pp.remote";
    let role = dir.path().join(role_id);
    std::fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    let bp = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "PP Remote",
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
            "enabled": true,
            "backend": "remote",
            "remote": { "url": rpc_url, "timeout_ms": 500 },
            "builtin": { "profile": "standard", "max_chars": max_chars, "strip_leading_quote": false }
        }
    });
    std::fs::write(role.join("config.json"), cfg.to_string()).unwrap();
    role_id.to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn remote_post_processor_falls_back_to_builtin_on_bad_url() {
    let dir = TempDir::new().unwrap();
    let raw = "abcdefgh";
    let role_id = write_remote_postproc_role(&dir, "http://127.0.0.1:1/rpc", 4);
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
        "builtin max_chars=4 when remote unreachable"
    );
}
