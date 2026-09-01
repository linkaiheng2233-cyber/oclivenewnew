use std::collections::BTreeSet;

use anyhow::bail;
use oclive_kernel_types::RuntimeEventTraceErrorKind;
use serde::{Deserialize, Serialize};

const EVIDENCE_KIND: &str = "runtime_event_trace_shadow_synthetic_fault_samples";

#[derive(Debug, Deserialize)]
pub(super) struct FaultContract {
    pub(super) schema_version: u16,
    pub(super) evidence_kind: String,
    pub(super) authoritative_runtime_stream: bool,
    pub(super) behavior_driver: bool,
    pub(super) synthetic_only: bool,
    pub(super) exports_raw_database: bool,
    pub(super) scenarios: Vec<FaultScenarioContract>,
}

#[derive(Debug, Deserialize)]
pub(super) struct FaultScenarioContract {
    pub(super) id: String,
    pub(super) action: FaultAction,
    pub(super) attempted_dispatches: u64,
    pub(super) expected_error_kind: RuntimeEventTraceErrorKind,
    pub(super) expected_invariants: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum FaultAction {
    QueueSaturation,
    PostStartWriteFailure,
}

impl FaultAction {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::QueueSaturation => "queue_saturation",
            Self::PostStartWriteFailure => "post_start_write_failure",
        }
    }
}

pub(super) fn parse_and_validate(text: &str) -> anyhow::Result<FaultContract> {
    let contract: FaultContract =
        serde_json::from_str(text).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    validate(&contract)?;
    Ok(contract)
}

fn validate(contract: &FaultContract) -> anyhow::Result<()> {
    if contract.schema_version != 1
        || contract.evidence_kind != EVIDENCE_KIND
        || contract.authoritative_runtime_stream
        || contract.behavior_driver
        || !contract.synthetic_only
        || contract.exports_raw_database
        || contract.scenarios.len() != 2
    {
        bail!("invalid runtime event trace fault sample contract header");
    }

    let mut ids = BTreeSet::new();
    let mut actions = BTreeSet::new();
    for scenario in &contract.scenarios {
        if !ids.insert(scenario.id.as_str())
            || !actions.insert(scenario.action.as_str())
            || scenario.attempted_dispatches == 0
            || scenario.attempted_dispatches > 4_096
        {
            bail!("invalid trace fault scenario {}", scenario.id);
        }
        let expected_error = match scenario.action {
            FaultAction::QueueSaturation => RuntimeEventTraceErrorKind::QueueFull,
            FaultAction::PostStartWriteFailure => RuntimeEventTraceErrorKind::WriteFailed,
        };
        if scenario.expected_error_kind != expected_error {
            bail!("{} has the wrong expected error kind", scenario.id);
        }
        let expected_invariants = invariant_names(scenario.action)
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let declared_invariants = scenario
            .expected_invariants
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if declared_invariants != expected_invariants
            || declared_invariants.len() != scenario.expected_invariants.len()
        {
            bail!("{} has an invalid invariant set", scenario.id);
        }
    }
    if actions.len() != 2 {
        bail!("fault sample contract must contain each supported action once");
    }
    Ok(())
}

pub(super) fn invariant_names(action: FaultAction) -> &'static [&'static str] {
    const COMMON: [&str; 4] = [
        "all_ring_dispatches_succeeded",
        "dispatch_accounting_balanced",
        "main_database_healthy",
        "privacy_flags_false",
    ];
    const QUEUE: [&str; 6] = [
        COMMON[0],
        COMMON[1],
        "all_enqueued_dispatches_drained",
        "dropped_dispatches_observed",
        COMMON[2],
        COMMON[3],
    ];
    const WRITE: [&str; 6] = [
        COMMON[0],
        COMMON[1],
        "write_failure_observed",
        "no_dispatch_dropped",
        COMMON[2],
        COMMON[3],
    ];
    match action {
        FaultAction::QueueSaturation => &QUEUE,
        FaultAction::PostStartWriteFailure => &WRITE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fault_contract_is_synthetic_and_behavior_neutral() {
        let text = include_str!(
            "../../../tests/fixtures/runtime_event_trace_shadow_fault_scenarios.v1.json"
        );
        let contract = parse_and_validate(text).expect("valid bundled fault sample contract");
        assert_eq!(contract.scenarios.len(), 2);
        assert!(contract.synthetic_only);
        assert!(!contract.authoritative_runtime_stream);
        assert!(!contract.behavior_driver);
        assert!(!contract.exports_raw_database);
    }
}
