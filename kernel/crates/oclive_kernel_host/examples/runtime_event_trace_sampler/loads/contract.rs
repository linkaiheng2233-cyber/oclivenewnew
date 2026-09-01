use std::collections::BTreeSet;

use anyhow::bail;
use serde::{Deserialize, Serialize};

const EVIDENCE_KIND: &str = "runtime_event_trace_shadow_synthetic_load_samples";
const EXPECTED_SCENARIOS: usize = 2;
const MAX_TOTAL_DISPATCHES: u64 = 4_096;
const MAX_EXECUTION_MS: u64 = 120_000;

#[derive(Debug, Deserialize)]
pub(super) struct LoadContract {
    pub(super) schema_version: u16,
    pub(super) evidence_kind: String,
    pub(super) authoritative_runtime_stream: bool,
    pub(super) behavior_driver: bool,
    pub(super) synthetic_only: bool,
    pub(super) exports_raw_database: bool,
    pub(super) bounded_execution: bool,
    pub(super) scenarios: Vec<LoadScenarioContract>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LoadScenarioContract {
    pub(super) id: String,
    pub(super) action: LoadAction,
    pub(super) workers: u16,
    pub(super) rounds: u16,
    pub(super) dispatches_per_worker: u16,
    pub(super) pause_between_rounds_ms: u64,
    pub(super) minimum_elapsed_ms: u64,
    pub(super) maximum_elapsed_ms: u64,
    pub(super) expected_invariants: Vec<String>,
}

impl LoadScenarioContract {
    pub(super) fn attempted_dispatches(&self) -> anyhow::Result<u64> {
        u64::from(self.workers)
            .checked_mul(u64::from(self.rounds))
            .and_then(|count| count.checked_mul(u64::from(self.dispatches_per_worker)))
            .ok_or_else(|| anyhow::anyhow!("{} dispatch count overflowed", self.id))
    }

    pub(super) fn expected_worker_runs(&self) -> u64 {
        u64::from(self.workers).saturating_mul(u64::from(self.rounds))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LoadAction {
    ConcurrentBurst,
    BoundedShortSoak,
}

impl LoadAction {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ConcurrentBurst => "concurrent_burst",
            Self::BoundedShortSoak => "bounded_short_soak",
        }
    }
}

pub(super) fn parse_and_validate(text: &str) -> anyhow::Result<LoadContract> {
    let contract: LoadContract =
        serde_json::from_str(text).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    validate(&contract)?;
    Ok(contract)
}

fn validate(contract: &LoadContract) -> anyhow::Result<()> {
    if contract.schema_version != 1
        || contract.evidence_kind != EVIDENCE_KIND
        || contract.authoritative_runtime_stream
        || contract.behavior_driver
        || !contract.synthetic_only
        || contract.exports_raw_database
        || !contract.bounded_execution
        || contract.scenarios.len() != EXPECTED_SCENARIOS
    {
        bail!("invalid runtime event trace load sample contract header");
    }

    let mut ids = BTreeSet::new();
    let mut actions = BTreeSet::new();
    for scenario in &contract.scenarios {
        let attempted_dispatches = scenario.attempted_dispatches()?;
        if !ids.insert(scenario.id.as_str())
            || !actions.insert(scenario.action.as_str())
            || scenario.workers == 0
            || scenario.workers > 64
            || scenario.rounds == 0
            || scenario.rounds > 128
            || scenario.dispatches_per_worker == 0
            || scenario.dispatches_per_worker > 256
            || attempted_dispatches > MAX_TOTAL_DISPATCHES
            || scenario.minimum_elapsed_ms > scenario.maximum_elapsed_ms
            || scenario.maximum_elapsed_ms == 0
            || scenario.maximum_elapsed_ms > MAX_EXECUTION_MS
            || scenario.pause_between_rounds_ms > 1_000
        {
            bail!("invalid trace load scenario {}", scenario.id);
        }
        match scenario.action {
            LoadAction::ConcurrentBurst
                if scenario.rounds != 1
                    || scenario.pause_between_rounds_ms != 0
                    || scenario.minimum_elapsed_ms != 0 =>
            {
                bail!("{} is not a bounded concurrent burst", scenario.id);
            }
            LoadAction::BoundedShortSoak
                if scenario.rounds < 2
                    || scenario.pause_between_rounds_ms == 0
                    || scenario.minimum_elapsed_ms == 0 =>
            {
                bail!("{} is not a bounded short soak", scenario.id);
            }
            _ => {}
        }

        let expected_invariants = invariant_names().iter().copied().collect::<BTreeSet<_>>();
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
    if actions.len() != EXPECTED_SCENARIOS {
        bail!("load sample contract must contain each supported action once");
    }
    Ok(())
}

pub(super) const fn invariant_names() -> &'static [&'static str] {
    &[
        "all_rounds_completed",
        "all_workers_completed",
        "all_ring_dispatches_succeeded",
        "dispatch_accounting_balanced",
        "all_enqueued_dispatches_drained",
        "no_duplicate_events",
        "only_expected_trace_errors",
        "trace_worker_remained_active",
        "bounded_ring_history_preserved",
        "main_database_healthy",
        "privacy_flags_false",
        "execution_time_within_contract",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_load_contract_is_synthetic_bounded_and_behavior_neutral() {
        let text = include_str!(
            "../../../tests/fixtures/runtime_event_trace_shadow_load_scenarios.v1.json"
        );
        let contract = parse_and_validate(text).expect("valid bundled load sample contract");
        assert_eq!(contract.scenarios.len(), EXPECTED_SCENARIOS);
        assert!(contract.synthetic_only);
        assert!(contract.bounded_execution);
        assert!(!contract.authoritative_runtime_stream);
        assert!(!contract.behavior_driver);
        assert!(!contract.exports_raw_database);
        assert_eq!(
            contract
                .scenarios
                .iter()
                .map(LoadScenarioContract::attempted_dispatches)
                .collect::<anyhow::Result<Vec<_>>>()
                .expect("valid dispatch counts")
                .into_iter()
                .sum::<u64>(),
            1_280
        );
    }
}
