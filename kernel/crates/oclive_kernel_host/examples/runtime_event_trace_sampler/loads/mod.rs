mod contract;
mod scenarios;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context};
use chrono::{SecondsFormat, Utc};
use oclive_kernel_types::{RuntimeEventTraceDiagnostics, RuntimeEventTraceErrorKind};
use serde::Serialize;
use sha2::{Digest, Sha256};

use contract::{parse_and_validate, LoadAction};

const EVIDENCE_FILE_NAME: &str = "runtime-event-trace-shadow.load-evidence.json";
const SUMMARY_FILE_NAME: &str = "runtime-event-trace-shadow.load-summary.md";

#[derive(Serialize)]
struct LoadEvidence {
    schema_version: u16,
    evidence_kind: String,
    authoritative_runtime_stream: bool,
    behavior_driver: bool,
    synthetic_only: bool,
    exports_raw_database: bool,
    bounded_execution: bool,
    generated_at: String,
    source_commit: String,
    source_worktree_dirty: bool,
    scenario_contract: String,
    scenario_contract_sha256: String,
    summary: LoadSummary,
    limitations: Vec<String>,
    scenarios: Vec<LoadScenarioEvidence>,
}

#[derive(Serialize)]
struct LoadSummary {
    scenarios: usize,
    passed: usize,
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    enqueued_dispatches: u64,
    persisted_dispatches: u64,
    dropped_dispatches: u64,
    observed_failures: u64,
    elapsed_ms: u64,
}

#[derive(Serialize)]
struct LoadScenarioEvidence {
    id: String,
    action: LoadAction,
    outcome: &'static str,
    workers: u16,
    rounds: u16,
    dispatches_per_worker: u16,
    pause_between_rounds_ms: u64,
    minimum_elapsed_ms: u64,
    maximum_elapsed_ms: u64,
    attempted_dispatches: u64,
    completed_rounds: u64,
    completed_worker_runs: u64,
    successful_ring_dispatches: u64,
    elapsed_ms: u64,
    diagnostics: RuntimeEventTraceDiagnostics,
    ring: LoadRingEvidence,
    invariants: BTreeMap<String, bool>,
}

#[derive(Serialize)]
struct LoadRingEvidence {
    history_len: u64,
    history_capacity: u64,
    last_allocated_sequence: u64,
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
            || !observed.diagnostics.accepting_dispatches
            || !observed.diagnostics.worker_active
            || !matches!(
                observed.diagnostics.last_error_kind,
                None | Some(RuntimeEventTraceErrorKind::QueueFull)
            )
        {
            bail!("{} diagnostics drifted from its load contract", scenario.id);
        }
        if observed.invariants.len() != scenario.expected_invariants.len() {
            bail!("{} produced an unexpected invariant set", scenario.id);
        }
        for invariant in &scenario.expected_invariants {
            if observed.invariants.get(invariant) != Some(&true) {
                bail!("{} failed invariant {invariant}", scenario.id);
            }
        }
        scenario_evidence.push(LoadScenarioEvidence {
            id: scenario.id.clone(),
            action: scenario.action,
            outcome: "pass",
            workers: scenario.workers,
            rounds: scenario.rounds,
            dispatches_per_worker: scenario.dispatches_per_worker,
            pause_between_rounds_ms: scenario.pause_between_rounds_ms,
            minimum_elapsed_ms: scenario.minimum_elapsed_ms,
            maximum_elapsed_ms: scenario.maximum_elapsed_ms,
            attempted_dispatches: observed.attempted_dispatches,
            completed_rounds: observed.completed_rounds,
            completed_worker_runs: observed.completed_worker_runs,
            successful_ring_dispatches: observed.successful_ring_dispatches,
            elapsed_ms: observed.elapsed_ms,
            diagnostics: observed.diagnostics,
            ring: LoadRingEvidence {
                history_len: observed.ring_history_len,
                history_capacity: observed.ring_history_capacity,
                last_allocated_sequence: observed.ring_last_allocated_sequence,
            },
            invariants: observed.invariants,
        });
    }

    let summary = LoadSummary {
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
        elapsed_ms: scenario_evidence
            .iter()
            .map(|scenario| scenario.elapsed_ms)
            .sum(),
    };
    let evidence = LoadEvidence {
        schema_version: 1,
        evidence_kind: contract.evidence_kind,
        authoritative_runtime_stream: false,
        behavior_driver: false,
        synthetic_only: true,
        exports_raw_database: false,
        bounded_execution: true,
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
            "Elapsed time and the exact enqueue/drop split are machine-local evidence, not a performance SLA or product contract."
                .into(),
            "The bounded short soak is a developer probe and does not replace production-duration or hardware soak."
                .into(),
            "This evidence validates concurrent fail-open accounting only, not Stream ordering, at-least-once delivery, consumers, checkpoints, or recovery."
                .into(),
            "Duplicate recorder input and sustained runtime use separate S1 contracts; no replay or source-binding bypass entry is introduced."
                .into(),
        ],
        scenarios: scenario_evidence,
    };

    std::fs::create_dir_all(output_dir).context("create trace load evidence directory")?;
    let mut evidence_json =
        serde_json::to_string_pretty(&evidence).context("serialize trace load evidence")?;
    evidence_json.push('\n');
    std::fs::write(output_dir.join(EVIDENCE_FILE_NAME), evidence_json)
        .context("write trace load evidence")?;
    std::fs::write(
        output_dir.join(SUMMARY_FILE_NAME),
        render_markdown_summary(&evidence),
    )
    .context("write trace load summary")?;

    println!(
        "load-sampler: PASS ({} scenarios; {} successful Ring dispatches; synthetic-only)",
        evidence.summary.scenarios, evidence.summary.successful_ring_dispatches
    );
    Ok(())
}

fn render_markdown_summary(evidence: &LoadEvidence) -> String {
    let mut output = format!(
        "# Runtime Event Trace Shadow Synthetic Load Samples\n\n\
         - Source commit: `{}`\n\
         - Source worktree dirty: `{}`\n\
         - Scope: synthetic-only, bounded, behavior-neutral concurrency evidence\n\
         - Authority: not a Runtime Event Stream, behavior input, or raw database export\n\n\
         ## Scenarios\n\n\
         | Scenario | Action | Workers × rounds × dispatches | Ring success | Enqueued | Persisted | Dropped | Elapsed | Last error |\n\
         | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n",
        evidence.source_commit, evidence.source_worktree_dirty
    );
    for scenario in &evidence.scenarios {
        output.push_str(&format!(
            "| `{}` | `{}` | {} × {} × {} | {}/{} | {} | {} | {} | {} ms | `{:?}` |\n",
            scenario.id,
            scenario.action.as_str(),
            scenario.workers,
            scenario.rounds,
            scenario.dispatches_per_worker,
            scenario.successful_ring_dispatches,
            scenario.attempted_dispatches,
            scenario.diagnostics.enqueued_dispatches,
            scenario.diagnostics.persisted_dispatches,
            scenario.diagnostics.dropped_dispatches,
            scenario.elapsed_ms,
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
