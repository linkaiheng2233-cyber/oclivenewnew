use std::collections::{BTreeMap, BTreeSet};

use anyhow::bail;
use serde::Deserialize;

pub(crate) const BEFORE_RESTART: &str = "before_restart";
pub(crate) const AFTER_RESTART: &str = "after_restart";

#[derive(Debug, Deserialize)]
pub(crate) struct SampleContract {
    pub(crate) schema_version: u16,
    pub(crate) evidence_kind: String,
    pub(crate) authoritative_runtime_stream: bool,
    pub(crate) behavior_driver: bool,
    pub(crate) synthetic_only: bool,
    pub(crate) scenarios: Vec<ScenarioContract>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ScenarioContract {
    pub(crate) id: String,
    pub(crate) phase: String,
    pub(crate) action: String,
    pub(crate) expected_events: Vec<ExpectedEvent>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ExpectedEvent {
    pub(crate) kind: String,
    pub(crate) source: String,
    pub(crate) source_weight_bps: u16,
    pub(crate) causation_index: Option<usize>,
    pub(crate) depth: u16,
}

pub(crate) fn parse_and_validate(text: &str) -> anyhow::Result<SampleContract> {
    let contract: SampleContract =
        serde_json::from_str(text).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    validate(&contract)?;
    Ok(contract)
}

fn validate(contract: &SampleContract) -> anyhow::Result<()> {
    if contract.schema_version != 1
        || contract.evidence_kind != "runtime_event_trace_shadow_synthetic_samples"
        || contract.authoritative_runtime_stream
        || contract.behavior_driver
        || !contract.synthetic_only
        || contract.scenarios.is_empty()
    {
        bail!("invalid runtime event trace sample contract header");
    }
    let mut ids = BTreeSet::new();
    let mut phases = BTreeMap::<&str, usize>::new();
    for scenario in &contract.scenarios {
        if !ids.insert(scenario.id.as_str())
            || ![BEFORE_RESTART, AFTER_RESTART].contains(&scenario.phase.as_str())
            || scenario.expected_events.is_empty()
        {
            bail!(
                "invalid runtime event trace sample scenario {}",
                scenario.id
            );
        }
        *phases.entry(scenario.phase.as_str()).or_default() += 1;
        for (index, event) in scenario.expected_events.iter().enumerate() {
            if event.causation_index.is_some_and(|cause| cause >= index) {
                bail!("{} has forward or self causation", scenario.id);
            }
        }
    }
    if !phases.contains_key(BEFORE_RESTART) || !phases.contains_key(AFTER_RESTART) {
        bail!("sample contract must cover both sides of a restart");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_contract_is_synthetic_and_restart_bounded() {
        let text =
            include_str!("../../tests/fixtures/runtime_event_trace_shadow_scenarios.v1.json");
        let contract = parse_and_validate(text).expect("valid bundled sample contract");
        assert_eq!(contract.scenarios.len(), 5);
        assert!(contract.synthetic_only);
        assert!(!contract.authoritative_runtime_stream);
        assert!(!contract.behavior_driver);
        assert!(contract
            .scenarios
            .iter()
            .any(|scenario| scenario.phase == BEFORE_RESTART));
        assert!(contract
            .scenarios
            .iter()
            .any(|scenario| scenario.phase == AFTER_RESTART));
    }
}
