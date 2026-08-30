#![allow(clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{EventModule, EventModuleRegistrar};
use oclive_kernel_host::domain::EventRing;
use oclive_kernel_types::{
    EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput, Result,
};

struct PublicSource;
struct PublicConsumer;

#[async_trait]
impl EventModule for PublicSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.public_source".into(),
            emissions: vec!["kernel.public.input".into()],
            influence_weight_bps: 6_400,
            ..Default::default()
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

#[async_trait]
impl EventModule for PublicConsumer {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.public_consumer".into(),
            subscriptions: vec!["kernel.public.input".into()],
            emissions: vec!["kernel.public.derived".into()],
            influence_weight_bps: 8_200,
            ..Default::default()
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput {
            emitted: vec![EventDraft {
                kind: "kernel.public.derived".into(),
                payload: serde_json::json!({"accepted": true}),
                metadata: Default::default(),
            }],
            ..Default::default()
        })
    }
}

#[tokio::test]
async fn exported_registry_emitter_and_envelope_form_a_complete_public_path() {
    let ring = EventRing::new();
    let source = ring
        .register_event_module(Arc::new(PublicSource))
        .expect("register public source");
    ring.register_event_module(Arc::new(PublicConsumer))
        .expect("register public consumer");

    let result = source
        .emit(
            "public-stream",
            Some("turn-public-1"),
            EventDraft {
                kind: "kernel.public.input".into(),
                payload: serde_json::json!({"value": 1}),
                metadata: Default::default(),
            },
        )
        .await
        .expect("dispatch public event");

    assert_eq!(result.primary.source, "module.test.public_source");
    assert_eq!(result.primary.source_weight_bps, 6_400);
    assert_eq!(result.emitted.len(), 1);
    assert_eq!(result.emitted[0].source, "module.test.public_consumer");
    assert_eq!(result.emitted[0].source_weight_bps, 8_200);
    assert_eq!(
        result.emitted[0].causation_id.as_deref(),
        Some(result.primary.event_id.as_str())
    );
}
