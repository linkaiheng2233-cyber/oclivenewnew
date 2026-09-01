use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use async_trait::async_trait;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput,
    RuntimeEventTraceDiagnostics,
};
use tokio::sync::Barrier;
use tokio::task::JoinSet;

struct TraceProbeSource {
    module_id: String,
    event_kind: String,
}

#[async_trait]
impl EventModule for TraceProbeSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: self.module_id.clone(),
            emissions: vec![self.event_kind.clone()],
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

pub(crate) fn register_source(
    kernel: &oclive_kernel_host::OcliveKernel,
    module_id: &str,
    event_kind: &str,
) -> anyhow::Result<Arc<dyn EventEmitter>> {
    kernel
        .register_event_module(Arc::new(TraceProbeSource {
            module_id: module_id.into(),
            event_kind: event_kind.into(),
        }))
        .map_err(anyhow::Error::msg)
}

pub(crate) async fn emit_many(
    emitter: &dyn EventEmitter,
    event_kind: &str,
    stream_key: &str,
    correlation_prefix: &str,
    attempted_dispatches: u64,
) -> anyhow::Result<u64> {
    let mut successful = 0_u64;
    for index in 0..attempted_dispatches {
        let correlation = format!("{correlation_prefix}:{index}");
        let result = emitter
            .emit(
                stream_key,
                Some(&correlation),
                EventDraft {
                    kind: event_kind.into(),
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
        if result.primary.kind != event_kind {
            anyhow::bail!("trace probe Ring dispatch returned an unexpected event kind");
        }
        successful = successful.saturating_add(1);
    }
    Ok(successful)
}

pub(crate) struct ConcurrentEmitOutcome {
    pub(crate) completed_workers: u64,
    pub(crate) successful_ring_dispatches: u64,
}

pub(crate) async fn emit_concurrently(
    emitter: Arc<dyn EventEmitter>,
    event_kind: &'static str,
    stream_key: &'static str,
    correlation_prefix: String,
    workers: u16,
    dispatches_per_worker: u16,
) -> anyhow::Result<ConcurrentEmitOutcome> {
    let barrier = Arc::new(Barrier::new(usize::from(workers).saturating_add(1)));
    let mut tasks = JoinSet::new();
    for worker in 0..workers {
        let emitter = emitter.clone();
        let barrier = barrier.clone();
        let correlation_prefix = format!("{correlation_prefix}:w{worker}");
        tasks.spawn(async move {
            barrier.wait().await;
            emit_many(
                emitter.as_ref(),
                event_kind,
                stream_key,
                &correlation_prefix,
                u64::from(dispatches_per_worker),
            )
            .await
        });
    }
    barrier.wait().await;

    let mut outcome = ConcurrentEmitOutcome {
        completed_workers: 0,
        successful_ring_dispatches: 0,
    };
    while let Some(joined) = tasks.join_next().await {
        let successful = joined.context("join synthetic trace worker")??;
        outcome.completed_workers = outcome.completed_workers.saturating_add(1);
        outcome.successful_ring_dispatches = outcome
            .successful_ring_dispatches
            .saturating_add(successful);
    }
    Ok(outcome)
}

pub(crate) async fn wait_for_diagnostics(
    kernel: &oclive_kernel_host::OcliveKernel,
    maximum_wait: Duration,
    ready: impl Fn(&RuntimeEventTraceDiagnostics) -> bool,
    timeout_context: &str,
) -> anyhow::Result<RuntimeEventTraceDiagnostics> {
    let started = Instant::now();
    loop {
        let diagnostics = kernel.runtime_event_trace_diagnostics();
        if ready(&diagnostics) {
            return Ok(diagnostics);
        }
        if started.elapsed() >= maximum_wait {
            anyhow::bail!("timed out waiting for {timeout_context}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub(crate) fn common_invariants(
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
