mod contract;
mod scenario;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context};
use chrono::{SecondsFormat, Utc};
use oclive_kernel_types::RuntimeEventTraceDiagnostics;
use serde::Serialize;
use sha2::{Digest, Sha256};

use contract::{parse_and_validate, SoakAction};

const EVIDENCE_FILE_NAME: &str = "runtime-event-trace-shadow.soak-evidence.json";
const SUMMARY_FILE_NAME: &str = "runtime-event-trace-shadow.soak-summary.md";

#[derive(Serialize)]
struct SoakEvidence {
    schema_version: u16,
    evidence_kind: String,
    authoritative_runtime_stream: bool,
    behavior_driver: bool,
    synthetic_only: bool,
    exports_raw_database: bool,
    bounded_execution: bool,
    routine_ci: bool,
    production_duration_claim: bool,
    generated_at: String,
    source_commit: String,
    source_worktree_dirty: bool,
    scenario_contract: String,
    scenario_contract_sha256: String,
    summary: SoakSummary,
    limitations: Vec<String>,
    scenarios: Vec<SoakScenarioEvidence>,
}

#[derive(Serialize)]
struct SoakSummary {
    scenarios: usize,
    passed: usize,
    steady_attempted_dispatches: u64,
    steady_successful_ring_dispatches: u64,
    post_restart_dispatches: u64,
    rows_after_shutdown: u64,
    rows_after_restart: u64,
    elapsed_ms: u64,
    rss_start_mib: u64,
    rss_peak_mib: u64,
    rss_end_mib: u64,
}

#[derive(Serialize)]
struct SoakScenarioEvidence {
    id: String,
    action: SoakAction,
    outcome: &'static str,
    duration_ms: u64,
    maximum_elapsed_ms: u64,
    interval_ms: u64,
    workers: u16,
    dispatches_per_worker_per_interval: u16,
    rounds: u64,
    expected_worker_runs: u64,
    health_check_interval_rounds: u64,
    expected_health_checks: u64,
    completed_health_checks: u64,
    maximum_rss_growth_mib: u64,
    attempted_dispatches: u64,
    completed_rounds: u64,
    completed_worker_runs: u64,
    successful_ring_dispatches: u64,
    post_restart_dispatches: u64,
    elapsed_ms: u64,
    rss_start_mib: u64,
    rss_peak_mib: u64,
    rss_end_mib: u64,
    rss_growth_mib: u64,
    steady_diagnostics: RuntimeEventTraceDiagnostics,
    restart_initial_diagnostics: RuntimeEventTraceDiagnostics,
    restart_diagnostics: RuntimeEventTraceDiagnostics,
    ring: SoakRingEvidence,
    persistence: SoakPersistenceEvidence,
    invariants: BTreeMap<String, bool>,
}

#[derive(Serialize)]
struct SoakRingEvidence {
    history_len: u64,
    history_capacity: u64,
    last_allocated_sequence: u64,
}

#[derive(Serialize)]
struct SoakPersistenceEvidence {
    rows_after_shutdown: u64,
    last_position_after_shutdown: Option<u64>,
    rows_after_restart: u64,
    last_position_after_restart: Option<u64>,
}

pub(crate) async fn run(
    contract_text: &str,
    scenario_contract: String,
    output_dir: &Path,
    source_commit: String,
    source_worktree_dirty: bool,
) -> anyhow::Result<()> {
    let contract = parse_and_validate(contract_text)?;
    let scenario = &contract.scenarios[0];
    let observed = scenario::execute(scenario).await?;
    if !observed.diagnostics.configured
        || !observed.diagnostics.accepting_dispatches
        || !observed.diagnostics.worker_active
        || observed.diagnostics.last_error_kind.is_some()
    {
        bail!("{} diagnostics drifted from its soak contract", scenario.id);
    }
    if observed.invariants.len() != scenario.expected_invariants.len() {
        bail!("{} produced an unexpected invariant set", scenario.id);
    }
    for invariant in &scenario.expected_invariants {
        if observed.invariants.get(invariant) != Some(&true) {
            bail!("{} failed invariant {invariant}", scenario.id);
        }
    }

    let scenario_evidence = SoakScenarioEvidence {
        id: scenario.id.clone(),
        action: scenario.action,
        outcome: "pass",
        duration_ms: scenario.duration_ms,
        maximum_elapsed_ms: scenario.maximum_elapsed_ms,
        interval_ms: scenario.interval_ms,
        workers: scenario.workers,
        dispatches_per_worker_per_interval: scenario.dispatches_per_worker_per_interval,
        rounds: observed.rounds,
        expected_worker_runs: observed.expected_worker_runs,
        health_check_interval_rounds: scenario.health_check_interval_rounds,
        expected_health_checks: observed.expected_health_checks,
        completed_health_checks: observed.completed_health_checks,
        maximum_rss_growth_mib: scenario.maximum_rss_growth_mib,
        attempted_dispatches: observed.attempted_dispatches,
        completed_rounds: observed.completed_rounds,
        completed_worker_runs: observed.completed_worker_runs,
        successful_ring_dispatches: observed.successful_ring_dispatches,
        post_restart_dispatches: observed.post_restart_dispatches,
        elapsed_ms: observed.elapsed_ms,
        rss_start_mib: observed.rss_start_mib,
        rss_peak_mib: observed.rss_peak_mib,
        rss_end_mib: observed.rss_end_mib,
        rss_growth_mib: observed.rss_peak_mib.saturating_sub(observed.rss_start_mib),
        steady_diagnostics: observed.diagnostics,
        restart_initial_diagnostics: observed.restart_initial_diagnostics,
        restart_diagnostics: observed.restart_diagnostics,
        ring: SoakRingEvidence {
            history_len: observed.ring_history_len,
            history_capacity: observed.ring_history_capacity,
            last_allocated_sequence: observed.ring_last_allocated_sequence,
        },
        persistence: SoakPersistenceEvidence {
            rows_after_shutdown: observed.rows_after_shutdown,
            last_position_after_shutdown: observed.last_position_after_shutdown,
            rows_after_restart: observed.rows_after_restart,
            last_position_after_restart: observed.last_position_after_restart,
        },
        invariants: observed.invariants,
    };
    let summary = SoakSummary {
        scenarios: 1,
        passed: 1,
        steady_attempted_dispatches: scenario_evidence.attempted_dispatches,
        steady_successful_ring_dispatches: scenario_evidence.successful_ring_dispatches,
        post_restart_dispatches: scenario_evidence.post_restart_dispatches,
        rows_after_shutdown: scenario_evidence.persistence.rows_after_shutdown,
        rows_after_restart: scenario_evidence.persistence.rows_after_restart,
        elapsed_ms: scenario_evidence.elapsed_ms,
        rss_start_mib: scenario_evidence.rss_start_mib,
        rss_peak_mib: scenario_evidence.rss_peak_mib,
        rss_end_mib: scenario_evidence.rss_end_mib,
    };
    let evidence = SoakEvidence {
        schema_version: 1,
        evidence_kind: contract.evidence_kind,
        authoritative_runtime_stream: false,
        behavior_driver: false,
        synthetic_only: true,
        exports_raw_database: false,
        bounded_execution: true,
        routine_ci: false,
        production_duration_claim: false,
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
            "This fixed ten-minute developer probe is intentionally excluded from routine CI."
                .into(),
            "Machine-local elapsed time and RSS growth are regression evidence, not a production-duration, throughput, or hardware SLA."
                .into(),
            "The trace remains behavior-neutral and cannot drive Event Ring, Session, memory, Prompt, model, output, or authoritative state."
                .into(),
            "No consumer, checkpoint, replay, retention, recovery, or Runtime Event Stream read contract is introduced."
                .into(),
        ],
        scenarios: vec![scenario_evidence],
    };

    std::fs::create_dir_all(output_dir).context("create trace soak evidence directory")?;
    let mut evidence_json =
        serde_json::to_string_pretty(&evidence).context("serialize trace soak evidence")?;
    evidence_json.push('\n');
    std::fs::write(output_dir.join(EVIDENCE_FILE_NAME), evidence_json)
        .context("write trace soak evidence")?;
    std::fs::write(
        output_dir.join(SUMMARY_FILE_NAME),
        render_markdown_summary(&evidence),
    )
    .context("write trace soak summary")?;

    println!(
        "soak-sampler: PASS (10 minutes; {} steady Ring dispatches; synthetic-only)",
        evidence.summary.steady_successful_ring_dispatches
    );
    Ok(())
}

fn render_markdown_summary(evidence: &SoakEvidence) -> String {
    let scenario = &evidence.scenarios[0];
    let mut output = format!(
        "# Runtime Event Trace Shadow Sustained Soak\n\n\
         - Result: PASS (1/1 fixed synthetic scenario)\n\
         - Source commit: `{}`\n\
         - Source worktree dirty: `{}`\n\
         - Duration: {} ms (machine-local elapsed {} ms)\n\
         - Steady Ring dispatches: {}/{}\n\
         - Health checks: {}/{}\n\
         - RSS start / peak / end: {} / {} / {} MiB\n\
         - Rows after shutdown / restart append: {} / {}\n\
         - Scope: synthetic-only, bounded, behavior-neutral, non-routine-CI evidence\n\
         - Authority: not a Runtime Event Stream or production-duration claim\n\n\
         ## Verified invariants\n\n",
        evidence.source_commit,
        evidence.source_worktree_dirty,
        scenario.duration_ms,
        scenario.elapsed_ms,
        scenario.successful_ring_dispatches,
        scenario.attempted_dispatches,
        scenario.completed_health_checks,
        scenario.expected_health_checks,
        scenario.rss_start_mib,
        scenario.rss_peak_mib,
        scenario.rss_end_mib,
        scenario.persistence.rows_after_shutdown,
        scenario.persistence.rows_after_restart,
    );
    for invariant in scenario.invariants.keys() {
        output.push_str(&format!("- `{invariant}`\n"));
    }
    output.push_str("\n## Limitations\n\n");
    for limitation in &evidence.limitations {
        output.push_str(&format!("- {limitation}\n"));
    }
    output
}
