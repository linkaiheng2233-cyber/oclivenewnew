//! Turn-origin boundary: embedded sensor turns may generate role output without mutating the
//! ordinary user-chat state graph.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::domain::chat_engine::{
    conversation_state_role_id, process_message, process_message_with_origin,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{SendMessageRequest, TurnOrigin};
use std::sync::Arc;

#[tokio::test]
async fn sensor_turn_is_side_effect_free_while_user_turn_keeps_existing_behavior() {
    let llm = Arc::new(MockLlmClient {
        reply: "收到。[EMO]{\"labels\":[\"anger\"],\"intensity\":0.8}[/EMO]".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("AppState");

    let role_id = "mumu";
    let session_id = "turn-origin-sensor";
    let srid = conversation_state_role_id(role_id, Some(session_id));
    state
        .db_manager
        .ensure_role_runtime(&srid)
        .await
        .expect("ensure role runtime");
    let before = state
        .db_manager
        .get_role_runtime_snapshot(&srid)
        .await
        .expect("snapshot")
        .expect("runtime row");

    let req = SendMessageRequest {
        role_id: role_id.to_string(),
        user_message: "[OCLIVE_SENSOR_CONTEXT_V1] carrier_state=held".to_string(),
        session_id: Some(session_id.to_string()),
        ..Default::default()
    };
    let sensor = process_message_with_origin(&state, &req, TurnOrigin::Sensor)
        .await
        .expect("sensor turn");

    assert_eq!(sensor.reply, "收到。");
    assert_eq!(sensor.favorability_delta, 0.0);
    assert_eq!(sensor.user_message_id, None);
    assert_eq!(sensor.assistant_message_id, None);
    assert_eq!(sensor.emotion.neutral, 1.0);
    assert_eq!(state.db_manager.count_memories(&srid).await.unwrap(), 0);
    assert!(state
        .db_manager
        .get_events(&srid, 10)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        state
            .db_manager
            .get_chat_message_count(&srid)
            .await
            .unwrap(),
        0
    );

    let after_sensor = state
        .db_manager
        .get_role_runtime_snapshot(&srid)
        .await
        .expect("snapshot")
        .expect("runtime row");
    assert_eq!(after_sensor.favorability, before.favorability);
    assert_eq!(after_sensor.emotion, before.emotion);
    assert_eq!(after_sensor.relation_state, before.relation_state);
    assert_eq!(after_sensor.scene, before.scene);
    assert_eq!(after_sensor.mutable_personality, before.mutable_personality);
    assert_eq!(
        after_sensor.ephemeral_personality,
        before.ephemeral_personality
    );
    assert_eq!(after_sensor.ephemeral_ttl_turns, before.ephemeral_ttl_turns);
    assert_eq!(after_sensor.deep_latch_active, before.deep_latch_active);
    assert_eq!(after_sensor.continuity_revision, before.continuity_revision);

    let user = process_message(&state, &req)
        .await
        .expect("ordinary user turn");
    assert_eq!(user.reply, "收到。");
    assert_eq!(
        state.db_manager.get_events(&srid, 10).await.unwrap().len(),
        1
    );
    assert_eq!(
        state
            .db_manager
            .get_chat_message_count(&srid)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        state
            .db_manager
            .get_current_emotion(&srid)
            .await
            .unwrap()
            .as_deref(),
        Some("angry")
    );
}
