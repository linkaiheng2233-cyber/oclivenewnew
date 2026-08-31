#![allow(clippy::expect_used)]

use std::sync::Arc;

use async_trait::async_trait;
use oclive_kernel_contracts::{EventModule, EventModuleRegistrar};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::{OcliveKernel, OcliveKernelConfig};
use oclive_kernel_types::{
    EventDispatchResult, EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleOutput,
    RuntimeEventTraceErrorKind,
};
use sqlx::Row;

const TRACE_EVENT_KIND: &str = "kernel.test.runtime_trace.observed";

struct TraceSource;

#[async_trait]
impl EventModule for TraceSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.runtime_trace_source".into(),
            emissions: vec![TRACE_EVENT_KIND.into()],
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

fn roles_dir() -> std::path::PathBuf {
    oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("distros/chat-pro/roles")
}

async fn build_kernel(config: OcliveKernelConfig) -> OcliveKernel {
    OcliveKernel::builder(config)
        .with_host_profile(HostProfile::default())
        .with_llm_client(Arc::new(MockLlmClient {
            reply: "trace shadow reply".into(),
        }))
        .build()
        .await
        .expect("build role kernel")
}

async fn emit_trace_event(kernel: &OcliveKernel, label: &str) -> EventDispatchResult {
    let emitter = kernel
        .register_event_module(Arc::new(TraceSource))
        .expect("register trace source");
    emitter
        .emit(
            "runtime-trace-test-stream",
            Some(label),
            EventDraft {
                kind: TRACE_EVENT_KIND.into(),
                payload: serde_json::json!({
                    "private": "must-not-be-persisted",
                    "label": label,
                }),
                metadata: [("secret".into(), serde_json::json!("must-not-be-persisted"))]
                    .into_iter()
                    .collect(),
            },
        )
        .await
        .expect("emit trace event")
}

#[tokio::test]
async fn trace_is_default_off_and_creates_no_database() {
    let temp = tempfile::tempdir().expect("temp app data");
    let default_trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let config = OcliveKernelConfig::new(temp.path(), roles_dir());
    assert!(config.runtime_event_trace_path().is_none());

    let kernel = build_kernel(config).await;
    let result = emit_trace_event(&kernel, "default-off").await;
    assert_eq!(result.primary.kind, TRACE_EVENT_KIND);
    let diagnostics = kernel.runtime_event_trace_diagnostics();
    assert!(!diagnostics.configured);
    assert!(!diagnostics.worker_active);
    assert_eq!(diagnostics.persisted_events, 0);
    assert!(!diagnostics.captures_payloads);
    assert!(!diagnostics.captures_metadata);
    assert!(!diagnostics.captures_stream_key);

    kernel.shutdown().await;
    assert!(!default_trace_path.exists());
}

#[tokio::test]
async fn enabled_trace_is_append_only_redacted_and_recovers_position_after_restart() {
    let temp = tempfile::tempdir().expect("temp app data");
    let config = OcliveKernelConfig::new(temp.path(), roles_dir()).with_runtime_event_trace();
    let trace_path = config
        .runtime_event_trace_path()
        .expect("configured trace path")
        .to_path_buf();
    assert_ne!(trace_path, config.database_path());

    let first_kernel = build_kernel(config.clone()).await;
    let first = emit_trace_event(&first_kernel, "trace-first").await;
    assert_eq!(first.primary.payload["label"], "trace-first");
    assert!(first.emitted.is_empty());
    let first_diagnostics = first_kernel.runtime_event_trace_diagnostics();
    assert!(first_diagnostics.configured);
    assert!(first_diagnostics.worker_active);
    assert!(!first_diagnostics.captures_payloads);
    assert!(!first_diagnostics.captures_metadata);
    assert!(!first_diagnostics.captures_stream_key);
    first_kernel.shutdown().await;

    assert!(trace_path.is_file());
    let second_kernel = build_kernel(config.clone()).await;
    assert_eq!(
        second_kernel
            .runtime_event_trace_diagnostics()
            .last_persisted_position,
        Some(1)
    );
    let second = emit_trace_event(&second_kernel, "trace-second").await;
    assert_eq!(second.primary.kind, first.primary.kind);
    assert_eq!(second.primary.source, first.primary.source);
    assert_eq!(second.primary.payload["label"], "trace-second");
    assert!(second.emitted.is_empty());
    second_kernel.shutdown().await;

    let pool = oclive_kernel_host::infrastructure::sqlite_pool::connect_file(&trace_path)
        .await
        .expect("open trace database");
    let rows = sqlx::query(
        "SELECT position, event_kind, source, correlation_id, ring_sequence \
         FROM runtime_event_trace_records ORDER BY position",
    )
    .fetch_all(&pool)
    .await
    .expect("read trace records");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get::<i64, _>("position"), 1);
    assert_eq!(rows[1].get::<i64, _>("position"), 2);
    assert_eq!(rows[0].get::<String, _>("event_kind"), TRACE_EVENT_KIND);
    assert_eq!(
        rows[0].get::<String, _>("source"),
        "module.test.runtime_trace_source"
    );
    assert_eq!(rows[0].get::<String, _>("correlation_id"), "trace-first");
    assert_eq!(rows[1].get::<String, _>("correlation_id"), "trace-second");
    assert!(rows[1].get::<i64, _>("ring_sequence") >= 1);

    let columns = sqlx::query("PRAGMA table_info(runtime_event_trace_records)")
        .fetch_all(&pool)
        .await
        .expect("read trace schema")
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();
    assert!(!columns.iter().any(|name| name == "payload"));
    assert!(!columns.iter().any(|name| name == "metadata"));
    assert!(!columns.iter().any(|name| name == "stream_key"));
    assert!(
        sqlx::query("UPDATE runtime_event_trace_records SET event_kind = 'tampered'")
            .execute(&pool)
            .await
            .is_err(),
        "trace records must reject updates"
    );
    assert!(
        sqlx::query("DELETE FROM runtime_event_trace_records")
            .execute(&pool)
            .await
            .is_err(),
        "trace records must reject deletes"
    );
    pool.close().await;
}

#[tokio::test]
async fn trace_refuses_to_share_the_main_kernel_database() {
    let temp = tempfile::tempdir().expect("temp app data");
    let base_config = OcliveKernelConfig::new(temp.path(), roles_dir());
    let main_database_path = base_config.database_path().to_path_buf();
    let config = base_config.with_runtime_event_trace_path(&main_database_path);

    let kernel = build_kernel(config).await;
    let diagnostics = kernel.runtime_event_trace_diagnostics();
    assert!(diagnostics.configured);
    assert!(!diagnostics.worker_active);
    assert!(diagnostics.failure_count >= 1);
    assert_eq!(
        diagnostics.last_error_kind,
        Some(RuntimeEventTraceErrorKind::SchemaFailed)
    );
    let result = emit_trace_event(&kernel, "same-database-fail-open").await;
    assert_eq!(result.primary.kind, TRACE_EVENT_KIND);
    kernel
        .health_check()
        .await
        .expect("main database remains healthy");
    kernel.shutdown().await;

    let pool = oclive_kernel_host::infrastructure::sqlite_pool::connect_file(&main_database_path)
        .await
        .expect("open main database");
    let trace_table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master \
         WHERE type = 'table' AND name = 'runtime_event_trace_records'",
    )
    .fetch_one(&pool)
    .await
    .expect("inspect main database schema");
    assert_eq!(trace_table_count, 0);
    pool.close().await;
}

#[tokio::test]
async fn malformed_trace_database_degrades_without_blocking_event_dispatch() {
    let temp = tempfile::tempdir().expect("temp app data");
    let trace_path = temp.path().join("malformed-runtime-event-trace.sqlite3");
    std::fs::write(&trace_path, b"not a sqlite database").expect("write malformed trace db");
    let config = OcliveKernelConfig::new(temp.path(), roles_dir())
        .with_runtime_event_trace_path(&trace_path);

    let kernel = build_kernel(config).await;
    let diagnostics = kernel.runtime_event_trace_diagnostics();
    assert!(diagnostics.configured);
    assert!(!diagnostics.worker_active);
    assert!(diagnostics.failure_count >= 1);
    assert!(matches!(
        diagnostics.last_error_kind,
        Some(RuntimeEventTraceErrorKind::OpenFailed | RuntimeEventTraceErrorKind::SchemaFailed)
    ));

    let result = emit_trace_event(&kernel, "malformed-fail-open").await;
    assert_eq!(result.primary.kind, TRACE_EVENT_KIND);
    assert_eq!(result.primary.payload["label"], "malformed-fail-open");
    kernel
        .health_check()
        .await
        .expect("main database remains healthy");
    kernel.shutdown().await;
}
