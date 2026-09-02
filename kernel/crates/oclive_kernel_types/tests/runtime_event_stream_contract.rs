use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use oclive_kernel_types::{
    RuntimeEventDeliveryDisposition, RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ModelPhase {
    Ready,
    Leased,
    RetryScheduled,
    Blocked,
    Disabled,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
struct ModelCheckpoint {
    phase: ModelPhase,
    next_position: u64,
    revision: u64,
    lease_epoch: u64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ModelAction {
    Claim {
        stream_position: u64,
    },
    Finish {
        stream_position: u64,
        checkpoint_revision: u64,
        lease_epoch: u64,
        disposition: RuntimeEventDeliveryDisposition,
    },
    LeaseExpired {
        checkpoint_revision: u64,
        lease_epoch: u64,
    },
    RetryDue,
    OperatorRetry,
    RetentionGap,
    Disable,
}

#[derive(Debug, Deserialize)]
struct ExpectedTransition {
    accepted: bool,
    phase: ModelPhase,
    next_position: u64,
    revision: u64,
    lease_epoch: u64,
}

#[derive(Debug, Deserialize)]
struct TransitionCase {
    id: String,
    start: ModelCheckpoint,
    action: ModelAction,
    expected: ExpectedTransition,
}

#[derive(Debug, Deserialize)]
struct TransitionContract {
    schema_version: u16,
    cases: Vec<TransitionCase>,
}

#[derive(Debug, Deserialize)]
struct StageA2Surface {
    id: String,
    evidence_path: String,
    required_tokens: Vec<String>,
    creates_turn_pipeline: bool,
    may_directly_commit_authoritative_state: bool,
    has_durable_output_receipt: bool,
}

#[derive(Debug, Deserialize)]
struct StageA2Evidence {
    id: String,
    evidence_path: String,
    required_tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StageA2StorageDecision {
    first_backend: String,
    main_state_database: String,
    production_stream_database: String,
    trace_database: String,
    all_paths_must_be_distinct: bool,
    cross_database_atomicity_claimed: bool,
    main_database_owns_producer_outbox: bool,
    stream_database_owns: Vec<String>,
    external_broker_triggers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StageA2PublicationCase {
    id: String,
    transaction_domain: String,
    atomic_writes: Vec<String>,
    publication_gate: String,
    failure_rule: String,
}

#[derive(Debug, Deserialize)]
struct StageA2IdempotencyContract {
    scope: Vec<String>,
    maximum_key_bytes: u16,
    empty_forbidden: bool,
    control_characters_forbidden: bool,
    trim_or_case_fold: bool,
    conflicting_content: String,
    credential_material_forbidden: bool,
}

#[derive(Debug, Deserialize)]
struct StageA2RetentionContract {
    normal_expiry_entry: String,
    normal_expiry_tombstone_fields: Vec<String>,
    normal_expiry_may_advance_checkpoint: bool,
    unexpected_physical_gap: String,
    required_event_classes_wait_for_required_consumers: bool,
    session_erasure_hard_deletes_partition: bool,
}

#[derive(Debug, Deserialize)]
struct StageA2ErasureContract {
    states: Vec<String>,
    required_order: Vec<String>,
    required_domains: Vec<String>,
    completion_requires_all_required_domains: bool,
    retry_is_idempotent: bool,
    completion_receipt_forbidden_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StageA2ReviewContract {
    schema_version: u16,
    stage: String,
    production_runtime_enabled: bool,
    observed_surfaces: Vec<StageA2Surface>,
    review_evidence: Vec<StageA2Evidence>,
    storage_decision: StageA2StorageDecision,
    publication_cases: Vec<StageA2PublicationCase>,
    idempotency_contract: StageA2IdempotencyContract,
    retention_contract: StageA2RetentionContract,
    session_erasure_contract: StageA2ErasureContract,
    known_gaps: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn assert_evidence_tokens(id: &str, evidence_path: &str, required_tokens: &[String]) {
    let path = repo_root().join(evidence_path);
    let evidence = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    for token in required_tokens {
        assert!(
            evidence.contains(token),
            "{id} missing evidence token {token:?}"
        );
    }
}

fn apply_reference_transition(
    mut checkpoint: ModelCheckpoint,
    action: &ModelAction,
) -> (bool, ModelCheckpoint) {
    let accepted = match action {
        ModelAction::Claim { stream_position }
            if checkpoint.phase == ModelPhase::Ready
                && *stream_position == checkpoint.next_position =>
        {
            checkpoint.phase = ModelPhase::Leased;
            checkpoint.revision += 1;
            checkpoint.lease_epoch += 1;
            true
        }
        ModelAction::Finish {
            stream_position,
            checkpoint_revision,
            lease_epoch,
            disposition,
        } if checkpoint.phase == ModelPhase::Leased
            && *stream_position == checkpoint.next_position
            && *checkpoint_revision == checkpoint.revision
            && *lease_epoch == checkpoint.lease_epoch =>
        {
            match disposition {
                RuntimeEventDeliveryDisposition::Applied
                | RuntimeEventDeliveryDisposition::AlreadyApplied => {
                    checkpoint.phase = ModelPhase::Ready;
                    checkpoint.next_position = stream_position + 1;
                }
                RuntimeEventDeliveryDisposition::RetryableFailure => {
                    checkpoint.phase = ModelPhase::RetryScheduled;
                }
                RuntimeEventDeliveryDisposition::TerminalFailure => {
                    checkpoint.phase = ModelPhase::Blocked;
                }
            }
            checkpoint.revision += 1;
            true
        }
        ModelAction::LeaseExpired {
            checkpoint_revision,
            lease_epoch,
        } if checkpoint.phase == ModelPhase::Leased
            && *checkpoint_revision == checkpoint.revision
            && *lease_epoch == checkpoint.lease_epoch =>
        {
            checkpoint.phase = ModelPhase::RetryScheduled;
            checkpoint.revision += 1;
            true
        }
        ModelAction::RetryDue if checkpoint.phase == ModelPhase::RetryScheduled => {
            checkpoint.phase = ModelPhase::Leased;
            checkpoint.revision += 1;
            checkpoint.lease_epoch += 1;
            true
        }
        ModelAction::OperatorRetry if checkpoint.phase == ModelPhase::Blocked => {
            checkpoint.phase = ModelPhase::RetryScheduled;
            checkpoint.revision += 1;
            true
        }
        ModelAction::RetentionGap if checkpoint.phase == ModelPhase::Ready => {
            checkpoint.phase = ModelPhase::Blocked;
            checkpoint.revision += 1;
            true
        }
        ModelAction::Disable if checkpoint.phase != ModelPhase::Disabled => {
            checkpoint.phase = ModelPhase::Disabled;
            checkpoint.revision += 1;
            true
        }
        _ => false,
    };

    (accepted, checkpoint)
}

#[test]
fn stage_a_consumer_transition_fixture_matches_reference_model() {
    let contract: TransitionContract = serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_consumer_transitions.v1.json"
    ))
    .expect("parse Runtime Event Stream transition fixture");

    assert_eq!(
        contract.schema_version,
        RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION
    );
    assert_eq!(contract.cases.len(), 14);

    let mut ids = BTreeSet::new();
    for case in contract.cases {
        assert!(
            ids.insert(case.id.clone()),
            "duplicate case id: {}",
            case.id
        );
        let (accepted, actual) = apply_reference_transition(case.start, &case.action);
        assert_eq!(accepted, case.expected.accepted, "{} accepted", case.id);
        assert_eq!(actual.phase, case.expected.phase, "{} phase", case.id);
        assert_eq!(
            actual.next_position, case.expected.next_position,
            "{} next_position",
            case.id
        );
        assert_eq!(
            actual.revision, case.expected.revision,
            "{} revision",
            case.id
        );
        assert_eq!(
            actual.lease_epoch, case.expected.lease_epoch,
            "{} lease_epoch",
            case.id
        );
    }
}

#[test]
fn stage_a2_review_fixture_freezes_non_runtime_boundaries() {
    let contract: StageA2ReviewContract = serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_review.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2 review fixture");

    assert_eq!(
        contract.schema_version,
        RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION
    );
    assert_eq!(contract.stage, "a2_1_design_only");
    assert!(!contract.production_runtime_enabled);

    let mut surface_ids = BTreeSet::new();
    for surface in &contract.observed_surfaces {
        assert!(surface_ids.insert(surface.id.as_str()), "{}", surface.id);
        assert!(!surface.creates_turn_pipeline, "{}", surface.id);
        assert!(
            !surface.may_directly_commit_authoritative_state,
            "{}",
            surface.id
        );
        assert!(!surface.has_durable_output_receipt, "{}", surface.id);
        assert_evidence_tokens(
            surface.id.as_str(),
            surface.evidence_path.as_str(),
            &surface.required_tokens,
        );
    }
    assert_eq!(surface_ids.len(), 5);

    let mut evidence_ids = BTreeSet::new();
    for evidence in &contract.review_evidence {
        assert!(evidence_ids.insert(evidence.id.as_str()), "{}", evidence.id);
        assert_evidence_tokens(
            evidence.id.as_str(),
            evidence.evidence_path.as_str(),
            &evidence.required_tokens,
        );
    }
    assert_eq!(evidence_ids.len(), 3);

    let storage = &contract.storage_decision;
    assert_eq!(storage.first_backend, "sqlite");
    assert!(storage.all_paths_must_be_distinct);
    assert!(!storage.cross_database_atomicity_claimed);
    assert!(storage.main_database_owns_producer_outbox);
    assert_eq!(
        BTreeSet::from([
            storage.main_state_database.as_str(),
            storage.production_stream_database.as_str(),
            storage.trace_database.as_str(),
        ])
        .len(),
        3
    );
    for required_owner in [
        "event_records",
        "consumer_checkpoints",
        "retention_tombstones",
        "erasure_work_items",
    ] {
        assert!(
            storage
                .stream_database_owns
                .iter()
                .any(|owner| owner == required_owner),
            "missing Stream storage owner {required_owner}"
        );
    }
    assert_eq!(storage.external_broker_triggers.len(), 3);

    let publication_cases = contract
        .publication_cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(publication_cases.len(), 4);
    let state_commit = publication_cases["authoritative_state_commit"];
    assert_eq!(state_commit.transaction_domain, "app.db");
    assert_eq!(
        state_commit.atomic_writes,
        ["authoritative_state", "producer_outbox"]
    );
    assert_eq!(
        state_commit.publication_gate,
        "state_and_outbox_commit_together"
    );
    assert!(state_commit
        .failure_rule
        .contains("retry_outbox_after_commit"));

    let ring = publication_cases["ring_dispatch_snapshot"];
    assert!(ring
        .atomic_writes
        .iter()
        .any(|item| item == "producer_outbox"));
    assert!(ring.publication_gate.contains("complete_dispatch"));
    assert!(ring.failure_rule.contains("release_authorization_early"));

    let consumer = publication_cases["consumer_side_effect"];
    assert!(consumer
        .atomic_writes
        .iter()
        .any(|item| item == "consumer_inbox"));
    assert!(consumer.publication_gate.contains("already_applied"));
    assert!(consumer
        .failure_rule
        .contains("without_repeating_side_effect"));

    let output = publication_cases["external_output_delivery"];
    assert!(output.publication_gate.contains("adapter_ack"));
    assert_eq!(
        output.failure_rule,
        "generated_response_is_not_a_delivery_receipt"
    );

    let idempotency = &contract.idempotency_contract;
    assert_eq!(
        idempotency.scope,
        ["session_partition", "source", "source_idempotency_key"]
    );
    assert_eq!(idempotency.maximum_key_bytes, 256);
    assert!(idempotency.empty_forbidden);
    assert!(idempotency.control_characters_forbidden);
    assert!(!idempotency.trim_or_case_fold);
    assert_eq!(idempotency.conflicting_content, "reject");
    assert!(idempotency.credential_material_forbidden);

    let retention = &contract.retention_contract;
    assert_eq!(retention.normal_expiry_entry, "position_tombstone");
    assert!(retention.normal_expiry_may_advance_checkpoint);
    assert_eq!(retention.unexpected_physical_gap, "blocked");
    assert!(retention.required_event_classes_wait_for_required_consumers);
    assert!(retention.session_erasure_hard_deletes_partition);
    for forbidden in ["event_id", "source", "correlation_id", "payload"] {
        assert!(
            !retention
                .normal_expiry_tombstone_fields
                .iter()
                .any(|field| field == forbidden),
            "normal-expiry tombstone leaks {forbidden}"
        );
    }

    let erasure = &contract.session_erasure_contract;
    assert_eq!(
        erasure.states.first().map(String::as_str),
        Some("requested")
    );
    assert_eq!(erasure.states.last().map(String::as_str), Some("blocked"));
    assert_eq!(
        erasure.required_order.first().map(String::as_str),
        Some("block_new_ingress_and_leases")
    );
    assert_eq!(
        erasure.required_order.last().map(String::as_str),
        Some("write_minimal_completion_receipt")
    );
    assert!(erasure.completion_requires_all_required_domains);
    assert!(erasure.retry_is_idempotent);
    assert!(erasure
        .required_domains
        .iter()
        .any(|domain| domain == "required_consumer_stores"));
    for forbidden in [
        "runtime_session_id",
        "session_partition",
        "access_subject_id",
        "role_id",
        "event_id",
        "payload",
    ] {
        assert!(
            erasure
                .completion_receipt_forbidden_fields
                .iter()
                .any(|field| field == forbidden),
            "completion receipt must forbid {forbidden}"
        );
    }

    for gap in [
        "no_real_qq_live_or_hardware_output_adapter",
        "no_multi_host_session_lease_evidence",
        "no_production_stream_schema_or_migration",
        "no_erasure_or_retention_executor",
        "no_consumer_runtime_or_read_api",
    ] {
        assert!(contract.known_gaps.iter().any(|item| item == gap));
    }
}
