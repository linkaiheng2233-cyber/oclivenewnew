use std::collections::BTreeSet;

use anyhow::bail;
use serde::{Deserialize, Serialize};

const EVIDENCE_KIND: &str = "runtime_event_trace_shadow_synthetic_sustained_soak";
const FIXED_DURATION_MS: u64 = 600_000;
const MAXIMUM_EXECUTION_MS: u64 = 720_000;
const MAXIMUM_TOTAL_DISPATCHES: u64 = 10_000;

#[derive(Debug, Deserialize)]
pub(super) struct SoakContract {
    pub(super) schema_version: u16,
    pub(super) evidence_kind: String,
    pub(super) authoritative_runtime_stream: bool,
    pub(super) behavior_driver: bool,
    pub(super) synthetic_only: bool,
    pub(super) exports_raw_database: bool,
    pub(super) bounded_execution: bool,
    pub(super) routine_ci: bool,
    pub(super) production_duration_claim: bool,
    pub(super) scenarios: Vec<SoakScenarioContract>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SoakScenarioContract {
    pub(super) id: String,
    pub(super) action: SoakAction,
    pub(super) duration_ms: u64,
    pub(super) maximum_elapsed_ms: u64,
    pub(super) interval_ms: u64,
    pub(super) workers: u16,
    pub(super) dispatches_per_worker_per_interval: u16,
    pub(super) health_check_interval_rounds: u64,
    pub(super) maximum_rss_growth_mib: u64,
    pub(super) expected_invariants: Vec<String>,
}

impl SoakScenarioContract {
    pub(super) fn rounds(&self) -> anyhow::Result<u64> {
        self.duration_ms
            .checked_div(self.interval_ms)
            .ok_or_else(|| anyhow::anyhow!("{} has a zero interval", self.id))
    }

    pub(super) fn attempted_dispatches(&self) -> anyhow::Result<u64> {
        self.rounds()?
            .checked_mul(u64::from(self.workers))
            .and_then(|count| count.checked_mul(u64::from(self.dispatches_per_worker_per_interval)))
            .ok_or_else(|| anyhow::anyhow!("{} dispatch count overflowed", self.id))
    }

    pub(super) fn expected_worker_runs(&self) -> anyhow::Result<u64> {
        self.rounds()?
            .checked_mul(u64::from(self.workers))
            .ok_or_else(|| anyhow::anyhow!("{} worker run count overflowed", self.id))
    }

    pub(super) fn expected_health_checks(&self) -> anyhow::Result<u64> {
        self.rounds()?
            .checked_div(self.health_check_interval_rounds)
            .ok_or_else(|| anyhow::anyhow!("{} has a zero health-check interval", self.id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SoakAction {
    FixedSustainedSoak,
}

pub(super) fn parse_and_validate(text: &str) -> anyhow::Result<SoakContract> {
    let contract: SoakContract =
        serde_json::from_str(text).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    validate(&contract)?;
    Ok(contract)
}

fn validate(contract: &SoakContract) -> anyhow::Result<()> {
    if contract.schema_version != 1
        || contract.evidence_kind != EVIDENCE_KIND
        || contract.authoritative_runtime_stream
        || contract.behavior_driver
        || !contract.synthetic_only
        || contract.exports_raw_database
        || !contract.bounded_execution
        || contract.routine_ci
        || contract.production_duration_claim
        || contract.scenarios.len() != 1
    {
        bail!("invalid runtime event trace sustained-soak contract header");
    }

    let scenario = &contract.scenarios[0];
    let rounds = scenario.rounds()?;
    let attempted_dispatches = scenario.attempted_dispatches()?;
    if scenario.id.is_empty()
        || scenario.action != SoakAction::FixedSustainedSoak
        || scenario.duration_ms != FIXED_DURATION_MS
        || scenario.maximum_elapsed_ms < scenario.duration_ms
        || scenario.maximum_elapsed_ms > MAXIMUM_EXECUTION_MS
        || scenario.interval_ms < 250
        || scenario.interval_ms > 5_000
        || !scenario.duration_ms.is_multiple_of(scenario.interval_ms)
        || scenario.workers == 0
        || scenario.workers > 8
        || scenario.dispatches_per_worker_per_interval == 0
        || scenario.dispatches_per_worker_per_interval > 8
        || attempted_dispatches > MAXIMUM_TOTAL_DISPATCHES
        || scenario.health_check_interval_rounds == 0
        || scenario.health_check_interval_rounds > rounds
        || !rounds.is_multiple_of(scenario.health_check_interval_rounds)
        || !(32..=512).contains(&scenario.maximum_rss_growth_mib)
    {
        bail!("invalid trace sustained-soak scenario {}", scenario.id);
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
    Ok(())
}

pub(super) const fn invariant_names() -> &'static [&'static str] {
    &[
        "all_rounds_completed",
        "all_workers_completed",
        "all_ring_dispatches_succeeded",
        "dispatch_accounting_balanced",
        "all_enqueued_dispatches_drained",
        "no_trace_drop_duplicate_or_failure",
        "trace_worker_remained_active",
        "bounded_ring_history_preserved",
        "periodic_main_database_health",
        "main_database_healthy",
        "privacy_flags_false",
        "rss_growth_within_contract",
        "execution_time_within_contract",
        "shutdown_persisted_all_rows",
        "restart_position_continuity",
        "post_restart_append_succeeded",
        "post_restart_main_database_healthy",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_soak_contract_is_fixed_private_and_behavior_neutral() {
        let text = include_str!(
            "../../../tests/fixtures/runtime_event_trace_shadow_soak_scenarios.v1.json"
        );
        let contract = parse_and_validate(text).expect("valid bundled sustained-soak contract");
        let scenario = &contract.scenarios[0];
        assert_eq!(scenario.duration_ms, FIXED_DURATION_MS);
        assert_eq!(scenario.rounds().expect("valid rounds"), 600);
        assert_eq!(
            scenario
                .attempted_dispatches()
                .expect("valid dispatch count"),
            2_400
        );
        assert_eq!(
            scenario
                .expected_health_checks()
                .expect("valid health-check count"),
            20
        );
        assert!(!contract.authoritative_runtime_stream);
        assert!(!contract.behavior_driver);
        assert!(!contract.exports_raw_database);
        assert!(!contract.routine_ci);
        assert!(!contract.production_duration_claim);
    }
}
