//! M2 slice 0 · [EMO] 派生 bot 情绪 → 六槽 current_emotion + events 行（端到端契约）。
//!
//! 主 LLM 回复带 `[EMO]` marker：labels[0]=anger 必须驱动六槽情绪图（angry），
//! 同一轮的 events 行只落六槽 token（bot_emotion=angry、user_emotion=词表先验），
//! 展示回复必须剥离 marker。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::domain::chat_engine::{conversation_state_role_id, process_message};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclive_kernel_types::models::Emotion;
use std::sync::Arc;

#[tokio::test]
async fn emo_marker_anger_drives_six_slot_emotion_and_event_row() {
    let llm = Arc::new(MockLlmClient {
        reply: "哼。\n\n[EMO]{\"labels\":[\"anger\"],\"intensity\":0.8}[/EMO]".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("AppState");

    let role_id = "mumu";
    let session_id = "emo-marker-sess";
    let res = process_message(
        &state,
        &SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: "我们吵架了".to_string(),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("process_message");

    assert!(
        !res.reply.contains("[EMO]"),
        "marker must be stripped from the display reply: {:?}",
        res.reply
    );

    let srid = conversation_state_role_id(role_id, Some(session_id));
    let current = state
        .db_manager
        .get_current_emotion(&srid)
        .await
        .expect("current emotion");
    assert_eq!(
        current.as_deref(),
        Some("angry"),
        "[EMO] labels[0]=anger must drive the six-slot emotion graph"
    );

    let events = state
        .db_manager
        .get_events(&srid, 10)
        .await
        .expect("events list");
    assert_eq!(events.len(), 1, "one turn should persist one event row");
    assert_eq!(
        events[0].bot_emotion, "angry",
        "event must carry the [EMO]-derived six-slot bot emotion"
    );
    assert!(
        events[0].user_emotion.parse::<Emotion>().is_ok(),
        "event user_emotion must stay a six-slot token, got: {:?}",
        events[0].user_emotion
    );
}

#[tokio::test]
async fn emo_derived_turn_feeds_memory_merge_via_ai_event_path() {
    use async_trait::async_trait;
    use oclive_kernel_host::domain::host_profile::{
        HostProfile, TurnThinkingDefault, TurnThinkingProfile,
    };
    use oclive_kernel_host::infrastructure::llm::LlmClient;
    use oclive_kernel_host::state::AppStateBuilder;
    use oclive_kernel_types::models::EventType;
    use oclivenewnew_tauri::error::Result;

    struct DualReplyLlm {
        main: String,
        tag: String,
    }

    #[async_trait]
    impl LlmClient for DualReplyLlm {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            Ok(self.main.clone())
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            Ok(self.tag.clone())
        }

        async fn startup_probe(&self) -> Result<()> {
            Ok(())
        }
    }

    let llm: Arc<dyn LlmClient> = Arc::new(DualReplyLlm {
        main: "哼。\n\n[EMO]{\"labels\":[\"anger\"],\"intensity\":0.8}[/EMO]".to_string(),
        tag: r#"{"event_type":"Quarrel","impact_factor":-0.8,"confidence":0.9}"#.to_string(),
    });
    // Deep mode keeps the AI event path on (host.event_impact_llm defaults
    // true), so the estimator consumes generate_tag and the turn carries the
    // full chain below; the strong_only gate itself is covered by the direct
    // atomic-turn test in memory_lifecycle_integration.
    let host = HostProfile {
        turn_thinking: TurnThinkingProfile {
            default: TurnThinkingDefault::Deep,
            ..TurnThinkingProfile::default()
        },
        ..HostProfile::default()
    };
    let state = AppStateBuilder::in_memory_test(llm, common::roles_dir(), None)
        .with_host_profile(host)
        .build()
        .await
        .expect("AppState");

    let role_id = "mumu";
    let session_id = "emo-memory-sess";
    let _res = process_message(
        &state,
        &SendMessageRequest {
            role_id: role_id.to_string(),
            user_message: "我们吵架了".to_string(),
            session_id: Some(session_id.to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("process_message");

    let srid = conversation_state_role_id(role_id, Some(session_id));

    // Chain: [EMO] anger -> six-slot bot emotion -> event row -> Quarrel event
    // -> deep full persistence -> memory merge -> LTM write (importance > 0).
    let memories = state
        .db_manager
        .count_memories(&srid)
        .await
        .expect("ltm count");
    assert_eq!(
        memories, 1,
        "memory merge must write LTM for a Quarrel turn driven by [EMO] output"
    );

    let events = state
        .db_manager
        .get_events(&srid, 10)
        .await
        .expect("events list");
    assert_eq!(events.len(), 1, "one turn should persist one event row");
    assert_eq!(
        events[0].event_type,
        EventType::Quarrel,
        "AI event path must classify the turn as Quarrel"
    );
    assert_eq!(
        events[0].bot_emotion, "angry",
        "event must carry the [EMO]-derived six-slot bot emotion"
    );
}
