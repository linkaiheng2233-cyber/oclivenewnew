mod contract;
mod scenarios;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context};
use chrono::{SecondsFormat, Utc};
use oclive_kernel_types::RuntimeEventTraceDiagnostics;
use serde::Serialize;
use sha2::{Digest, Sha256};

use contract::{parse_and_validate, FaultAction};

const EVIDENCE_FILE_NAME: &str = "runtime-event-trace-shadow.fault-evidence.json";
const SUMMARY_FILE_NAME: &str = "runtime-event-trace-shadow.fault-summary.md";

#[derive(Serialize)]
struct FaultEvidence {
    schema_version: u16,
    evidence_kind: String,
    authoritative_runtime_stream: bool,
    behavior_driver: bool,
    synthetic_only: bool,
    exports_raw_database: bool,
    generated_at: String,
    source_commit: String,
    source_worktree_dirty: bool,
    scenario_contract: String,
    scenario_contract_sha256: String,
    summary: FaultSummary,
    limitations: Vec<String>,
    scenarios: Vec<FaultScenarioEvidence>,
}

#[derive(Serialize)]
struct FaultSummary {
    scenarios: usize,
    passed: usize,
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    enqueued_dispatches: u64,
    persisted_dispatches: u64,
    dropped_dispatches: u64,
    observed_failures: u64,
}

#[derive(Serialize)]
struct FaultScenarioEvidence {
    id: String,
    action: FaultAction,
    outcome: &'static str,
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    diagnostics: RuntimeEventTraceDiagnostics,
    invariants: BTreeMap<String, bool>,
}

pub(crate) async fn run(
    contract_text: &str,
    scenario_contract: String,
    output_dir: &Path,
    source_commit: String,
    source_worktree_dirty: bool,
) -> anyhow::Result<()> {
    let contract = parse_and_validate(contract_text)?;
    let mut scenario_evidence = Vec::with_capacity(contract.scenarios.len());
    for scenario in &contract.scenarios {
        let observed = scenarios::execute(scenario).await?;
        if !observed.diagnostics.configured
            || !observed.diagnostics.worker_active
            || observed.diagnostics.last_error_kind != Some(scenario.expected_error_kind)
        {
            bail!(
                "{} diagnostics drifted from its fault contract",
                scenario.id
            );
        }
        if observed.invariants.len() != scenario.expected_invariants.len() {
            bail!("{} produced an unexpected invariant set", scenario.id);
        }
        for invariant in &scenario.expected_invariants {
            if observed.invariants.get(invariant) != Some(&true) {
                bail!("{} failed invariant {invariant}", scenario.id);
            }
        }
        scenario_evidence.push(FaultScenarioEvidence {
            id: scenario.id.clone(),
            action: scenario.action,
            outcome: "pass",
            attempted_dispatches: scenario.attempted_dispatches,
            successful_ring_dispatches: observed.successful_ring_dispatches,
            diagnostics: observed.diagnostics,
            invariants: observed.invariants,
        });
    }

    let summary = FaultSummary {
        scenarios: scenario_evidence.len(),
        passed: scenario_evidence.len(),
        attempted_dispatches: scenario_evidence
            .iter()
            .map(|scenario| scenario.attempted_dispatches)
            .sum(),
        successful_ring_dispatches: scenario_evidence
            .iter()
            .map(|scenario| scenario.successful_ring_dispatches)
            .sum(),
        enqueued_dispatches: scenario_evidence
            .iter()
            .map(|scenario| scenario.diagnostics.enqueued_dispatches)
            .sum(),
        persisted_dispatches: scenario_evidence
            .iter()
            .map(|scenario| scenario.diagnostics.persisted_dispatches)
            .sum(),
        dropped_dispatches: scenario_evidence
            .iter()
            .map(|scenario| scenario.diagnostics.dropped_dispatches)
            .sum(),
        observed_failures: scenario_evidence
            .iter()
            .map(|scenario| scenario.diagnostics.failure_count)
            .sum(),
    };
    let evidence = FaultEvidence {
        schema_version: 1,
        evidence_kind: contract.evidence_kind,
        authoritative_runtime_stream: false,
        behavior_driver: false,
        synthetic_only: true,
        exports_raw_database: false,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_commit,
        source_worktree_dirty,
        scenario_contract,
        scenario_contract_sha256: format!(
            "{:x}",
            Sha256::digest(contract_text.as_bytes())
        ),
        summary,
        limitations: vec![
            "Synthetic temporary databases only; no user, role-memory, Prompt, or model content is collected."
                .into(),
            "Faults are induced with a temporary SQLite writer lock or table removal and never touch the kernel database."
                .into(),
            "This evidence validates fail-open accounting, not Production Stream delivery or recovery semantics."
                .into(),
            "Duplicate delivery, concurrency, and long-run soak remain outside this S1.1 slice."
                .into(),
        ],
        scenarios: scenario_evidence,
    };

    std::fs::create_dir_all(output_dir).context("create trace fault evidence directory")?;
    let mut evidence_json =
        serde_json::to_string_pretty(&evidence).context("serialize trace fault evidence")?;
    evidence_json.push('\n');
    std::fs::write(output_dir.join(EVIDENCE_FILE_NAME), evidence_json)
        .context("write trace fault evidence")?;
    std::fs::write(
        output_dir.join(SUMMARY_FILE_NAME),
        render_markdown_summary(&evidence),
    )
    .context("write trace fault summary")?;

    println!(
        "fault-sampler: PASS ({} scenarios; {} successful Ring dispatches; synthetic-only)",
        evidence.summary.scenarios, evidence.summary.successful_ring_dispatches
    );
    Ok(())
}

fn render_markdown_summary(evidence: &FaultEvidence) -> String {
    let mut output = format!(
        "# Runtime Event Trace Shadow Synthetic Fault Samples\n\n\
         - Source commit: `{}`\n\
         - Source worktree dirty: `{}`\n\
         - Scope: synthetic-only, behavior-neutral fail-open evidence\n\
         - Authority: not a Runtime Event Stream, behavior input, or raw database export\n\n\
         ## Scenarios\n\n\
         | Scenario | Action | Ring success | Enqueued | Persisted | Dropped | Last error |\n\
         | --- | --- | ---: | ---: | ---: | ---: | --- |\n",
        evidence.source_commit, evidence.source_worktree_dirty
    );
    for scenario in &evidence.scenarios {
        output.push_str(&format!(
            "| `{}` | `{}` | {}/{} | {} | {} | {} | `{:?}` |\n",
            scenario.id,
            scenario.action.as_str(),
            scenario.successful_ring_dispatches,
            scenario.attempted_dispatches,
            scenario.diagnostics.enqueued_dispatches,
            scenario.diagnostics.persisted_dispatches,
            scenario.diagnostics.dropped_dispatches,
            scenario.diagnostics.last_error_kind
        ));
    }
    output.push_str("\n## Verified invariants\n\n");
    for scenario in &evidence.scenarios {
        let names = scenario
            .invariants
            .keys()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("- **{}**: {names}\n", scenario.id));
    }
    output.push_str("\n## Limitations\n\n");
    for limitation in &evidence.limitations {
        output.push_str(&format!("- {limitation}\n"));
    }
    output
}
