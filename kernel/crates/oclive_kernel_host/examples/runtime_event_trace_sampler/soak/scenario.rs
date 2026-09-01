use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use anyhow::Context;
use oclive_kernel_host::infrastructure::sqlite_pool;
use oclive_kernel_types::RuntimeEventTraceDiagnostics;
use sysinfo::{Pid, ProcessesToUpdate, System};

use super::contract::SoakScenarioContract;
use crate::{build_kernel, probe, sample_config};

const SOAK_MODULE_ID: &str = "sample.trace_soak_source";
const SOAK_EVENT_KIND: &str = "kernel.sample.trace.sustained_soak_probe";
const SOAK_STREAM_KEY: &str = "synthetic:runtime-event-trace-sustained-soak";
const POST_RESTART_DISPATCHES: u64 = 1;
const DIAGNOSTICS_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
const MIB: u64 = 1024 * 1024;

pub(super) struct ScenarioRun {
    pub(super) rounds: u64,
    pub(super) attempted_dispatches: u64,
    pub(super) expected_worker_runs: u64,
    pub(super) completed_rounds: u64,
    pub(super) completed_worker_runs: u64,
    pub(super) successful_ring_dispatches: u64,
    pub(super) expected_health_checks: u64,
    pub(super) completed_health_checks: u64,
    pub(super) elapsed_ms: u64,
    pub(super) rss_start_mib: u64,
    pub(super) rss_peak_mib: u64,
    pub(super) rss_end_mib: u64,
    pub(super) diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) restart_initial_diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) restart_diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) ring_history_len: u64,
    pub(super) ring_history_capacity: u64,
    pub(super) ring_last_allocated_sequence: u64,
    pub(super) rows_after_shutdown: u64,
    pub(super) last_position_after_shutdown: Option<u64>,
    pub(super) rows_after_restart: u64,
    pub(super) last_position_after_restart: Option<u64>,
    pub(super) post_restart_dispatches: u64,
    pub(super) invariants: BTreeMap<String, bool>,
}

struct DispatchProgress {
    completed_rounds: u64,
    completed_worker_runs: u64,
    successful_ring_dispatches: u64,
    completed_health_checks: u64,
}

struct SustainedOutcome {
    progress: DispatchProgress,
    diagnostics: RuntimeEventTraceDiagnostics,
    ring_history_len: u64,
    ring_history_capacity: u64,
    ring_last_allocated_sequence: u64,
    main_database_healthy: bool,
    rss_peak_mib: u64,
    rss_end_mib: u64,
}

struct RestartOutcome {
    initial_diagnostics: RuntimeEventTraceDiagnostics,
    diagnostics: RuntimeEventTraceDiagnostics,
    successful_dispatches: u64,
    main_database_healthy: bool,
}

struct PersistenceSnapshot {
    rows: u64,
    last_position: Option<u64>,
}

struct ProcessRssSampler {
    system: System,
    pid: Pid,
}

impl ProcessRssSampler {
    fn new() -> anyhow::Result<Self> {
        let pid = Pid::from_u32(std::process::id());
        let mut sampler = Self {
            system: System::new(),
            pid,
        };
        sampler.sample_mib()?;
        Ok(sampler)
    }

    fn sample_mib(&mut self) -> anyhow::Result<u64> {
        self.system
            .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);
        let bytes = self
            .system
            .process(self.pid)
            .context("sample sustained-soak process RSS")?
            .memory();
        Ok(bytes.saturating_add(MIB - 1) / MIB)
    }
}

pub(super) async fn execute(scenario: &SoakScenarioContract) -> anyhow::Result<ScenarioRun> {
    let temp = tempfile::tempdir().context("create trace sustained-soak directory")?;
    let trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let config = sample_config(temp.path(), &trace_path)?;
    let kernel = build_kernel(config.clone()).await?;
    let emitter = probe::register_source(&kernel, SOAK_MODULE_ID, SOAK_EVENT_KIND)?;
    let rounds = scenario.rounds()?;
    let attempted_dispatches = scenario.attempted_dispatches()?;
    let expected_worker_runs = scenario.expected_worker_runs()?;
    let expected_health_checks = scenario.expected_health_checks()?;

    let mut rss_sampler = ProcessRssSampler::new()?;
    let rss_start_mib = rss_sampler.sample_mib()?;
    let started = Instant::now();
    let maximum_duration = Duration::from_millis(scenario.maximum_elapsed_ms);
    let execution = tokio::time::timeout(maximum_duration, async {
        let schedule_started = tokio::time::Instant::now();
        let mut progress = DispatchProgress {
            completed_rounds: 0,
            completed_worker_runs: 0,
            successful_ring_dispatches: 0,
            completed_health_checks: 0,
        };
        let mut rss_peak_mib = rss_start_mib;

        for round in 0..rounds {
            let dispatch = probe::emit_concurrently(
                emitter.clone(),
                SOAK_EVENT_KIND,
                SOAK_STREAM_KEY,
                format!("synthetic-soak:{}:r{round}", scenario.id),
                scenario.workers,
                scenario.dispatches_per_worker_per_interval,
            )
            .await?;
            progress.completed_rounds = progress.completed_rounds.saturating_add(1);
            progress.completed_worker_runs = progress
                .completed_worker_runs
                .saturating_add(dispatch.completed_workers);
            progress.successful_ring_dispatches = progress
                .successful_ring_dispatches
                .saturating_add(dispatch.successful_ring_dispatches);

            let completed_round = round.saturating_add(1);
            if completed_round.is_multiple_of(scenario.health_check_interval_rounds)
                && kernel.health_check().await.is_ok()
            {
                progress.completed_health_checks =
                    progress.completed_health_checks.saturating_add(1);
            }
            rss_peak_mib = rss_peak_mib.max(rss_sampler.sample_mib()?);

            let scheduled_elapsed =
                Duration::from_millis(completed_round.saturating_mul(scenario.interval_ms));
            tokio::time::sleep_until(schedule_started + scheduled_elapsed).await;
        }

        let diagnostics = probe::wait_for_diagnostics(
            &kernel,
            DIAGNOSTICS_DRAIN_TIMEOUT,
            |diagnostics| {
                diagnostics
                    .enqueued_dispatches
                    .saturating_add(diagnostics.dropped_dispatches)
                    == attempted_dispatches
                    && diagnostics
                        .persisted_dispatches
                        .saturating_add(diagnostics.failed_dispatches)
                        == diagnostics.enqueued_dispatches
            },
            "sustained synthetic trace diagnostics to drain",
        )
        .await?;
        let ring = kernel.event_ring_diagnostics(0);
        let main_database_healthy = kernel.health_check().await.is_ok();
        let rss_end_mib = rss_sampler.sample_mib()?;
        rss_peak_mib = rss_peak_mib.max(rss_end_mib);
        Ok::<SustainedOutcome, anyhow::Error>(SustainedOutcome {
            progress,
            diagnostics,
            ring_history_len: ring.history_len,
            ring_history_capacity: ring.history_capacity,
            ring_last_allocated_sequence: ring.last_allocated_sequence,
            main_database_healthy,
            rss_peak_mib,
            rss_end_mib,
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
                "{} exceeded its {} ms sustained-soak execution bound",
                scenario.id,
                scenario.maximum_elapsed_ms
            );
        }
    };
    kernel.shutdown().await;
    let after_shutdown = read_persistence(&trace_path).await?;

    let restart_kernel = build_kernel(config).await?;
    let restart_execution = async {
        let initial_diagnostics = restart_kernel.runtime_event_trace_diagnostics();
        let restart_emitter =
            probe::register_source(&restart_kernel, SOAK_MODULE_ID, SOAK_EVENT_KIND)?;
        let successful_dispatches = probe::emit_many(
            restart_emitter.as_ref(),
            SOAK_EVENT_KIND,
            SOAK_STREAM_KEY,
            &format!("synthetic-soak:{}:restart", scenario.id),
            POST_RESTART_DISPATCHES,
        )
        .await?;
        let diagnostics = probe::wait_for_diagnostics(
            &restart_kernel,
            DIAGNOSTICS_DRAIN_TIMEOUT,
            |diagnostics| {
                diagnostics.persisted_dispatches == POST_RESTART_DISPATCHES
                    && diagnostics.persisted_events == POST_RESTART_DISPATCHES
            },
            "post-restart synthetic trace append",
        )
        .await?;
        let main_database_healthy = restart_kernel.health_check().await.is_ok();
        Ok::<RestartOutcome, anyhow::Error>(RestartOutcome {
            initial_diagnostics,
            diagnostics,
            successful_dispatches,
            main_database_healthy,
        })
    }
    .await;
    restart_kernel.shutdown().await;
    let restart = restart_execution?;
    let after_restart = read_persistence(&trace_path).await?;

    let invariant_input = InvariantInput {
        rounds,
        attempted_dispatches,
        expected_worker_runs,
        expected_health_checks,
        elapsed_ms,
        rss_start_mib,
        after_shutdown: &after_shutdown,
        after_restart: &after_restart,
        sustained: &outcome,
        restart: &restart,
    };
    let invariants = soak_invariants(scenario, &invariant_input);
    Ok(ScenarioRun {
        rounds,
        attempted_dispatches,
        expected_worker_runs,
        completed_rounds: outcome.progress.completed_rounds,
        completed_worker_runs: outcome.progress.completed_worker_runs,
        successful_ring_dispatches: outcome.progress.successful_ring_dispatches,
        expected_health_checks,
        completed_health_checks: outcome.progress.completed_health_checks,
        elapsed_ms,
        rss_start_mib,
        rss_peak_mib: outcome.rss_peak_mib,
        rss_end_mib: outcome.rss_end_mib,
        diagnostics: outcome.diagnostics,
        restart_initial_diagnostics: restart.initial_diagnostics,
        restart_diagnostics: restart.diagnostics,
        ring_history_len: outcome.ring_history_len,
        ring_history_capacity: outcome.ring_history_capacity,
        ring_last_allocated_sequence: outcome.ring_last_allocated_sequence,
        rows_after_shutdown: after_shutdown.rows,
        last_position_after_shutdown: after_shutdown.last_position,
        rows_after_restart: after_restart.rows,
        last_position_after_restart: after_restart.last_position,
        post_restart_dispatches: restart.successful_dispatches,
        invariants,
    })
}

struct InvariantInput<'a> {
    rounds: u64,
    attempted_dispatches: u64,
    expected_worker_runs: u64,
    expected_health_checks: u64,
    elapsed_ms: u64,
    rss_start_mib: u64,
    after_shutdown: &'a PersistenceSnapshot,
    after_restart: &'a PersistenceSnapshot,
    sustained: &'a SustainedOutcome,
    restart: &'a RestartOutcome,
}

fn soak_invariants(
    scenario: &SoakScenarioContract,
    input: &InvariantInput<'_>,
) -> BTreeMap<String, bool> {
    let diagnostics = &input.sustained.diagnostics;
    let mut invariants = probe::common_invariants(
        input.attempted_dispatches,
        input.sustained.progress.successful_ring_dispatches,
        input.sustained.main_database_healthy,
        diagnostics,
    );
    invariants.insert(
        "all_rounds_completed".into(),
        input.sustained.progress.completed_rounds == input.rounds,
    );
    invariants.insert(
        "all_workers_completed".into(),
        input.sustained.progress.completed_worker_runs == input.expected_worker_runs,
    );
    invariants.insert(
        "all_enqueued_dispatches_drained".into(),
        diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
            && diagnostics.failed_dispatches == 0
            && diagnostics.persisted_events == diagnostics.enqueued_events
            && diagnostics.failed_events == 0,
    );
    invariants.insert(
        "no_trace_drop_duplicate_or_failure".into(),
        diagnostics.dropped_dispatches == 0
            && diagnostics.dropped_events == 0
            && diagnostics.duplicate_events == 0
            && diagnostics.failed_dispatches == 0
            && diagnostics.failed_events == 0
            && diagnostics.failure_count == 0
            && diagnostics.last_error_kind.is_none(),
    );
    invariants.insert(
        "trace_worker_remained_active".into(),
        diagnostics.configured && diagnostics.accepting_dispatches && diagnostics.worker_active,
    );
    invariants.insert(
        "bounded_ring_history_preserved".into(),
        input.sustained.ring_history_len
            == input
                .attempted_dispatches
                .min(input.sustained.ring_history_capacity)
            && input.sustained.ring_last_allocated_sequence == input.attempted_dispatches,
    );
    invariants.insert(
        "periodic_main_database_health".into(),
        input.sustained.progress.completed_health_checks == input.expected_health_checks,
    );
    invariants.insert(
        "rss_growth_within_contract".into(),
        input
            .sustained
            .rss_peak_mib
            .saturating_sub(input.rss_start_mib)
            <= scenario.maximum_rss_growth_mib,
    );
    invariants.insert(
        "execution_time_within_contract".into(),
        input.elapsed_ms >= scenario.duration_ms && input.elapsed_ms <= scenario.maximum_elapsed_ms,
    );
    invariants.insert(
        "shutdown_persisted_all_rows".into(),
        input.after_shutdown.rows == input.attempted_dispatches
            && input.after_shutdown.last_position == Some(input.attempted_dispatches)
            && diagnostics.last_persisted_position == Some(input.attempted_dispatches),
    );
    invariants.insert(
        "restart_position_continuity".into(),
        input.restart.initial_diagnostics.configured
            && input.restart.initial_diagnostics.accepting_dispatches
            && input.restart.initial_diagnostics.worker_active
            && input.restart.initial_diagnostics.enqueued_dispatches == 0
            && input.restart.initial_diagnostics.enqueued_events == 0
            && input.restart.initial_diagnostics.persisted_dispatches == 0
            && input.restart.initial_diagnostics.persisted_events == 0
            && input.restart.initial_diagnostics.duplicate_events == 0
            && input.restart.initial_diagnostics.failed_dispatches == 0
            && input.restart.initial_diagnostics.failed_events == 0
            && input.restart.initial_diagnostics.dropped_dispatches == 0
            && input.restart.initial_diagnostics.dropped_events == 0
            && input.restart.initial_diagnostics.failure_count == 0
            && input.restart.initial_diagnostics.last_error_kind.is_none()
            && input.restart.initial_diagnostics.last_persisted_position
                == input.after_shutdown.last_position,
    );
    let expected_position = input
        .attempted_dispatches
        .checked_add(POST_RESTART_DISPATCHES);
    invariants.insert(
        "post_restart_append_succeeded".into(),
        input.restart.diagnostics.configured
            && input.restart.diagnostics.accepting_dispatches
            && input.restart.diagnostics.worker_active
            && input.restart.successful_dispatches == POST_RESTART_DISPATCHES
            && input.restart.diagnostics.enqueued_dispatches == POST_RESTART_DISPATCHES
            && input.restart.diagnostics.persisted_dispatches == POST_RESTART_DISPATCHES
            && input.restart.diagnostics.persisted_events == POST_RESTART_DISPATCHES
            && input.restart.diagnostics.duplicate_events == 0
            && input.restart.diagnostics.failed_dispatches == 0
            && input.restart.diagnostics.failed_events == 0
            && input.restart.diagnostics.dropped_dispatches == 0
            && input.restart.diagnostics.dropped_events == 0
            && input.restart.diagnostics.failure_count == 0
            && input.restart.diagnostics.last_error_kind.is_none()
            && !input.restart.diagnostics.captures_payloads
            && !input.restart.diagnostics.captures_metadata
            && !input.restart.diagnostics.captures_stream_key
            && input.restart.diagnostics.last_persisted_position == expected_position
            && input.after_restart.rows == expected_position.unwrap_or(u64::MAX)
            && input.after_restart.last_position == expected_position,
    );
    invariants.insert(
        "post_restart_main_database_healthy".into(),
        input.restart.main_database_healthy,
    );
    invariants
}

async fn read_persistence(path: &std::path::Path) -> anyhow::Result<PersistenceSnapshot> {
    let pool = sqlite_pool::connect_file(path)
        .await
        .with_context(|| format!("open sustained-soak trace database {}", path.display()))?;
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM runtime_event_trace_records")
        .fetch_one(&pool)
        .await
        .context("count sustained-soak trace records")?;
    let last_position: Option<i64> =
        sqlx::query_scalar("SELECT MAX(position) FROM runtime_event_trace_records")
            .fetch_one(&pool)
            .await
            .context("read sustained-soak final position")?;
    pool.close().await;
    Ok(PersistenceSnapshot {
        rows: u64::try_from(rows).context("sustained-soak row count is negative")?,
        last_position: last_position
            .map(|position| u64::try_from(position).context("sustained-soak position is negative"))
            .transpose()?,
    })
}
