//! 集成测试：编排 + mock LLM（不依赖真实 Ollama）

use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[tokio::test]
async fn send_message_happy_path_mock_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟回复".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "mumu".to_string(),
        user_message: "今天天气不错".to_string(),
        scene_id: None,
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert_eq!(res.reply, "模拟回复");
    assert!(
        !res.bot_emotion.is_empty(),
        "bot_emotion should be set for UI"
    );
    assert_eq!(res.portrait_emotion, "neutral", "mock LLM tag path");
    assert!(!res.events.is_empty());
}

#[tokio::test]
async fn send_message_persists_event_to_db() {
    let llm = Arc::new(MockLlmClient {
        reply: "别生气啦".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "mumu".to_string(),
        user_message: "你太坏了，我生气了".to_string(),
        scene_id: None,
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert!(
        !res.events.is_empty(),
        "response should include detected event summary"
    );

    let stored = state
        .db_manager
        .get_events("mumu", 5)
        .await
        .expect("get_events");
    assert!(
        !stored.is_empty(),
        "events table should have a row after save_event"
    );
    assert_eq!(
        format!("{:?}", stored[0].event_type),
        res.events[0].event_type,
        "DB event_type should match API response"
    );
}
