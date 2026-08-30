//! Source-bound proactive proposals and Event-module admission decisions.

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    models::dto::TurnOrigin, AppError, EventDraft, EventEnvelope, EventModuleDeclaration,
    EventModuleOutput, ProactiveTurnAuthorization, ProactiveTurnProposal, Result,
    EVENT_INFLUENCE_WEIGHT_SCALE, PROACTIVE_TURN_AUTHORIZED_EVENT_KIND,
    PROACTIVE_TURN_PROPOSED_EVENT_KIND,
};

use super::EventRing;

pub const PROACTIVE_TURN_DECISION_MODULE_ID: &str = "builtin.proactive_turn_decision";

const PROACTIVE_TURN_DECISION_EVENT_SOURCE: &str = "module.builtin.proactive_turn_decision";
const MIN_PROACTIVE_TURN_INFLUENCE_BPS: u16 = 5_000;
const MAX_CONTEXT_ID_BYTES: usize = 160;
const MAX_OBSERVATION_BYTES: usize = 2_048;

/// One-shot host permit proving that the built-in Event decision module authorized a proposal.
///
/// Fields are private and the type is not cloneable or deserializable, so callers cannot forge or
/// replay an authorization by constructing the public wire DTO directly.
pub struct ProactiveTurnPermit {
    authorization: ProactiveTurnAuthorization,
    event_id: String,
    correlation_id: String,
}

impl ProactiveTurnPermit {
    /// Returns the admitted proposal and its computed reply influence for inspection.
    #[must_use]
    pub fn authorization(&self) -> &ProactiveTurnAuthorization {
        &self.authorization
    }

    /// Event Ring identity of the authorization event.
    #[must_use]
    pub fn event_id(&self) -> &str {
        self.event_id.as_str()
    }

    /// Correlation inherited from the source proposal and later reused by the Turn Engine.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    pub(crate) fn into_parts(self) -> (ProactiveTurnAuthorization, String, String) {
        (self.authorization, self.event_id, self.correlation_id)
    }
}

struct ProactiveTurnDecisionModule;

#[async_trait]
impl EventModule for ProactiveTurnDecisionModule {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: PROACTIVE_TURN_DECISION_MODULE_ID.into(),
            subscriptions: vec![PROACTIVE_TURN_PROPOSED_EVENT_KIND.into()],
            emissions: vec![PROACTIVE_TURN_AUTHORIZED_EVENT_KIND.into()],
            priority: 100,
        }
    }

    async fn handle(&self, event: &EventEnvelope) -> Result<EventModuleOutput> {
        let proposal: ProactiveTurnProposal = serde_json::from_value(event.payload.clone())?;
        validate_proposal(&proposal)?;
        let reply_influence_bps = proposal_influence_bps(event.source_weight_bps, &proposal);
        if reply_influence_bps < MIN_PROACTIVE_TURN_INFLUENCE_BPS {
            return Ok(EventModuleOutput::default());
        }

        let authorization = ProactiveTurnAuthorization {
            proposal,
            reply_influence_bps,
        };
        Ok(EventModuleOutput {
            emitted: vec![EventDraft {
                kind: PROACTIVE_TURN_AUTHORIZED_EVENT_KIND.into(),
                payload: serde_json::to_value(authorization)?,
                metadata: Default::default(),
            }],
            ..Default::default()
        })
    }
}

pub(crate) fn register_proactive_turn_decision_module(
    ring: &EventRing,
) -> std::result::Result<(), String> {
    ring.register_event_module(Arc::new(ProactiveTurnDecisionModule))?;
    Ok(())
}

/// Submits a proactive-turn proposal through a source-bound Event Ring handle.
///
/// The caller cannot provide an event source or influence weight. The returned value exists only
/// when the built-in Event decision module admitted the proposal. This function does not invoke
/// the Turn Engine or mutate chat state.
///
/// # Errors
///
/// Returns Event Ring validation or dispatch errors, including malformed proposal payloads and a
/// source emitter that did not declare the proactive proposal event kind.
pub async fn propose_proactive_turn(
    emitter: &dyn EventEmitter,
    stream_key: &str,
    correlation_id: &str,
    proposal: ProactiveTurnProposal,
) -> Result<Option<ProactiveTurnPermit>> {
    let dispatched = emitter
        .emit(
            stream_key,
            Some(correlation_id),
            EventDraft {
                kind: PROACTIVE_TURN_PROPOSED_EVENT_KIND.into(),
                payload: serde_json::to_value(proposal)?,
                metadata: Default::default(),
            },
        )
        .await?;

    let Some(event) = dispatched.emitted.into_iter().find(|event| {
        event.kind == PROACTIVE_TURN_AUTHORIZED_EVENT_KIND
            && event.source == PROACTIVE_TURN_DECISION_EVENT_SOURCE
            && event.causation_id.as_deref() == Some(dispatched.primary.event_id.as_str())
    }) else {
        return Ok(None);
    };
    let authorization: ProactiveTurnAuthorization = serde_json::from_value(event.payload)?;
    validate_proposal(&authorization.proposal)?;
    if !(MIN_PROACTIVE_TURN_INFLUENCE_BPS..=EVENT_INFLUENCE_WEIGHT_SCALE)
        .contains(&authorization.reply_influence_bps)
    {
        return Err(invalid_proposal());
    }
    Ok(Some(ProactiveTurnPermit {
        authorization,
        event_id: event.event_id,
        correlation_id: event.correlation_id,
    }))
}

fn proposal_influence_bps(source_weight_bps: u16, proposal: &ProactiveTurnProposal) -> u16 {
    let evidence_bps =
        (u32::from(proposal.confidence_bps) + u32::from(proposal.urgency_bps) * 2) / 3;
    let weighted =
        u32::from(source_weight_bps) * evidence_bps / u32::from(EVENT_INFLUENCE_WEIGHT_SCALE);
    weighted.min(u32::from(EVENT_INFLUENCE_WEIGHT_SCALE)) as u16
}

fn validate_proposal(proposal: &ProactiveTurnProposal) -> Result<()> {
    validate_context_id(&proposal.role_id)?;
    if let Some(session_id) = proposal.session_id.as_deref() {
        validate_context_id(session_id)?;
    }
    if let Some(scene_id) = proposal.scene_id.as_deref() {
        validate_context_id(scene_id)?;
    }
    if proposal.origin == TurnOrigin::User
        || proposal.observation.is_empty()
        || proposal.observation != proposal.observation.trim()
        || proposal.observation.len() > MAX_OBSERVATION_BYTES
        || proposal.observation.contains('\0')
        || proposal.confidence_bps > EVENT_INFLUENCE_WEIGHT_SCALE
        || proposal.urgency_bps > EVENT_INFLUENCE_WEIGHT_SCALE
    {
        return Err(invalid_proposal());
    }
    Ok(())
}

fn validate_context_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value != value.trim()
        || value.len() > MAX_CONTEXT_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(invalid_proposal());
    }
    Ok(())
}

fn invalid_proposal() -> AppError {
    AppError::InvalidParameter("event_ring: invalid proactive turn proposal".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::{
        EventModuleRegistryPolicy, ProactiveSignalKind, EVENT_RING_SCHEMA_VERSION,
    };

    struct ProactiveSource;
    struct UnrelatedSource;

    #[async_trait]
    impl EventModule for ProactiveSource {
        fn declaration(&self) -> EventModuleDeclaration {
            EventModuleDeclaration {
                module_id: "test.proactive_source".into(),
                emissions: vec![PROACTIVE_TURN_PROPOSED_EVENT_KIND.into()],
                ..Default::default()
            }
        }

        async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
            Ok(EventModuleOutput::default())
        }
    }

    #[async_trait]
    impl EventModule for UnrelatedSource {
        fn declaration(&self) -> EventModuleDeclaration {
            EventModuleDeclaration {
                module_id: "test.unrelated_source".into(),
                emissions: vec!["kernel.test.unrelated".into()],
                ..Default::default()
            }
        }

        async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
            Ok(EventModuleOutput::default())
        }
    }

    fn proposal() -> ProactiveTurnProposal {
        ProactiveTurnProposal {
            role_id: "mumu".into(),
            session_id: Some("desktop".into()),
            scene_id: Some("default".into()),
            origin: TurnOrigin::Sensor,
            signal_kind: ProactiveSignalKind::SensorObservation,
            observation: "The carrier was picked up.".into(),
            confidence_bps: 9_000,
            urgency_bps: 8_000,
        }
    }

    fn source_emitter(
        ring: &EventRing,
        influence_weight_bps: u16,
    ) -> Result<Arc<dyn EventEmitter>> {
        ring.register_event_module_with_policy(
            Arc::new(ProactiveSource),
            EventModuleRegistryPolicy {
                influence_weight_bps,
                ..Default::default()
            },
        )
        .map_err(AppError::InvalidParameter)
    }

    #[tokio::test]
    async fn admitted_proposal_has_decision_source_and_causation() -> Result<()> {
        let ring = EventRing::new();
        register_proactive_turn_decision_module(&ring).map_err(AppError::InvalidParameter)?;
        let emitter = source_emitter(&ring, 10_000)?;

        let permit = propose_proactive_turn(
            emitter.as_ref(),
            "role:mumu:desktop",
            "proactive-1",
            proposal(),
        )
        .await?
        .expect("high-influence proposal should be authorized");

        assert_eq!(permit.authorization().reply_influence_bps, 8_333);
        assert_eq!(permit.authorization().proposal.role_id, "mumu");
        assert_eq!(permit.correlation_id(), "proactive-1");
        let events = ring.recent_events(2);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].schema_version, EVENT_RING_SCHEMA_VERSION);
        assert_eq!(events[0].source, "module.test.proactive_source");
        assert_eq!(events[1].source, PROACTIVE_TURN_DECISION_EVENT_SOURCE);
        assert_eq!(
            events[1].causation_id.as_deref(),
            Some(events[0].event_id.as_str())
        );
        Ok(())
    }

    #[tokio::test]
    async fn registry_weight_can_keep_proposal_unadmitted() -> Result<()> {
        let ring = EventRing::new();
        register_proactive_turn_decision_module(&ring).map_err(AppError::InvalidParameter)?;
        let emitter = source_emitter(&ring, 4_000)?;

        let permit = propose_proactive_turn(
            emitter.as_ref(),
            "role:mumu:desktop",
            "proactive-low-weight",
            proposal(),
        )
        .await?;

        assert!(permit.is_none());
        assert_eq!(ring.recent_events(2).len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn user_origin_is_rejected_without_committing_ring_history() -> Result<()> {
        let ring = EventRing::new();
        register_proactive_turn_decision_module(&ring).map_err(AppError::InvalidParameter)?;
        let emitter = source_emitter(&ring, 10_000)?;
        let mut invalid = proposal();
        invalid.origin = TurnOrigin::User;

        let result = propose_proactive_turn(
            emitter.as_ref(),
            "role:mumu:desktop",
            "proactive-user",
            invalid,
        )
        .await;

        assert!(result.is_err());
        assert!(ring.recent_events(2).is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn source_without_proactive_emission_scope_cannot_use_ingress() -> Result<()> {
        let ring = EventRing::new();
        register_proactive_turn_decision_module(&ring).map_err(AppError::InvalidParameter)?;
        let emitter = ring
            .register_event_module(Arc::new(UnrelatedSource))
            .map_err(AppError::InvalidParameter)?;

        let result = propose_proactive_turn(
            emitter.as_ref(),
            "role:mumu:desktop",
            "proactive-out-of-scope",
            proposal(),
        )
        .await;

        assert!(result.is_err());
        assert!(ring.recent_events(2).is_empty());
        Ok(())
    }
}
