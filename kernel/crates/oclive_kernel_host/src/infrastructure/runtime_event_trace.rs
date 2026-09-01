//! Optional, fail-open SQLite shadow for successful Event Ring dispatches.
//!
//! This recorder is deliberately downstream of Event Ring authority. The dispatch path performs
//! one bounded `try_send`; database I/O runs on a background task and can never veto an event.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context};
use chrono::{SecondsFormat, Utc};
use oclive_kernel_types::{
    EventEnvelope, RuntimeEventTraceDiagnostics, RuntimeEventTraceErrorKind,
    RUNTIME_EVENT_TRACE_DIAGNOSTICS_SCHEMA_VERSION,
};
use parking_lot::RwLock;
use sqlx::{Row, SqlitePool};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::domain::event_ring::EventDispatchTraceSink;

const TRACE_QUEUE_CAPACITY: usize = 256;
const TRACE_DATABASE_SCHEMA_VERSION: i64 = 1;
const TRACE_DATABASE_APPLICATION_ID: i64 = 0x4f43_5254; // "OCRT"

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceRecord {
    ring_schema_version: u16,
    event_id: String,
    event_kind: String,
    source: String,
    source_weight_bps: u16,
    correlation_id: String,
    causation_id: Option<String>,
    ring_sequence: u64,
    event_depth: u16,
    occurred_at: String,
}

impl From<&EventEnvelope> for TraceRecord {
    fn from(event: &EventEnvelope) -> Self {
        Self {
            ring_schema_version: event.schema_version,
            event_id: event.event_id.clone(),
            event_kind: event.kind.clone(),
            source: event.source.clone(),
            source_weight_bps: event.source_weight_bps,
            correlation_id: event.correlation_id.clone(),
            causation_id: event.causation_id.clone(),
            ring_sequence: event.sequence,
            event_depth: event.depth,
            occurred_at: event
                .occurred_at
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
        }
    }
}

enum TraceCommand {
    Record(Vec<TraceRecord>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Default)]
struct RuntimeEventTraceState {
    configured: bool,
    accepting_dispatches: AtomicBool,
    worker_active: AtomicBool,
    enqueued_dispatches: AtomicU64,
    enqueued_events: AtomicU64,
    persisted_dispatches: AtomicU64,
    persisted_events: AtomicU64,
    duplicate_events: AtomicU64,
    failed_dispatches: AtomicU64,
    failed_events: AtomicU64,
    dropped_dispatches: AtomicU64,
    dropped_events: AtomicU64,
    failure_count: AtomicU64,
    last_persisted_position: AtomicU64,
    last_error_kind: RwLock<Option<RuntimeEventTraceErrorKind>>,
}

impl RuntimeEventTraceState {
    fn new(configured: bool) -> Self {
        Self {
            configured,
            ..Self::default()
        }
    }

    fn record_failure(&self, kind: RuntimeEventTraceErrorKind) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_error_kind.write() = Some(kind);
    }

    fn record_failed_dispatch(&self, event_count: usize) {
        self.failed_dispatches.fetch_add(1, Ordering::Relaxed);
        self.failed_events
            .fetch_add(event_count as u64, Ordering::Relaxed);
    }

    fn diagnostics(&self) -> RuntimeEventTraceDiagnostics {
        let last_position = self.last_persisted_position.load(Ordering::Acquire);
        RuntimeEventTraceDiagnostics {
            schema_version: RUNTIME_EVENT_TRACE_DIAGNOSTICS_SCHEMA_VERSION,
            configured: self.configured,
            accepting_dispatches: self.accepting_dispatches.load(Ordering::Acquire),
            worker_active: self.worker_active.load(Ordering::Acquire),
            queue_capacity: if self.configured {
                TRACE_QUEUE_CAPACITY as u64
            } else {
                0
            },
            enqueued_dispatches: self.enqueued_dispatches.load(Ordering::Relaxed),
            enqueued_events: self.enqueued_events.load(Ordering::Relaxed),
            persisted_dispatches: self.persisted_dispatches.load(Ordering::Relaxed),
            persisted_events: self.persisted_events.load(Ordering::Relaxed),
            duplicate_events: self.duplicate_events.load(Ordering::Relaxed),
            failed_dispatches: self.failed_dispatches.load(Ordering::Relaxed),
            failed_events: self.failed_events.load(Ordering::Relaxed),
            dropped_dispatches: self.dropped_dispatches.load(Ordering::Relaxed),
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            last_persisted_position: (last_position != 0).then_some(last_position),
            last_error_kind: *self.last_error_kind.read(),
            captures_payloads: false,
            captures_metadata: false,
            captures_stream_key: false,
        }
    }
}

/// Per-kernel shadow recorder. A missing sender means disabled or startup-degraded operation.
pub(crate) struct RuntimeEventTrace {
    state: Arc<RuntimeEventTraceState>,
    sender: Option<mpsc::Sender<TraceCommand>>,
    shutdown_lock: Mutex<()>,
}

impl RuntimeEventTrace {
    fn inactive(state: Arc<RuntimeEventTraceState>) -> Arc<Self> {
        Arc::new(Self {
            state,
            sender: None,
            shutdown_lock: Mutex::new(()),
        })
    }

    pub(crate) async fn start(trace_path: Option<&Path>, main_database_path: &Path) -> Arc<Self> {
        let Some(trace_path) = trace_path else {
            return Self::inactive(Arc::new(RuntimeEventTraceState::new(false)));
        };

        let state = Arc::new(RuntimeEventTraceState::new(true));
        if paths_refer_to_same_file(trace_path, main_database_path) {
            state.record_failure(RuntimeEventTraceErrorKind::SchemaFailed);
            tracing::warn!(
                target: "oclive_runtime_event_trace",
                "runtime event trace path resolves to the main kernel database; shadow disabled"
            );
            return Self::inactive(state);
        }

        if let Some(parent) = trace_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            if let Err(error) = tokio::fs::create_dir_all(parent).await {
                state.record_failure(RuntimeEventTraceErrorKind::OpenFailed);
                tracing::warn!(
                    target: "oclive_runtime_event_trace",
                    error = %error,
                    "runtime event trace directory could not be created; shadow disabled"
                );
                return Self::inactive(state);
            }
        }

        let store = match RuntimeEventTraceStore::open(trace_path).await {
            Ok(store) => store,
            Err(error) => {
                state.record_failure(RuntimeEventTraceErrorKind::OpenFailed);
                tracing::warn!(
                    target: "oclive_runtime_event_trace",
                    error = %error,
                    "runtime event trace database could not be opened; shadow disabled"
                );
                return Self::inactive(state);
            }
        };

        let initial_position = match store.initialize().await {
            Ok(position) => position,
            Err(error) => {
                state.record_failure(RuntimeEventTraceErrorKind::SchemaFailed);
                tracing::warn!(
                    target: "oclive_runtime_event_trace",
                    error = %error,
                    "runtime event trace schema is unavailable; shadow disabled"
                );
                store.close().await;
                return Self::inactive(state);
            }
        };
        state
            .last_persisted_position
            .store(initial_position, Ordering::Release);

        let (sender, receiver) = mpsc::channel(TRACE_QUEUE_CAPACITY);
        state.accepting_dispatches.store(true, Ordering::Release);
        state.worker_active.store(true, Ordering::Release);
        tokio::spawn(run_trace_worker(store, receiver, state.clone()));
        Arc::new(Self {
            state,
            sender: Some(sender),
            shutdown_lock: Mutex::new(()),
        })
    }

    #[must_use]
    pub(crate) fn diagnostics(&self) -> RuntimeEventTraceDiagnostics {
        self.state.diagnostics()
    }

    #[must_use]
    pub(crate) fn is_recording(&self) -> bool {
        self.sender.is_some()
            && self.state.accepting_dispatches.load(Ordering::Acquire)
            && self.state.worker_active.load(Ordering::Acquire)
    }

    /// Drain all records queued before shutdown and close the independent SQLite pool.
    pub(crate) async fn shutdown(&self) {
        let _guard = self.shutdown_lock.lock().await;
        self.state
            .accepting_dispatches
            .store(false, Ordering::Release);
        if !self.state.worker_active.load(Ordering::Acquire) {
            return;
        }
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        let (done_sender, done_receiver) = oneshot::channel();
        if sender
            .send(TraceCommand::Shutdown(done_sender))
            .await
            .is_err()
        {
            if self.state.worker_active.load(Ordering::Acquire) {
                self.state
                    .record_failure(RuntimeEventTraceErrorKind::WorkerClosed);
            }
            return;
        }
        if done_receiver.await.is_err() && self.state.worker_active.load(Ordering::Acquire) {
            self.state
                .record_failure(RuntimeEventTraceErrorKind::WorkerClosed);
        }
    }
}

impl EventDispatchTraceSink for RuntimeEventTrace {
    fn record_successful_dispatch(&self, events: &[EventEnvelope]) {
        let Some(sender) = self.sender.as_ref() else {
            return;
        };
        if events.is_empty() {
            return;
        }
        let event_count = events.len() as u64;
        if !self.state.accepting_dispatches.load(Ordering::Acquire) {
            self.state
                .dropped_dispatches
                .fetch_add(1, Ordering::Relaxed);
            self.state
                .dropped_events
                .fetch_add(event_count, Ordering::Relaxed);
            return;
        }
        let records = events.iter().map(TraceRecord::from).collect();
        match sender.try_send(TraceCommand::Record(records)) {
            Ok(()) => {
                self.state
                    .enqueued_dispatches
                    .fetch_add(1, Ordering::Relaxed);
                self.state
                    .enqueued_events
                    .fetch_add(event_count, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.state
                    .dropped_dispatches
                    .fetch_add(1, Ordering::Relaxed);
                self.state
                    .dropped_events
                    .fetch_add(event_count, Ordering::Relaxed);
                self.state
                    .record_failure(RuntimeEventTraceErrorKind::QueueFull);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.state
                    .accepting_dispatches
                    .store(false, Ordering::Release);
                self.state
                    .dropped_dispatches
                    .fetch_add(1, Ordering::Relaxed);
                self.state
                    .dropped_events
                    .fetch_add(event_count, Ordering::Relaxed);
                self.state
                    .record_failure(RuntimeEventTraceErrorKind::WorkerClosed);
            }
        }
    }
}

async fn run_trace_worker(
    store: RuntimeEventTraceStore,
    mut receiver: mpsc::Receiver<TraceCommand>,
    state: Arc<RuntimeEventTraceState>,
) {
    let mut store_available = true;
    let mut shutdown_waiters = Vec::new();
    while let Some(command) = receiver.recv().await {
        match command {
            TraceCommand::Record(records) if store_available => {
                match store.append_dispatch(&records).await {
                    Ok(outcome) => {
                        state.persisted_dispatches.fetch_add(1, Ordering::Relaxed);
                        state
                            .persisted_events
                            .fetch_add(outcome.inserted_events, Ordering::Relaxed);
                        state
                            .duplicate_events
                            .fetch_add(outcome.duplicate_events, Ordering::Relaxed);
                        state
                            .last_persisted_position
                            .store(outcome.last_position, Ordering::Release);
                    }
                    Err(error) => {
                        state.record_failed_dispatch(records.len());
                        state.record_failure(RuntimeEventTraceErrorKind::WriteFailed);
                        state.accepting_dispatches.store(false, Ordering::Release);
                        receiver.close();
                        store_available = false;
                        tracing::warn!(
                            target: "oclive_runtime_event_trace",
                            event_count = records.len(),
                            error = %error,
                            "runtime event trace write failed; recorder stopped and Event Ring result remains authoritative"
                        );
                    }
                }
            }
            TraceCommand::Record(records) => {
                state.record_failed_dispatch(records.len());
            }
            TraceCommand::Shutdown(done) => {
                state.accepting_dispatches.store(false, Ordering::Release);
                receiver.close();
                shutdown_waiters.push(done);
            }
        }
    }
    store.close().await;
    state.accepting_dispatches.store(false, Ordering::Release);
    state.worker_active.store(false, Ordering::Release);
    for waiter in shutdown_waiters {
        let _ = waiter.send(());
    }
}

struct RuntimeEventTraceStore {
    pool: SqlitePool,
}

struct AppendOutcome {
    inserted_events: u64,
    duplicate_events: u64,
    last_position: u64,
}

impl RuntimeEventTraceStore {
    async fn open(path: &Path) -> anyhow::Result<Self> {
        let pool = super::sqlite_pool::connect_file(path)
            .await
            .context("connect trace SQLite")?;
        Ok(Self { pool })
    }

    async fn initialize(&self) -> anyhow::Result<u64> {
        let quick_check: String = sqlx::query_scalar("PRAGMA quick_check(1)")
            .fetch_one(&self.pool)
            .await
            .context("run trace SQLite quick_check")?;
        if quick_check != "ok" {
            bail!("trace SQLite quick_check failed");
        }

        let application_id: i64 = sqlx::query_scalar("PRAGMA application_id")
            .fetch_one(&self.pool)
            .await
            .context("read trace SQLite application_id")?;
        let user_version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&self.pool)
            .await
            .context("read trace SQLite user_version")?;

        if application_id == 0 && user_version == 0 {
            let user_table_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master \
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .fetch_one(&self.pool)
            .await
            .context("inspect unclaimed trace SQLite")?;
            if user_table_count != 0 {
                bail!("refusing to claim a non-empty SQLite database as runtime event trace");
            }
        } else if application_id != TRACE_DATABASE_APPLICATION_ID {
            bail!("SQLite application_id does not identify a runtime event trace database");
        }

        match user_version {
            0 => self.create_schema().await?,
            TRACE_DATABASE_SCHEMA_VERSION => self.ensure_append_only_triggers().await?,
            other => bail!("unsupported runtime event trace schema version {other}"),
        }
        self.verify_schema().await?;

        let last_position: Option<i64> =
            sqlx::query_scalar("SELECT MAX(position) FROM runtime_event_trace_records")
                .fetch_one(&self.pool)
                .await
                .context("read last trace position")?;
        last_position.map_or(Ok(0), |position| {
            u64::try_from(position).context("trace position is negative")
        })
    }

    async fn create_schema(&self) -> anyhow::Result<()> {
        let mut transaction = self.pool.begin().await.context("begin trace schema")?;
        sqlx::query(
            "CREATE TABLE runtime_event_trace_records (
                position INTEGER PRIMARY KEY AUTOINCREMENT,
                ring_schema_version INTEGER NOT NULL,
                event_id TEXT NOT NULL UNIQUE,
                event_kind TEXT NOT NULL,
                source TEXT NOT NULL,
                source_weight_bps INTEGER NOT NULL,
                correlation_id TEXT NOT NULL,
                causation_id TEXT,
                ring_sequence INTEGER NOT NULL,
                event_depth INTEGER NOT NULL,
                occurred_at TEXT NOT NULL,
                ingested_at TEXT NOT NULL
            )",
        )
        .execute(&mut *transaction)
        .await
        .context("create trace records table")?;
        sqlx::query(
            "CREATE INDEX runtime_event_trace_correlation_idx
             ON runtime_event_trace_records(correlation_id, position)",
        )
        .execute(&mut *transaction)
        .await
        .context("create trace correlation index")?;
        sqlx::query(&format!(
            "PRAGMA application_id = {TRACE_DATABASE_APPLICATION_ID}"
        ))
        .execute(&mut *transaction)
        .await
        .context("set trace SQLite application_id")?;
        sqlx::query(&format!(
            "PRAGMA user_version = {TRACE_DATABASE_SCHEMA_VERSION}"
        ))
        .execute(&mut *transaction)
        .await
        .context("set trace SQLite user_version")?;
        transaction.commit().await.context("commit trace schema")?;
        self.ensure_append_only_triggers().await
    }

    async fn ensure_append_only_triggers(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS runtime_event_trace_no_update
             BEFORE UPDATE ON runtime_event_trace_records
             BEGIN
                 SELECT RAISE(ABORT, 'runtime event trace is append-only');
             END",
        )
        .execute(&self.pool)
        .await
        .context("create trace no-update trigger")?;
        sqlx::query(
            "CREATE TRIGGER IF NOT EXISTS runtime_event_trace_no_delete
             BEFORE DELETE ON runtime_event_trace_records
             BEGIN
                 SELECT RAISE(ABORT, 'runtime event trace is append-only');
             END",
        )
        .execute(&self.pool)
        .await
        .context("create trace no-delete trigger")?;
        Ok(())
    }

    async fn verify_schema(&self) -> anyhow::Result<()> {
        const EXPECTED_COLUMNS: [&str; 12] = [
            "position",
            "ring_schema_version",
            "event_id",
            "event_kind",
            "source",
            "source_weight_bps",
            "correlation_id",
            "causation_id",
            "ring_sequence",
            "event_depth",
            "occurred_at",
            "ingested_at",
        ];
        let actual_columns = sqlx::query("PRAGMA table_info(runtime_event_trace_records)")
            .fetch_all(&self.pool)
            .await
            .context("inspect trace records schema")?
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect::<Vec<_>>();
        if actual_columns
            != EXPECTED_COLUMNS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        {
            bail!("runtime event trace columns do not match the privacy-minimized schema");
        }
        Ok(())
    }

    async fn append_dispatch(&self, records: &[TraceRecord]) -> anyhow::Result<AppendOutcome> {
        let ingested_at = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        let mut transaction = self.pool.begin().await.context("begin trace append")?;
        let mut inserted_events = 0_u64;
        for record in records {
            let ring_sequence =
                i64::try_from(record.ring_sequence).context("ring sequence exceeds SQLite i64")?;
            let result = sqlx::query(
                "INSERT INTO runtime_event_trace_records (
                    ring_schema_version, event_id, event_kind, source, source_weight_bps,
                    correlation_id, causation_id, ring_sequence, event_depth, occurred_at,
                    ingested_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(event_id) DO NOTHING",
            )
            .bind(i64::from(record.ring_schema_version))
            .bind(&record.event_id)
            .bind(&record.event_kind)
            .bind(&record.source)
            .bind(i64::from(record.source_weight_bps))
            .bind(&record.correlation_id)
            .bind(&record.causation_id)
            .bind(ring_sequence)
            .bind(i64::from(record.event_depth))
            .bind(&record.occurred_at)
            .bind(&ingested_at)
            .execute(&mut *transaction)
            .await
            .context("append trace event")?;
            inserted_events = inserted_events.saturating_add(result.rows_affected());
            if result.rows_affected() == 0 {
                let existing = sqlx::query(
                    "SELECT ring_schema_version, event_id, event_kind, source,
                            source_weight_bps, correlation_id, causation_id, ring_sequence,
                            event_depth, occurred_at
                     FROM runtime_event_trace_records WHERE event_id = ?",
                )
                .bind(&record.event_id)
                .fetch_one(&mut *transaction)
                .await
                .context("read duplicate trace event")?;
                let existing = trace_record_from_row(&existing)?;
                if existing != *record {
                    bail!("runtime event trace event_id conflicts with an existing header");
                }
            }
        }
        transaction.commit().await.context("commit trace append")?;

        let last_position: Option<i64> =
            sqlx::query_scalar("SELECT MAX(position) FROM runtime_event_trace_records")
                .fetch_one(&self.pool)
                .await
                .context("read appended trace position")?;
        let last_position = last_position.map_or(Ok(0), |position| {
            u64::try_from(position).context("trace position is negative")
        })?;
        let attempted_events = records.len() as u64;
        Ok(AppendOutcome {
            inserted_events,
            duplicate_events: attempted_events.saturating_sub(inserted_events),
            last_position,
        })
    }

    async fn close(self) {
        self.pool.close().await;
    }
}

fn trace_record_from_row(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<TraceRecord> {
    Ok(TraceRecord {
        ring_schema_version: u16::try_from(row.try_get::<i64, _>("ring_schema_version")?)
            .context("stored ring schema version is out of range")?,
        event_id: row.try_get("event_id")?,
        event_kind: row.try_get("event_kind")?,
        source: row.try_get("source")?,
        source_weight_bps: u16::try_from(row.try_get::<i64, _>("source_weight_bps")?)
            .context("stored source weight is out of range")?,
        correlation_id: row.try_get("correlation_id")?,
        causation_id: row.try_get("causation_id")?,
        ring_sequence: u64::try_from(row.try_get::<i64, _>("ring_sequence")?)
            .context("stored ring sequence is negative")?,
        event_depth: u16::try_from(row.try_get::<i64, _>("event_depth")?)
            .context("stored event depth is out of range")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

fn paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    if paths_equal_for_platform(left, right) {
        return true;
    }
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => paths_equal_for_platform(&left, &right),
        _ => false,
    }
}

#[cfg(windows)]
fn paths_equal_for_platform(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(not(windows))]
fn paths_equal_for_platform(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use oclive_kernel_types::EVENT_RING_SCHEMA_VERSION;
    use serde_json::json;

    use super::*;

    fn synthetic_event(event_id: &str, sequence: u64) -> EventEnvelope {
        EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: event_id.into(),
            kind: "kernel.test.trace.duplicate_probe".into(),
            source: "module.test.trace_duplicate_source".into(),
            source_weight_bps: 10_000,
            stream_key: "must-not-be-exported".into(),
            correlation_id: format!("synthetic-trace-correlation-{sequence}"),
            causation_id: None,
            sequence,
            depth: 0,
            occurred_at: Utc::now(),
            payload: json!({ "private": "must-not-be-exported" }),
            metadata: [("private".into(), json!("must-not-be-exported"))]
                .into_iter()
                .collect(),
        }
    }

    #[tokio::test]
    async fn duplicate_headers_are_idempotent_and_counted_without_replay_entry() {
        let temp = tempfile::tempdir().expect("create duplicate trace test directory");
        let trace_path = temp.path().join("runtime-event-trace.sqlite3");
        let main_database_path = temp.path().join("main-kernel.sqlite3");
        let trace = RuntimeEventTrace::start(Some(&trace_path), &main_database_path).await;
        assert!(trace.is_recording());

        let event = synthetic_event("synthetic-duplicate-event", 1);
        trace.record_successful_dispatch(std::slice::from_ref(&event));
        trace.record_successful_dispatch(std::slice::from_ref(&event));

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        let diagnostics = loop {
            let diagnostics = trace.diagnostics();
            if diagnostics.persisted_dispatches == 2 {
                break diagnostics;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "duplicate trace dispatches did not drain before the deadline"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        };

        assert!(diagnostics.configured);
        assert!(diagnostics.accepting_dispatches);
        assert!(diagnostics.worker_active);
        assert_eq!(diagnostics.enqueued_dispatches, 2);
        assert_eq!(diagnostics.enqueued_events, 2);
        assert_eq!(diagnostics.persisted_dispatches, 2);
        assert_eq!(diagnostics.persisted_events, 1);
        assert_eq!(diagnostics.duplicate_events, 1);
        assert_eq!(diagnostics.failed_dispatches, 0);
        assert_eq!(diagnostics.failed_events, 0);
        assert_eq!(diagnostics.dropped_dispatches, 0);
        assert_eq!(diagnostics.dropped_events, 0);
        assert_eq!(diagnostics.failure_count, 0);
        assert_eq!(diagnostics.last_error_kind, None);
        assert_eq!(diagnostics.last_persisted_position, Some(1));
        assert!(!diagnostics.captures_payloads);
        assert!(!diagnostics.captures_metadata);
        assert!(!diagnostics.captures_stream_key);

        let pool = super::super::sqlite_pool::connect_file(&trace_path)
            .await
            .expect("open duplicate trace test database");
        let persisted_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM runtime_event_trace_records")
                .fetch_one(&pool)
                .await
                .expect("count duplicate trace test records");
        assert_eq!(persisted_rows, 1);
        pool.close().await;

        trace.shutdown().await;
        let stopped_diagnostics = trace.diagnostics();
        assert!(!stopped_diagnostics.accepting_dispatches);
        assert!(!stopped_diagnostics.worker_active);

        let marker = json!({
            "schema_version": 1,
            "attempted_dispatches": 2,
            "configured": diagnostics.configured,
            "accepting_dispatches_during_sample": diagnostics.accepting_dispatches,
            "worker_active_during_sample": diagnostics.worker_active,
            "accepting_dispatches_after_shutdown": stopped_diagnostics.accepting_dispatches,
            "worker_stopped_after_shutdown": !stopped_diagnostics.worker_active,
            "enqueued_dispatches": diagnostics.enqueued_dispatches,
            "enqueued_events": diagnostics.enqueued_events,
            "persisted_dispatches": diagnostics.persisted_dispatches,
            "persisted_events": diagnostics.persisted_events,
            "duplicate_events": diagnostics.duplicate_events,
            "failed_dispatches": diagnostics.failed_dispatches,
            "failed_events": diagnostics.failed_events,
            "persisted_rows": persisted_rows,
            "dropped_dispatches": diagnostics.dropped_dispatches,
            "dropped_events": diagnostics.dropped_events,
            "failure_count": diagnostics.failure_count,
            "last_persisted_position": diagnostics.last_persisted_position,
            "captures_payloads": diagnostics.captures_payloads,
            "captures_metadata": diagnostics.captures_metadata,
            "captures_stream_key": diagnostics.captures_stream_key,
            "test_compilation_only": true,
            "production_replay_entry": false,
        });
        println!("runtime-event-trace-duplicate-evidence:{marker}");
    }

    #[tokio::test]
    async fn conflicting_duplicate_header_stops_recorder_without_overwriting_original() {
        let temp = tempfile::tempdir().expect("create conflicting trace test directory");
        let trace_path = temp.path().join("runtime-event-trace.sqlite3");
        let main_database_path = temp.path().join("main-kernel.sqlite3");
        let trace = RuntimeEventTrace::start(Some(&trace_path), &main_database_path).await;
        let event = synthetic_event("synthetic-conflicting-event", 1);
        trace.record_successful_dispatch(std::slice::from_ref(&event));

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while trace.diagnostics().persisted_dispatches != 1 {
            assert!(
                tokio::time::Instant::now() < deadline,
                "first trace header did not persist before the deadline"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let mut conflicting = event;
        conflicting.kind = "kernel.test.trace.conflicting_header".into();
        trace.record_successful_dispatch(std::slice::from_ref(&conflicting));
        let diagnostics = loop {
            let diagnostics = trace.diagnostics();
            if !diagnostics.worker_active {
                break diagnostics;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "conflicting trace header did not stop the worker before the deadline"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        };

        assert!(!diagnostics.accepting_dispatches);
        assert_eq!(diagnostics.enqueued_dispatches, 2);
        assert_eq!(diagnostics.enqueued_events, 2);
        assert_eq!(diagnostics.persisted_dispatches, 1);
        assert_eq!(diagnostics.persisted_events, 1);
        assert_eq!(diagnostics.duplicate_events, 0);
        assert_eq!(diagnostics.failed_dispatches, 1);
        assert_eq!(diagnostics.failed_events, 1);
        assert_eq!(diagnostics.failure_count, 1);
        assert_eq!(
            diagnostics.last_error_kind,
            Some(RuntimeEventTraceErrorKind::WriteFailed)
        );

        let pool = super::super::sqlite_pool::connect_file(&trace_path)
            .await
            .expect("open conflicting trace test database");
        let persisted_kind: String = sqlx::query_scalar(
            "SELECT event_kind FROM runtime_event_trace_records WHERE event_id = ?",
        )
        .bind("synthetic-conflicting-event")
        .fetch_one(&pool)
        .await
        .expect("read original trace header");
        assert_eq!(persisted_kind, "kernel.test.trace.duplicate_probe");
        pool.close().await;
        trace.shutdown().await;
    }

    #[tokio::test]
    async fn concurrent_shutdown_is_idempotent_and_drains_accepted_dispatches() {
        let temp = tempfile::tempdir().expect("create shutdown trace test directory");
        let trace_path = temp.path().join("runtime-event-trace.sqlite3");
        let main_database_path = temp.path().join("main-kernel.sqlite3");
        let trace = RuntimeEventTrace::start(Some(&trace_path), &main_database_path).await;
        for sequence in 1..=64 {
            let event = synthetic_event(&format!("synthetic-shutdown-event-{sequence}"), sequence);
            trace.record_successful_dispatch(std::slice::from_ref(&event));
        }

        tokio::join!(trace.shutdown(), trace.shutdown());
        let diagnostics = trace.diagnostics();
        assert!(!diagnostics.accepting_dispatches);
        assert!(!diagnostics.worker_active);
        assert_eq!(diagnostics.enqueued_dispatches, 64);
        assert_eq!(diagnostics.enqueued_events, 64);
        assert_eq!(diagnostics.persisted_dispatches, 64);
        assert_eq!(diagnostics.persisted_events, 64);
        assert_eq!(diagnostics.duplicate_events, 0);
        assert_eq!(diagnostics.failed_dispatches, 0);
        assert_eq!(diagnostics.failed_events, 0);
        assert_eq!(diagnostics.dropped_dispatches, 0);
        assert_eq!(diagnostics.dropped_events, 0);
        assert_eq!(diagnostics.failure_count, 0);
    }
}
