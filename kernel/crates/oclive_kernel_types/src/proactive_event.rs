//! Typed proactive-turn proposals carried by the generic Event Ring.

use serde::{Deserialize, Serialize};

use crate::models::dto::TurnOrigin;

/// A registered input module proposes that the role may have a reason to speak.
pub const PROACTIVE_TURN_PROPOSED_EVENT_KIND: &str = "kernel.proactive.turn.proposed";

/// The Event decision module has admitted one proactive-turn proposal.
pub const PROACTIVE_TURN_AUTHORIZED_EVENT_KIND: &str = "kernel.proactive.turn.authorized";

/// Broad class of signal that caused a module to propose a proactive turn.
///
/// This remains distribution-neutral: device-specific payloads belong to input adapters, not the
/// kernel contract.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::ProactiveSignalKind;
///
/// assert_eq!(
///     serde_json::to_string(&ProactiveSignalKind::SensorObservation).unwrap(),
///     "\"sensor_observation\""
/// );
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProactiveSignalKind {
    SensorObservation,
    Schedule,
    InternalState,
    ModuleSignal,
}

/// A module proposal that the role may have a reason to start a non-user turn.
///
/// `observation` is untrusted evidence, not a prompt instruction. A later Turn Engine adapter must
/// render it through an origin-aware evidence section rather than presenting it as user speech.
/// Scores use the Event Ring fixed-point scale (`10_000 == 1.0`).
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{
///     models::dto::TurnOrigin, ProactiveSignalKind, ProactiveTurnProposal,
/// };
///
/// let proposal = ProactiveTurnProposal {
///     role_id: "mumu".into(),
///     session_id: Some("desktop".into()),
///     scene_id: None,
///     origin: TurnOrigin::Sensor,
///     signal_kind: ProactiveSignalKind::SensorObservation,
///     observation: "The carrier was picked up.".into(),
///     confidence_bps: 9_000,
///     urgency_bps: 6_000,
/// };
/// assert_eq!(proposal.origin, TurnOrigin::Sensor);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveTurnProposal {
    pub role_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    pub origin: TurnOrigin,
    pub signal_kind: ProactiveSignalKind,
    pub observation: String,
    pub confidence_bps: u16,
    pub urgency_bps: u16,
}

/// Event-module decision admitting one proactive-turn proposal.
///
/// This authorization is still not a reply command. The Turn Engine remains responsible for
/// deciding how an authorized observation enters role context and whether generation can run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProactiveTurnAuthorization {
    pub proposal: ProactiveTurnProposal,
    pub reply_influence_bps: u16,
}
