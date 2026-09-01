use std::collections::BTreeSet;

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
