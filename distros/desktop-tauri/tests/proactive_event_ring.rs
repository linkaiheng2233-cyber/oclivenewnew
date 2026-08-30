//! Proactive Event Ring proposals are event-only until the Turn Engine explicitly consumes an
//! authorization; they must not synthesize user chat or memory rows.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_host::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_host::domain::event_ring::propose_proactive_turn;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::TurnOrigin;
use oclive_kernel_types::{
    AppError, EventEnvelope, EventModuleDeclaration, EventModuleOutput, EventModuleRegistryPolicy,
    ProactiveSignalKind, ProactiveTurnProposal, Result, PROACTIVE_TURN_PROPOSED_EVENT_KIND,
};

struct EmbeddedSensorSource;

#[async_trait]
impl EventModule for EmbeddedSensorSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "embedded.sensor_test".into(),
            emissions: vec![PROACTIVE_TURN_PROPOSED_EVENT_KIND.into()],
            ..Default::default()
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

#[tokio::test]
async fn authorized_proactive_event_does_not_pollute_user_chat_state() -> Result<()> {
    let state = AppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "unused".into(),
        }),
        common::roles_dir(),
    )
    .await?;
    let emitter: Arc<dyn EventEmitter> = state
        .event_ring
        .register_event_module_with_policy(
            Arc::new(EmbeddedSensorSource),
            EventModuleRegistryPolicy {
                influence_weight_bps: 9_000,
                ..Default::default()
            },
        )
        .map_err(AppError::InvalidParameter)?;
    let role_id = "mumu";
    let session_id = "proactive-event-ring";
    let state_role_id = conversation_state_role_id(role_id, Some(session_id));

    let authorization = propose_proactive_turn(
        emitter.as_ref(),
        "role:mumu:proactive-event-ring",
        "sensor-observation-1",
        ProactiveTurnProposal {
            role_id: role_id.into(),
            session_id: Some(session_id.into()),
            scene_id: Some("default".into()),
            origin: TurnOrigin::Sensor,
            signal_kind: ProactiveSignalKind::SensorObservation,
            observation: "The carrier was picked up.".into(),
            confidence_bps: 9_000,
            urgency_bps: 8_000,
        },
    )
    .await?;

    assert!(authorization.is_some());
    assert_eq!(
        state
            .db_manager
            .get_chat_message_count(&state_role_id)
            .await?,
        0
    );
    assert_eq!(state.db_manager.count_memories(&state_role_id).await?, 0);
    assert!(state
        .db_manager
        .get_events(&state_role_id, 10)
        .await?
        .is_empty());
    Ok(())
}
