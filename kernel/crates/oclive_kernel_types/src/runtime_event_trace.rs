//! Privacy-minimized diagnostics for the optional Runtime Event trace shadow.

use serde::{Deserialize, Serialize};

/// Schema version for [`RuntimeEventTraceDiagnostics`].
pub const RUNTIME_EVENT_TRACE_DIAGNOSTICS_SCHEMA_VERSION: u16 = 2;

/// Last failure observed by the fail-open Runtime Event trace recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventTraceErrorKind {
    OpenFailed,
    SchemaFailed,
    WriteFailed,
    QueueFull,
    WorkerClosed,
}

/// Read-only health counters for the optional Runtime Event trace shadow.
///
/// The trace is observability-only. These diagnostics intentionally expose no event payload,
/// metadata, stream key, database path, or model-facing content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEventTraceDiagnostics {
    pub schema_version: u16,
    pub configured: bool,
    /// Whether the recorder still accepts successful Ring dispatches.
    ///
    /// This becomes false before an orderly shutdown and after a terminal storage failure.
    #[serde(default)]
    pub accepting_dispatches: bool,
    pub worker_active: bool,
    pub queue_capacity: u64,
    pub enqueued_dispatches: u64,
    pub enqueued_events: u64,
    pub persisted_dispatches: u64,
    pub persisted_events: u64,
    pub duplicate_events: u64,
    /// Dispatch batches accepted by the queue but not persisted after a terminal write failure.
    #[serde(default)]
    pub failed_dispatches: u64,
    /// Events belonging to failed dispatch batches.
    #[serde(default)]
    pub failed_events: u64,
    pub dropped_dispatches: u64,
    pub dropped_events: u64,
    pub failure_count: u64,
    pub last_persisted_position: Option<u64>,
    pub last_error_kind: Option<RuntimeEventTraceErrorKind>,
    pub captures_payloads: bool,
    pub captures_metadata: bool,
    pub captures_stream_key: bool,
}

impl Default for RuntimeEventTraceDiagnostics {
    fn default() -> Self {
        Self {
            schema_version: RUNTIME_EVENT_TRACE_DIAGNOSTICS_SCHEMA_VERSION,
            configured: false,
            accepting_dispatches: false,
            worker_active: false,
            queue_capacity: 0,
            enqueued_dispatches: 0,
            enqueued_events: 0,
            persisted_dispatches: 0,
            persisted_events: 0,
            duplicate_events: 0,
            failed_dispatches: 0,
            failed_events: 0,
            dropped_dispatches: 0,
            dropped_events: 0,
            failure_count: 0,
            last_persisted_position: None,
            last_error_kind: None,
            captures_payloads: false,
            captures_metadata: false,
            captures_stream_key: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_v1_without_lifecycle_counters_remains_readable() {
        let value = serde_json::json!({
            "schema_version": 1,
            "configured": true,
            "worker_active": true,
            "queue_capacity": 256,
            "enqueued_dispatches": 1,
            "enqueued_events": 1,
            "persisted_dispatches": 1,
            "persisted_events": 1,
            "duplicate_events": 0,
            "dropped_dispatches": 0,
            "dropped_events": 0,
            "failure_count": 0,
            "last_persisted_position": 1,
            "last_error_kind": null,
            "captures_payloads": false,
            "captures_metadata": false,
            "captures_stream_key": false
        });

        let diagnostics: RuntimeEventTraceDiagnostics =
            serde_json::from_value(value).expect("deserialize diagnostics v1");
        assert!(!diagnostics.accepting_dispatches);
        assert_eq!(diagnostics.failed_dispatches, 0);
        assert_eq!(diagnostics.failed_events, 0);
    }
}
