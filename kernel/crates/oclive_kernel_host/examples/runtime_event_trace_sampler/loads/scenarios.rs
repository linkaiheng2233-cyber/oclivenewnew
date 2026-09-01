use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use oclive_kernel_contracts::EventEmitter;
use oclive_kernel_types::{RuntimeEventTraceDiagnostics, RuntimeEventTraceErrorKind};
use tokio::sync::Barrier;
use tokio::task::JoinSet;

use super::contract::LoadScenarioContract;
use crate::{build_kernel, probe, sample_config};

const LOAD_MODULE_ID: &str = "sample.trace_load_source";
const LOAD_EVENT_KIND: &str = "kernel.sample.trace.load_probe";
const LOAD_STREAM_KEY: &str = "synthetic:runtime-event-trace-load-sample";

pub(super) struct ScenarioRun {
    pub(super) attempted_dispatches: u64,
    pub(super) completed_rounds: u64,
    pub(super) completed_worker_runs: u64,
    pub(super) successful_ring_dispatches: u64,
    pub(super) elapsed_ms: u64,
    pub(super) diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) ring_history_len: u64,
    pub(super) ring_history_capacity: u64,
    pub(super) ring_last_allocated_sequence: u64,
    pub(super) invariants: BTreeMap<String, bool>,
}

struct DispatchProgress {
    completed_rounds: u64,
    completed_worker_runs: u64,
    successful_ring_dispatches: u64,
}

struct ExecutionOutcome {
    progress: DispatchProgress,
    diagnostics: RuntimeEventTraceDiagnostics,
    ring_history_len: u64,
    ring_history_capacity: u64,
    ring_last_allocated_sequence: u64,
    main_database_healthy: bool,
}

pub(super) async fn execute(scenario: &LoadScenarioContract) -> anyhow::Result<ScenarioRun> {
    let temp = tempfile::tempdir().context("create trace load sample directory")?;
    let trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let kernel = build_kernel(sample_config(temp.path(), &trace_path)?).await?;
    let emitter = probe::register_source(&kernel, LOAD_MODULE_ID, LOAD_EVENT_KIND)?;
    let attempted_dispatches = scenario.attempted_dispatches()?;
    let maximum_duration = Duration::from_millis(scenario.maximum_elapsed_ms);
    let started = Instant::now();

    let execution = tokio::time::timeout(maximum_duration, async {
        let progress = dispatch_rounds(emitter, scenario).await?;
        let diagnostics = probe::wait_for_diagnostics(
            &kernel,
            maximum_duration,
            |diagnostics| {
                diagnostics
                    .enqueued_dispatches
                    .saturating_add(diagnostics.dropped_dispatches)
                    == attempted_dispatches
                    && diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
            },
            "synthetic trace load diagnostics to drain",
        )
        .await?;
        let ring = kernel.event_ring_diagnostics(0);
        let main_database_healthy = kernel.health_check().await.is_ok();
        Ok::<ExecutionOutcome, anyhow::Error>(ExecutionOutcome {
            progress,
            diagnostics,
            ring_history_len: ring.history_len,
            ring_history_capacity: ring.history_capacity,
            ring_last_allocated_sequence: ring.last_allocated_sequence,
            main_database_healthy,
        })
    })
    .await;
    let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    let outcome = match execution {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(error)) => {
            kernel.shutdown().await;
            return Err(error);
        }
        Err(_) => {
            kernel.shutdown().await;
            anyhow::bail!(
                "{} exceeded its {} ms execution bound",
                scenario.id,
                scenario.maximum_elapsed_ms
            );
        }
    };
    kernel.shutdown().await;

    let invariants = load_invariants(scenario, attempted_dispatches, elapsed_ms, &outcome);
    Ok(ScenarioRun {
        attempted_dispatches,
        completed_rounds: outcome.progress.completed_rounds,
        completed_worker_runs: outcome.progress.completed_worker_runs,
        successful_ring_dispatches: outcome.progress.successful_ring_dispatches,
        elapsed_ms,
        diagnostics: outcome.diagnostics,
        ring_history_len: outcome.ring_history_len,
        ring_history_capacity: outcome.ring_history_capacity,
        ring_last_allocated_sequence: outcome.ring_last_allocated_sequence,
        invariants,
    })
}

async fn dispatch_rounds(
    emitter: Arc<dyn EventEmitter>,
    scenario: &LoadScenarioContract,
) -> anyhow::Result<DispatchProgress> {
    let mut progress = DispatchProgress {
        completed_rounds: 0,
        completed_worker_runs: 0,
        successful_ring_dispatches: 0,
    };
    for round in 0..scenario.rounds {
        let round_progress = dispatch_round(emitter.clone(), scenario, round).await?;
        progress.completed_rounds = progress
            .completed_rounds
            .saturating_add(round_progress.completed_rounds);
        progress.completed_worker_runs = progress
            .completed_worker_runs
            .saturating_add(round_progress.completed_worker_runs);
        progress.successful_ring_dispatches = progress
            .successful_ring_dispatches
            .saturating_add(round_progress.successful_ring_dispatches);
        if round.saturating_add(1) < scenario.rounds {
            tokio::time::sleep(Duration::from_millis(scenario.pause_between_rounds_ms)).await;
        }
    }
    Ok(progress)
}

async fn dispatch_round(
    emitter: Arc<dyn EventEmitter>,
    scenario: &LoadScenarioContract,
    round: u16,
) -> anyhow::Result<DispatchProgress> {
    let barrier = Arc::new(Barrier::new(
        usize::from(scenario.workers).saturating_add(1),
    ));
    let mut tasks = JoinSet::new();
    for worker in 0..scenario.workers {
        let emitter = emitter.clone();
        let barrier = barrier.clone();
        let correlation_prefix = format!("synthetic-load:{}:r{round}:w{worker}", scenario.id);
        let dispatches = u64::from(scenario.dispatches_per_worker);
        tasks.spawn(async move {
            barrier.wait().await;
            probe::emit_many(
                emitter.as_ref(),
                LOAD_EVENT_KIND,
                LOAD_STREAM_KEY,
                &correlation_prefix,
                dispatches,
            )
            .await
        });
    }
    barrier.wait().await;

    let mut progress = DispatchProgress {
        completed_rounds: 1,
        completed_worker_runs: 0,
        successful_ring_dispatches: 0,
    };
    while let Some(joined) = tasks.join_next().await {
        let successful = joined.context("join synthetic trace load worker")??;
        progress.completed_worker_runs = progress.completed_worker_runs.saturating_add(1);
        progress.successful_ring_dispatches = progress
            .successful_ring_dispatches
            .saturating_add(successful);
    }
    Ok(progress)
}

fn load_invariants(
    scenario: &LoadScenarioContract,
    attempted_dispatches: u64,
    elapsed_ms: u64,
    outcome: &ExecutionOutcome,
) -> BTreeMap<String, bool> {
    let mut invariants = probe::common_invariants(
        attempted_dispatches,
        outcome.progress.successful_ring_dispatches,
        outcome.main_database_healthy,
        &outcome.diagnostics,
    );
    invariants.insert(
        "all_rounds_completed".into(),
        outcome.progress.completed_rounds == u64::from(scenario.rounds),
    );
    invariants.insert(
        "all_workers_completed".into(),
        outcome.progress.completed_worker_runs == scenario.expected_worker_runs(),
    );
    invariants.insert(
        "all_enqueued_dispatches_drained".into(),
        outcome.diagnostics.persisted_dispatches == outcome.diagnostics.enqueued_dispatches
            && outcome
                .diagnostics
                .persisted_events
                .saturating_add(outcome.diagnostics.duplicate_events)
                == outcome.diagnostics.enqueued_events,
    );
    invariants.insert(
        "no_duplicate_events".into(),
        outcome.diagnostics.duplicate_events == 0,
    );
    invariants.insert(
        "only_expected_trace_errors".into(),
        match outcome.diagnostics.last_error_kind {
            None => {
                outcome.diagnostics.failure_count == 0
                    && outcome.diagnostics.dropped_dispatches == 0
            }
            Some(RuntimeEventTraceErrorKind::QueueFull) => {
                outcome.diagnostics.dropped_dispatches > 0
                    && outcome.diagnostics.failure_count == outcome.diagnostics.dropped_dispatches
            }
            Some(_) => false,
        },
    );
    invariants.insert(
        "trace_worker_remained_active".into(),
        outcome.diagnostics.configured && outcome.diagnostics.worker_active,
    );
    invariants.insert(
        "bounded_ring_history_preserved".into(),
        outcome.ring_history_len == attempted_dispatches.min(outcome.ring_history_capacity)
            && outcome.ring_last_allocated_sequence == attempted_dispatches,
    );
    invariants.insert(
        "execution_time_within_contract".into(),
        elapsed_ms >= scenario.minimum_elapsed_ms && elapsed_ms <= scenario.maximum_elapsed_ms,
    );
    invariants
}
