//! Reply-mode contract: protocol markers are presentation-only and adult turns stay isolated.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{
    AdultInteractionAction, AdultInteractionRequest, SendMessageRequest,
};
use oclivenewnew_tauri::error::Result;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

struct RecordingLlm {
    reply: String,
    prompts: Mutex<Vec<String>>,
}

impl RecordingLlm {
    fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: reply.into(),
            prompts: Mutex::new(Vec::new()),
        }
    }

    fn prompts(&self) -> Vec<String> {
        self.prompts.lock().expect("prompt lock").clone()
    }
}

#[async_trait]
impl LlmClient for RecordingLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts
            .lock()
            .expect("prompt lock")
            .push(prompt.to_string());
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn write_reply_mode_role(roles: &TempDir, with_adult_extension: bool) -> String {
    let role_id = "reply-mode.test";
    let role_dir = roles.path().join(role_id);
    std::fs::create_dir_all(role_dir.join("scenes").join("default")).unwrap();
    let blueprint = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": role_id,
            "name": "Reply Mode Test",
            "version": "0.1.0",
            "author": "test",
            "description": "reply mode integration fixture",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
                "friend": {
                    "initial_favorability": 50,
                    "favor_multiplier": 1.0,
                    "prompt_hint": "friend"
                }
            },
            "default_relation": "friend",
            "scenes": ["default"]
        },
        "slot_registry": {
            "llm": {
                "type": "llm",
                "label": "LLM",
                "backend": "ollama",
                "position": 1
            }
        }
    });
    std::fs::write(role_dir.join("pipeline.ocblueprint"), blueprint.to_string()).unwrap();
    let config = serde_json::json!({
        "chat_storage": { "mirror": false },
        "reply_mode": {
            "mode": "burst",
            "segments": 3,
            "separator": "+++",
            "delays_ms": [0, 120, 240],
            "streaming": "live"
        }
    });
    std::fs::write(role_dir.join("config.json"), config.to_string()).unwrap();
    if with_adult_extension {
        let adult_extension = serde_json::json!({
            "schema_version": 1,
            "character_is_adult": true,
            "persona": "adult test persona",
            "dialogue_guidance": "adult test guidance",
            "pacing": {
                "mode": "ai",
                "suggested_interval_ms": 4000
            },
            "scenes": {}
        });
        std::fs::write(
            role_dir.join("adult_extension.json"),
            adult_extension.to_string(),
        )
        .unwrap();
    }
    role_id.to_string()
}

#[tokio::test]
async fn ordinary_burst_strips_markers_before_response_chat_and_short_term_memory() {
    let roles = TempDir::new().unwrap();
    let role_id = write_reply_mode_role(&roles, false);
    let llm = Arc::new(RecordingLlm::new(
        "first burst\n+++\nsecond burst\n+++\nthird burst",
    ));
    let state = AppState::new_in_memory_with_llm(llm.clone(), roles.path().to_path_buf())
        .await
        .expect("state");

    let response = process_message(
        &state,
        &SendMessageRequest {
            role_id: role_id.clone(),
            user_message: "hello".to_string(),
            scene_id: Some("default".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("chat");

    assert_eq!(response.reply, "first burst\nsecond burst\nthird burst");
    let presentation = response.reply_presentation.expect("presentation");
    assert_eq!(
        presentation.segments,
        vec!["first burst", "second burst", "third burst"]
    );
    assert_eq!(presentation.delays_ms, vec![0, 120, 240]);

    let memory_turns = state
        .db_manager
        .list_short_term_turns(&role_id)
        .await
        .expect("short-term memory turns");
    let memory_reply = &memory_turns.last().expect("short-term memory turn").1;
    assert_eq!(memory_reply, response.reply.as_str());
    assert!(!memory_reply.contains("+++"));

    let messages = state
        .conversation_store
        .fetch_messages(&role_id, 10, 0)
        .await
        .expect("chat messages");
    let assistant = messages
        .iter()
        .find(|message| message.sender == "assistant")
        .expect("assistant chat row");
    assert_eq!(assistant.content, response.reply.as_str());
    assert!(!assistant.content.contains("+++"));
    let metadata: serde_json::Value =
        serde_json::from_str(assistant.metadata.as_deref().expect("assistant metadata"))
            .expect("metadata json");
    assert_eq!(
        metadata["reply_segments"],
        serde_json::json!(["first burst", "second burst", "third burst"])
    );
    assert!(llm
        .prompts()
        .iter()
        .any(|prompt| prompt.contains("绝对不允许省略分隔符")));
}

#[tokio::test]
async fn adult_structured_turn_does_not_inject_or_apply_reply_mode() {
    let roles = TempDir::new().unwrap();
    let role_id = write_reply_mode_role(&roles, true);
    let llm = Arc::new(RecordingLlm::new(
        r#"{"dialogue":"adult dialogue","narration":"adult narration","interaction_state":"active","next_beat_interval_ms":1200}"#,
    ));
    let state = AppState::new_in_memory_with_llm(llm.clone(), roles.path().to_path_buf())
        .await
        .expect("state");

    let response = process_message(
        &state,
        &SendMessageRequest {
            role_id,
            user_message: "continue".to_string(),
            scene_id: Some("default".to_string()),
            adult: Some(AdultInteractionRequest {
                confirmed_adult: true,
                global_enabled: true,
                role_enabled: true,
                interaction_active: false,
                action: AdultInteractionAction::Message,
                stage: None,
            }),
            ..Default::default()
        },
    )
    .await
    .expect("adult chat");

    assert!(response.adult_beat.is_some());
    assert!(response.reply_presentation.is_none());
    let prompts = llm.prompts();
    assert!(prompts
        .iter()
        .any(|prompt| prompt.contains("本轮最终输出契约")));
    assert!(prompts
        .iter()
        .all(|prompt| !prompt.contains("绝对不允许省略分隔符")));
}
