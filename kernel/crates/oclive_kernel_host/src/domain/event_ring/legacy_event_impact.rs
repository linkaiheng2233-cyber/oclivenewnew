//! Compatibility bridge from the legacy dialogue `event` slot into the generic Event Ring.

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    EventDraft, EventEnvelope, EventImpactEstimate, EventModuleDeclaration, EventModuleOutput,
    Result,
};

use super::EventRing;

pub const LEGACY_EVENT_IMPACT_KIND: &str = "kernel.chat.event_impact.estimated";
pub const LEGACY_EVENT_IMPACT_MODULE_ID: &str = "builtin.event_impact_bridge";

struct LegacyEventImpactSource;

#[async_trait]
impl EventModule for LegacyEventImpactSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: LEGACY_EVENT_IMPACT_MODULE_ID.into(),
            emissions: vec![LEGACY_EVENT_IMPACT_KIND.into()],
            ..Default::default()
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

pub(crate) fn register_legacy_event_impact_source(
    ring: &EventRing,
) -> std::result::Result<Arc<dyn EventEmitter>, String> {
    ring.register_event_module(Arc::new(LegacyEventImpactSource))
}

/// Publishes the legacy event-impact result and decodes the transformed primary payload.
///
/// With no registered Event Ring modules this is a serialization round trip and preserves all
/// estimate fields. Modules may intentionally replace the payload with another valid
/// [`EventImpactEstimate`].
///
/// # Errors
///
/// Returns Event Ring validation/module errors or a serialization error when a module produces an
/// invalid event-impact payload.
pub async fn publish_legacy_event_impact(
    emitter: &dyn EventEmitter,
    stream_key: &str,
    correlation_id: Option<&str>,
    estimate: EventImpactEstimate,
) -> Result<EventImpactEstimate> {
    let payload = serde_json::to_value(estimate)?;
    let dispatched = emitter
        .emit(
            stream_key,
            correlation_id,
            EventDraft {
                kind: LEGACY_EVENT_IMPACT_KIND.into(),
                payload,
                metadata: Default::default(),
            },
        )
        .await?;
    Ok(serde_json::from_value(dispatched.primary.payload)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_contracts::{EventModule, EventModuleRegistrar};
    use oclive_kernel_types::{
        EventEnvelope, EventModuleDeclaration, EventModuleOutput, EventType,
    };
    use std::sync::Arc;

    struct OverrideImpact;

    #[async_trait]
    impl EventModule for OverrideImpact {
        fn declaration(&self) -> EventModuleDeclaration {
            EventModuleDeclaration {
                module_id: "test.impact.override".into(),
                subscriptions: vec![LEGACY_EVENT_IMPACT_KIND.into()],
                priority: 0,
                ..Default::default()
            }
        }

        async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
            Ok(EventModuleOutput {
                payload: Some(serde_json::to_value(EventImpactEstimate {
                    event_type: EventType::Apology,
                    impact_factor: 0.8,
                    confidence: 0.9,
                })?),
                ..Default::default()
            })
        }
    }

    #[tokio::test]
    async fn empty_ring_preserves_legacy_estimate() -> Result<()> {
        let ring = EventRing::new();
        let emitter = register_legacy_event_impact_source(&ring)
            .map_err(oclive_kernel_types::AppError::InvalidParameter)?;
        let estimate = EventImpactEstimate {
            event_type: EventType::Praise,
            impact_factor: 0.4,
            confidence: 0.75,
        };

        let output =
            publish_legacy_event_impact(emitter.as_ref(), "chat:mumu", None, estimate.clone())
                .await?;

        assert_eq!(output.event_type, estimate.event_type);
        assert_eq!(output.impact_factor, estimate.impact_factor);
        assert_eq!(output.confidence, estimate.confidence);
        let recent = ring.recent_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].kind, LEGACY_EVENT_IMPACT_KIND);
        Ok(())
    }

    #[tokio::test]
    async fn registered_module_can_replace_legacy_estimate_payload() -> Result<()> {
        let ring = EventRing::new();
        let emitter = register_legacy_event_impact_source(&ring)
            .map_err(oclive_kernel_types::AppError::InvalidParameter)?;
        ring.register_event_module(Arc::new(OverrideImpact))
            .map_err(oclive_kernel_types::AppError::InvalidParameter)?;
        let estimate = EventImpactEstimate {
            event_type: EventType::Ignore,
            impact_factor: 0.0,
            confidence: 0.0,
        };

        let output =
            publish_legacy_event_impact(emitter.as_ref(), "chat:mumu", Some("turn-1"), estimate)
                .await?;

        assert_eq!(output.event_type, EventType::Apology);
        assert_eq!(output.impact_factor, 0.8);
        assert_eq!(output.confidence, 0.9);
        Ok(())
    }
}
