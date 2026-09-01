//! Stage A data contracts for the future Runtime Event Stream.
//!
//! These types describe the durable outer record, host-owned Session partition mapping, and
//! consumer checkpoint control plane. They intentionally provide no storage, read API, consumer
//! loop, replay path, or turn entry point.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::EventEnvelope;

/// Current schema version for the Stage A Runtime Event Stream contracts.
pub const RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION: u16 = 1;

/// Authority and lifecycle class assigned to an event kind by the trusted host registry.
///
/// This value lives outside [`EventEnvelope`] so ordinary payload or metadata cannot self-report
/// authority. A model may propose an event, but it cannot label its output as a committed state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventSemanticType {
    Fact,
    Observation,
    Proposal,
    Decision,
    State,
    Output,
}

/// Coarse payload privacy class frozen by the trusted event-kind registry.
///
/// Credentials, complete system prompts, raw long-term-memory bodies, and unconsented media are
/// forbidden from the generic Stream even when a consumer could otherwise read this class.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventPrivacyClass {
    Public,
    Internal,
    Personal,
    Sensitive,
}

/// One host-approved channel endpoint attached to a runtime Session.
///
/// `endpoint_id` is host-issued and opaque. `adapter_id` names the registered adapter; neither
/// field grants the endpoint permission to select a Session partition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSessionEndpointBinding {
    pub endpoint_id: String,
    pub adapter_id: String,
}

/// Host-owned mapping from a runtime Session to exactly one v1 Stream partition.
///
/// The existing request `session_id` is only a compatibility lookup input. It must resolve through
/// a trusted host registry before this binding is created and must never be copied directly into
/// `partition_id`. The role-play identity selected inside a conversation is mutable state and is
/// deliberately not part of partition identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSessionPartitionBinding {
    pub schema_version: u16,
    /// Opaque Stream partition identifier issued by the host.
    pub partition_id: String,
    /// Opaque canonical Session identity issued by the host.
    pub runtime_session_id: String,
    pub role_id: String,
    /// Host-computed role-pack content/build revision used for provenance, not partition equality.
    pub role_pack_revision: String,
    /// Authorization/data-owner subject; distinct from the role-play user identity.
    pub access_subject_id: String,
    /// Explicit compatibility mapping to the current per-conversation persistence namespace.
    pub legacy_srid: String,
    /// CAS revision for Session binding changes such as endpoint or role-pack revision updates.
    pub binding_revision: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<RuntimeSessionEndpointBinding>,
}

/// Immutable durable record proposed for the future Production Stream.
///
/// The trusted host builds this outer record only after a complete Event Ring dispatch succeeds.
/// `EventEnvelope::sequence` remains process-local Ring evidence; `stream_position` is the only
/// per-partition durable order. Dispatch-local payload replacement is not promoted to durable
/// provenance: a semantic change must be emitted as a child event with its own source and cause.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeEventRecord {
    pub schema_version: u16,
    pub session_partition: String,
    pub session_binding_revision: u64,
    /// Store-assigned, monotonically increasing position within `session_partition`.
    pub stream_position: u64,
    pub semantic_type: RuntimeEventSemanticType,
    /// Versioned payload contract selected by the trusted event-kind registry.
    pub payload_schema: String,
    pub privacy_class: RuntimeEventPrivacyClass,
    /// Host policy identifier captured at append time; retention execution is not implemented.
    pub retention_policy_id: String,
    /// Optional source-scoped, opaque retry key approved by the host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_idempotency_key: Option<String>,
    pub ingested_at: DateTime<Utc>,
    pub envelope: EventEnvelope,
}

/// Read and delivery policy for one future Runtime Event Stream consumer.
///
/// This registry does not contain Event influence, proposal admission, or state-commit authority.
/// Delivery is serial within each Session partition and may be concurrent only across partitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEventStreamConsumerRegistration {
    pub schema_version: u16,
    pub consumer_id: String,
    /// Canonical dotted kinds or namespace wildcards such as `kernel.memory.*`.
    pub subscriptions: Vec<String>,
    pub semantic_types: Vec<RuntimeEventSemanticType>,
    /// Host-owned reference to the Sessions this consumer may read.
    pub session_scope_id: String,
    pub privacy_ceiling: RuntimeEventPrivacyClass,
    /// Maximum number of Session partitions processed concurrently; v1 remains serial per part.
    pub max_in_flight_partitions: u16,
    pub lease_duration_ms: u64,
    pub max_delivery_attempts: u32,
    pub retry_initial_backoff_ms: u64,
    pub retry_max_backoff_ms: u64,
}

/// Persisted phase for one consumer's progress in one Session partition.
///
/// A blocked partition never skips an event automatically. `Disabled` revokes delivery without
/// deleting progress. Retry and lease timestamps are control-plane data and do not enter character
/// context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RuntimeEventConsumerCheckpointState {
    Ready,
    Leased {
        event_id: String,
        stream_position: u64,
        attempt: u32,
        lease_expires_at: DateTime<Utc>,
    },
    RetryScheduled {
        event_id: String,
        stream_position: u64,
        attempt: u32,
        retry_not_before: DateTime<Utc>,
        error_code: String,
    },
    Blocked {
        event_id: String,
        stream_position: u64,
        attempt: u32,
        reason_code: String,
    },
    Disabled {
        reason_code: String,
    },
}

/// Durable consumer progress for one Session partition.
///
/// `next_position` starts at one. It advances to `delivered_position + 1` only after an idempotent
/// side effect is durably applied or confirmed already applied. `revision` protects checkpoint CAS;
/// `lease_epoch` rejects acknowledgements from stale workers after retry or restart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEventConsumerCheckpoint {
    pub schema_version: u16,
    pub consumer_id: String,
    pub session_partition: String,
    pub next_position: u64,
    pub revision: u64,
    pub lease_epoch: u64,
    pub state: RuntimeEventConsumerCheckpointState,
    pub updated_at: DateTime<Utc>,
}

/// Consumer-reported result for one leased delivery.
///
/// `Applied` and `AlreadyApplied` may advance the checkpoint after revision/lease validation.
/// Retryable or terminal failures never advance it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventDeliveryDisposition {
    Applied,
    AlreadyApplied,
    RetryableFailure,
    TerminalFailure,
}

/// Result bound to the exact checkpoint revision and lease epoch presented to a consumer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEventDeliveryResult {
    pub consumer_id: String,
    pub session_partition: String,
    pub event_id: String,
    pub stream_position: u64,
    pub checkpoint_revision: u64,
    pub lease_epoch: u64,
    pub disposition: RuntimeEventDeliveryDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EVENT_INFLUENCE_WEIGHT_SCALE, EVENT_RING_SCHEMA_VERSION};
    use chrono::TimeZone;
    use std::collections::BTreeMap;

    fn example_envelope() -> EventEnvelope {
        EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: "event-1".into(),
            kind: "kernel.memory.recall.candidate".into(),
            source: "module.builtin.memory_recollection".into(),
            source_weight_bps: EVENT_INFLUENCE_WEIGHT_SCALE,
            stream_key: "ring-route-only".into(),
            correlation_id: "turn-1".into(),
            causation_id: None,
            sequence: 7,
            depth: 0,
            occurred_at: Utc.timestamp_opt(10, 0).single().expect("timestamp"),
            payload: serde_json::json!({"memory_id": "memory-1"}),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn semantic_type_has_stable_snake_case_wire_names() {
        assert_eq!(
            serde_json::to_string(&RuntimeEventSemanticType::Observation).unwrap(),
            "\"observation\""
        );
        assert_eq!(
            serde_json::to_string(&RuntimeEventSemanticType::State).unwrap(),
            "\"state\""
        );
    }

    #[test]
    fn outer_record_keeps_partition_position_separate_from_ring_fields() {
        let record = RuntimeEventRecord {
            schema_version: RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION,
            session_partition: "partition-1".into(),
            session_binding_revision: 3,
            stream_position: 42,
            semantic_type: RuntimeEventSemanticType::Proposal,
            payload_schema: "kernel.memory.recall.candidate/v1".into(),
            privacy_class: RuntimeEventPrivacyClass::Personal,
            retention_policy_id: "runtime.personal.short/v1".into(),
            source_idempotency_key: Some("opaque-source-key".into()),
            ingested_at: Utc.timestamp_opt(20, 0).single().expect("timestamp"),
            envelope: example_envelope(),
        };

        let json = serde_json::to_value(record).expect("serialize record");
        assert_eq!(json["stream_position"], 42);
        assert_eq!(json["envelope"]["sequence"], 7);
        assert_eq!(json["session_partition"], "partition-1");
        assert_eq!(json["envelope"]["stream_key"], "ring-route-only");
        assert_eq!(json["semantic_type"], "proposal");
    }

    #[test]
    fn consumer_registry_contains_no_event_authority_fields() {
        let registration = RuntimeEventStreamConsumerRegistration {
            schema_version: RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION,
            consumer_id: "builtin.memory.timeline".into(),
            subscriptions: vec!["kernel.memory.*".into()],
            semantic_types: vec![RuntimeEventSemanticType::Fact],
            session_scope_id: "scope.role-owner".into(),
            privacy_ceiling: RuntimeEventPrivacyClass::Personal,
            max_in_flight_partitions: 4,
            lease_duration_ms: 30_000,
            max_delivery_attempts: 8,
            retry_initial_backoff_ms: 250,
            retry_max_backoff_ms: 30_000,
        };

        let object = serde_json::to_value(registration)
            .expect("serialize registration")
            .as_object()
            .expect("registration object")
            .clone();
        for forbidden in [
            "influence_weight_bps",
            "proposal_admission",
            "state_commit_authority",
        ] {
            assert!(!object.contains_key(forbidden));
        }
    }

    #[test]
    fn checkpoint_state_is_explicitly_tagged() {
        let state = RuntimeEventConsumerCheckpointState::RetryScheduled {
            event_id: "event-1".into(),
            stream_position: 5,
            attempt: 2,
            retry_not_before: Utc.timestamp_opt(30, 0).single().expect("timestamp"),
            error_code: "temporary_unavailable".into(),
        };
        let json = serde_json::to_value(state).expect("serialize checkpoint state");
        assert_eq!(json["state"], "retry_scheduled");
        assert_eq!(json["stream_position"], 5);
    }
}
