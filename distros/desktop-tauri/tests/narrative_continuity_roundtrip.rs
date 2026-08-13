//! Narrative continuity through the real multi-turn `process_message` path.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use async_trait::async_trait;
use oclive_kernel_host::domain::chat_engine::{conversation_state_role_id, process_message};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::error::Result;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

struct SequencePromptLlm {
    prompts: Arc<Mutex<Vec<String>>>,
    replies: Mutex<VecDeque<String>>,
}

#[async_trait]
impl LlmClient for SequencePromptLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts.lock().push(prompt.to_string());
        Ok(self
            .replies
            .lock()
            .pop_front()
            .expect("one deterministic reply per main generation"))
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn request(message: &str, scene_id: &str) -> SendMessageRequest {
    SendMessageRequest {
        role_id: "mumu".to_string(),
        user_message: message.to_string(),
        scene_id: Some(scene_id.to_string()),
        session_id: Some("continuity-roundtrip".to_string()),
        ..Default::default()
    }
}

#[tokio::test]
async fn continuity_keeps_transitions_and_reselects_state_across_turns() {
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let replies = [
        "那就继续坐着聊会儿吧。",
        "她抱着枕头，走进卧室，又回头看了你一眼。",
        "她靠在床头继续陪你聊天。",
        "到了学校，她先安静地坐下来。",
    ]
    .into_iter()
    .map(|reply| format!("{reply}\n[EMO]{{\"labels\":[\"neutral\"],\"intensity\":0.3}}[/EMO]"))
    .collect();
    let llm: Arc<dyn LlmClient> = Arc::new(SequencePromptLlm {
        prompts: prompts.clone(),
        replies: Mutex::new(replies),
    });
    let mut host = HostProfile::default();
    host.turn_thinking.fast_skip_complex_emotion = false;
    let state = AppStateBuilder::in_memory_test(llm, common::roles_dir(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("state");
    let srid = conversation_state_role_id("mumu", Some("continuity-roundtrip"));

    state
        .db_manager
        .ensure_role_runtime(&srid)
        .await
        .expect("ensure runtime");
    let initial_revision = state
        .db_manager
        .set_narrative_continuity_state(&srid, "home", "sofa_lounge", 0)
        .await
        .expect("seed continuity")
        .expect("seed revision");

    process_message(&state, &request("先在这里聊一会儿。", "home"))
        .await
        .expect("same-state turn");
    assert_eq!(
        state
            .db_manager
            .get_narrative_continuity_state(&srid)
            .await
            .expect("read after same-state turn"),
        Some(("home".into(), "sofa_lounge".into(), initial_revision)),
        "a reply without a configured movement marker must retain the state"
    );

    process_message(&state, &request("有点困了。", "home"))
        .await
        .expect("transition turn");
    let bedroom_revision = initial_revision + 1;
    assert_eq!(
        state
            .db_manager
            .get_narrative_continuity_state(&srid)
            .await
            .expect("read after transition"),
        Some(("home".into(), "bedroom_wind_down".into(), bedroom_revision,)),
        "the explicit configured action must advance the persisted state"
    );

    process_message(&state, &request("再说一会儿。", "home"))
        .await
        .expect("post-transition turn");
    process_message(&state, &request("到学校以后呢？", "school"))
        .await
        .expect("scene-switch turn");

    let captured = prompts.lock();
    assert_eq!(captured.len(), 4, "expected one main prompt per turn");
    assert!(
        captured[0].contains("当前子地点：客厅")
            && captured[0].contains("环境锚点：柔软的沙发和茶几"),
        "the seeded home state must be injected into the first prompt"
    );
    assert!(
        captured[1].contains("当前子地点：客厅"),
        "the state must remain stable until the transition reply completes"
    );
    assert!(
        captured[2].contains("当前子地点：卧室")
            && captured[2].contains("环境锚点：床头暖灯和叠好的被子"),
        "the next prompt must consume the transitioned bedroom state"
    );
    assert!(
        ["教室", "教学楼走廊", "食堂", "图书馆", "校门口"]
            .iter()
            .any(|location| captured[3].contains(&format!("当前子地点：{location}"))),
        "switching to school must select one of the school continuity states"
    );
    assert!(
        !captured[3].contains("床头暖灯和叠好的被子"),
        "the previous scene's bedroom anchor must not leak into school"
    );

    let final_state = state
        .db_manager
        .get_narrative_continuity_state(&srid)
        .await
        .expect("read final continuity")
        .expect("school continuity state");
    assert_eq!(final_state.0, "school");
    assert!(final_state.2 > bedroom_revision);
}
