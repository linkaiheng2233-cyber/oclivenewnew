use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Context;
use oclive_kernel_host::infrastructure::sqlite_pool;
use oclive_kernel_types::{RuntimeEventTraceDiagnostics, RuntimeEventTraceErrorKind};

use super::contract::{FaultAction, FaultScenarioContract};
use crate::{build_kernel, probe, sample_config};

const FAULT_MODULE_ID: &str = "sample.trace_fault_source";
const FAULT_EVENT_KIND: &str = "kernel.sample.trace.fault_probe";
const FAULT_STREAM_KEY: &str = "synthetic:runtime-event-trace-fault-sample";

pub(super) struct ScenarioRun {
    pub(super) successful_ring_dispatches: u64,
    pub(super) diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) invariants: BTreeMap<String, bool>,
}

pub(super) async fn execute(scenario: &FaultScenarioContract) -> anyhow::Result<ScenarioRun> {
    match scenario.action {
        FaultAction::QueueSaturation => queue_saturation(scenario).await,
        FaultAction::PostStartWriteFailure => post_start_write_failure(scenario).await,
    }
}

async fn queue_saturation(scenario: &FaultScenarioContract) -> anyhow::Result<ScenarioRun> {
    let temp = tempfile::tempdir().context("create queue saturation sample directory")?;
    let trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let kernel = build_kernel(sample_config(temp.path(), &trace_path)?).await?;
    let emitter = probe::register_source(&kernel, FAULT_MODULE_ID, FAULT_EVENT_KIND)?;

    let lock_pool = sqlite_pool::connect_file(&trace_path)
        .await
        .context("open synthetic trace database for write lock")?;
    let mut lock_connection = lock_pool
        .acquire()
        .await
        .context("acquire synthetic trace write-lock connection")?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *lock_connection)
        .await
        .context("hold synthetic trace writer")?;

    let dispatch_result = probe::emit_many(
        emitter.as_ref(),
        FAULT_EVENT_KIND,
        FAULT_STREAM_KEY,
        &format!("synthetic-fault:{}", scenario.id),
        scenario.attempted_dispatches,
    )
    .await;
    let rollback_result = sqlx::query("ROLLBACK")
        .execute(&mut *lock_connection)
        .await
        .context("release synthetic trace writer");
    drop(lock_connection);
    lock_pool.close().await;
    let successful_ring_dispatches = dispatch_result?;
    rollback_result?;

    let main_database_healthy = kernel.health_check().await.is_ok();
    let diagnostics = probe::wait_for_diagnostics(
        &kernel,
        Duration::from_secs(10),
        |diagnostics| {
            diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::QueueFull)
                && diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
        },
        "synthetic trace queue-saturation diagnostics",
    )
    .await?;
    kernel.shutdown().await;
    let invariants = queue_invariants(
        scenario.attempted_dispatches,
        successful_ring_dispatches,
        main_database_healthy,
        &diagnostics,
    );
    Ok(ScenarioRun {
        successful_ring_dispatches,
        diagnostics,
        invariants,
    })
}

async fn post_start_write_failure(scenario: &FaultScenarioContract) -> anyhow::Result<ScenarioRun> {
    let temp = tempfile::tempdir().context("create write failure sample directory")?;
    let trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let kernel = build_kernel(sample_config(temp.path(), &trace_path)?).await?;
    let emitter = probe::register_source(&kernel, FAULT_MODULE_ID, FAULT_EVENT_KIND)?;

    let sabotage_pool = sqlite_pool::connect_file(&trace_path)
        .await
        .context("open synthetic trace database for write-failure probe")?;
    sqlx::query("DROP TABLE runtime_event_trace_records")
        .execute(&sabotage_pool)
        .await
        .context("remove synthetic trace table for write-failure probe")?;
    sabotage_pool.close().await;

    let successful_ring_dispatches = probe::emit_many(
        emitter.as_ref(),
        FAULT_EVENT_KIND,
        FAULT_STREAM_KEY,
        &format!("synthetic-fault:{}", scenario.id),
        scenario.attempted_dispatches,
    )
    .await?;
    let main_database_healthy = kernel.health_check().await.is_ok();
    let diagnostics = probe::wait_for_diagnostics(
        &kernel,
        Duration::from_secs(10),
        |diagnostics| {
            diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::WriteFailed)
                && !diagnostics.accepting_dispatches
                && !diagnostics.worker_active
                && diagnostics.persisted_dispatches + diagnostics.failed_dispatches
                    == diagnostics.enqueued_dispatches
        },
        "synthetic trace write-failure diagnostics",
    )
    .await?;
    kernel.shutdown().await;
    let invariants = write_failure_invariants(
        scenario.attempted_dispatches,
        successful_ring_dispatches,
        main_database_healthy,
        &diagnostics,
    );
    Ok(ScenarioRun {
        successful_ring_dispatches,
        diagnostics,
        invariants,
    })
}

fn queue_invariants(
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    main_database_healthy: bool,
    diagnostics: &RuntimeEventTraceDiagnostics,
) -> BTreeMap<String, bool> {
    let mut invariants = probe::common_invariants(
        attempted_dispatches,
        successful_ring_dispatches,
        main_database_healthy,
        diagnostics,
    );
    invariants.insert(
        "all_enqueued_dispatches_drained".into(),
        diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
            && diagnostics.persisted_events == diagnostics.enqueued_events
            && diagnostics.duplicate_events == 0
            && diagnostics.failed_dispatches == 0
            && diagnostics.failed_events == 0,
    );
    invariants.insert(
        "dropped_dispatches_observed".into(),
        diagnostics.dropped_dispatches > 0
            && diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::QueueFull),
    );
    invariants
}

fn write_failure_invariants(
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    main_database_healthy: bool,
    diagnostics: &RuntimeEventTraceDiagnostics,
) -> BTreeMap<String, bool> {
    let mut invariants = probe::common_invariants(
        attempted_dispatches,
        successful_ring_dispatches,
        main_database_healthy,
        diagnostics,
    );
    invariants.insert(
        "write_failure_observed".into(),
        diagnostics.failure_count > 0
            && diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::WriteFailed)
            && diagnostics.persisted_dispatches == 0
            && diagnostics.persisted_events == 0,
    );
    invariants.insert(
        "write_failure_accounted".into(),
        diagnostics.failed_dispatches == diagnostics.enqueued_dispatches
            && diagnostics.failed_events == diagnostics.enqueued_events,
    );
    invariants.insert(
        "recorder_stopped_after_write_failure".into(),
        !diagnostics.accepting_dispatches && !diagnostics.worker_active,
    );
    invariants.insert(
        "no_dispatch_dropped".into(),
        diagnostics.dropped_dispatches == 0 && diagnostics.dropped_events == 0,
    );
    invariants
}
