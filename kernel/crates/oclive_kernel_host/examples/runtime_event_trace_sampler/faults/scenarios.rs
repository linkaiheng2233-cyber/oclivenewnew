use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_host::infrastructure::sqlite_pool;
use oclive_kernel_types::{
    EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput,
    RuntimeEventTraceDiagnostics, RuntimeEventTraceErrorKind,
};

use super::contract::{FaultAction, FaultScenarioContract};
use crate::{build_kernel, sample_config};

const FAULT_EVENT_KIND: &str = "kernel.sample.trace.fault_probe";

pub(super) struct ScenarioRun {
    pub(super) successful_ring_dispatches: u64,
    pub(super) diagnostics: RuntimeEventTraceDiagnostics,
    pub(super) invariants: BTreeMap<String, bool>,
}

struct FaultTraceSource;

#[async_trait]
impl EventModule for FaultTraceSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "sample.trace_fault_source".into(),
            emissions: vec![FAULT_EVENT_KIND.into()],
            ..Default::default()
        }
    }

    async fn handle(
        &self,
        _event: &EventEnvelope,
    ) -> oclive_kernel_types::Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
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
    let emitter = kernel
        .register_event_module(Arc::new(FaultTraceSource))
        .map_err(anyhow::Error::msg)?;

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

    let dispatch_result = emit_many(
        emitter.as_ref(),
        scenario.id.as_str(),
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
    let diagnostics = wait_for_diagnostics(&kernel, |diagnostics| {
        diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::QueueFull)
            && diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
    })
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
    let emitter = kernel
        .register_event_module(Arc::new(FaultTraceSource))
        .map_err(anyhow::Error::msg)?;

    let sabotage_pool = sqlite_pool::connect_file(&trace_path)
        .await
        .context("open synthetic trace database for write-failure probe")?;
    sqlx::query("DROP TABLE runtime_event_trace_records")
        .execute(&sabotage_pool)
        .await
        .context("remove synthetic trace table for write-failure probe")?;
    sabotage_pool.close().await;

    let successful_ring_dispatches = emit_many(
        emitter.as_ref(),
        scenario.id.as_str(),
        scenario.attempted_dispatches,
    )
    .await?;
    let main_database_healthy = kernel.health_check().await.is_ok();
    let diagnostics = wait_for_diagnostics(&kernel, |diagnostics| {
        diagnostics.last_error_kind == Some(RuntimeEventTraceErrorKind::WriteFailed)
    })
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

async fn emit_many(
    emitter: &dyn EventEmitter,
    scenario_id: &str,
    attempted_dispatches: u64,
) -> anyhow::Result<u64> {
    let mut successful = 0_u64;
    for index in 0..attempted_dispatches {
        let correlation = format!("synthetic-fault:{scenario_id}:{index}");
        let result = emitter
            .emit(
                "synthetic:runtime-event-trace-fault-sample",
                Some(&correlation),
                EventDraft {
                    kind: FAULT_EVENT_KIND.into(),
                    payload: serde_json::json!({
                        "synthetic_private_body": "must-not-be-exported",
                    }),
                    metadata: [(
                        "synthetic.secret".into(),
                        serde_json::json!("must-not-be-exported"),
                    )]
                    .into_iter()
                    .collect(),
                },
            )
            .await
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if result.primary.kind != FAULT_EVENT_KIND {
            anyhow::bail!("fault sample Ring dispatch returned an unexpected event kind");
        }
        successful = successful.saturating_add(1);
    }
    Ok(successful)
}

async fn wait_for_diagnostics(
    kernel: &oclive_kernel_host::OcliveKernel,
    ready: impl Fn(&RuntimeEventTraceDiagnostics) -> bool,
) -> anyhow::Result<RuntimeEventTraceDiagnostics> {
    for _ in 0..1_000 {
        let diagnostics = kernel.runtime_event_trace_diagnostics();
        if ready(&diagnostics) {
            return Ok(diagnostics);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    anyhow::bail!("timed out waiting for synthetic trace fault diagnostics")
}

fn common_invariants(
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    main_database_healthy: bool,
    diagnostics: &RuntimeEventTraceDiagnostics,
) -> BTreeMap<String, bool> {
    [
        (
            "all_ring_dispatches_succeeded",
            successful_ring_dispatches == attempted_dispatches,
        ),
        (
            "dispatch_accounting_balanced",
            diagnostics
                .enqueued_dispatches
                .saturating_add(diagnostics.dropped_dispatches)
                == attempted_dispatches
                && diagnostics
                    .enqueued_events
                    .saturating_add(diagnostics.dropped_events)
                    == attempted_dispatches,
        ),
        ("main_database_healthy", main_database_healthy),
        (
            "privacy_flags_false",
            !diagnostics.captures_payloads
                && !diagnostics.captures_metadata
                && !diagnostics.captures_stream_key,
        ),
    ]
    .into_iter()
    .map(|(name, passed)| (name.into(), passed))
    .collect()
}

fn queue_invariants(
    attempted_dispatches: u64,
    successful_ring_dispatches: u64,
    main_database_healthy: bool,
    diagnostics: &RuntimeEventTraceDiagnostics,
) -> BTreeMap<String, bool> {
    let mut invariants = common_invariants(
        attempted_dispatches,
        successful_ring_dispatches,
        main_database_healthy,
        diagnostics,
    );
    invariants.insert(
        "all_enqueued_dispatches_drained".into(),
        diagnostics.persisted_dispatches == diagnostics.enqueued_dispatches
            && diagnostics.persisted_events == diagnostics.enqueued_events
            && diagnostics.duplicate_events == 0,
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
    let mut invariants = common_invariants(
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
        "no_dispatch_dropped".into(),
        diagnostics.dropped_dispatches == 0 && diagnostics.dropped_events == 0,
    );
    invariants
}
