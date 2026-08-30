//! Generic in-process Event Ring data contracts.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current wire schema for [`EventEnvelope`].
pub const EVENT_RING_SCHEMA_VERSION: u16 = 2;

/// Fixed-point scale used by Event Ring influence weights (`10_000 == 1.0`).
pub const EVENT_INFLUENCE_WEIGHT_SCALE: u16 = 10_000;

/// Current wire schema for [`EventRingDiagnostics`].
pub const EVENT_RING_DIAGNOSTICS_SCHEMA_VERSION: u16 = 2;

/// One immutable-identity event travelling through the kernel Event Ring.
///
/// Modules may replace `payload`, merge `metadata`, or emit child events, but the ring keeps
/// identity, source, stream, correlation, causation, and ordering fields authoritative.
///
/// # Examples
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use oclive_kernel_types::{
///     EventEnvelope, EVENT_INFLUENCE_WEIGHT_SCALE, EVENT_RING_SCHEMA_VERSION,
/// };
///
/// let event = EventEnvelope {
///     schema_version: EVENT_RING_SCHEMA_VERSION,
///     event_id: "event-1".into(),
///     kind: "kernel.chat.message.received".into(),
///     source: "module.kernel.chat".into(),
///     source_weight_bps: EVENT_INFLUENCE_WEIGHT_SCALE,
///     stream_key: "chat:mumu".into(),
///     correlation_id: "turn-1".into(),
///     causation_id: None,
///     sequence: 1,
///     depth: 0,
///     occurred_at: Utc.timestamp_opt(0, 0).single().expect("valid timestamp"),
///     payload: serde_json::json!({"text": "hello"}),
///     metadata: Default::default(),
/// };
/// assert_eq!(event.schema_version, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub event_id: String,
    pub kind: String,
    pub source: String,
    /// Registry-owned snapshot of the source module's base proposal influence.
    pub source_weight_bps: u16,
    pub stream_key: String,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    pub sequence: u64,
    pub depth: u16,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

/// Untrusted event content proposed by a registered module.
///
/// Modules create drafts, while the Event Ring supplies authoritative identity, source, weight,
/// stream, correlation, causation, ordering, depth, and timestamp fields.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::EventDraft;
///
/// let draft = EventDraft {
///     kind: "kernel.memory.recall.candidate".into(),
///     payload: serde_json::json!({"memory_id": "memory-1"}),
///     metadata: Default::default(),
/// };
/// assert_eq!(draft.kind, "kernel.memory.recall.candidate");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDraft {
    pub kind: String,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

/// Capabilities declared by an Event Ring module.
///
/// Subscriptions and emissions are canonical dotted event kinds, a trailing namespace wildcard
/// such as `kernel.chat.*`, or `*`. Lower priorities run first; ties use `module_id` ordering.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::EventModuleDeclaration;
///
/// let declaration = EventModuleDeclaration {
///     module_id: "builtin.memory.observe".into(),
///     subscriptions: vec!["kernel.chat.*".into()],
///     emissions: vec!["kernel.memory.recall.candidate".into()],
///     priority: 100,
/// };
/// assert_eq!(declaration.subscriptions.len(), 1);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventModuleDeclaration {
    pub module_id: String,
    pub subscriptions: Vec<String>,
    pub emissions: Vec<String>,
    pub priority: i32,
}

/// Kernel-owned policy applied when a module is admitted to the Event Ring registry.
///
/// The module does not return this policy from [`EventModuleDeclaration`]. The trusted registrar
/// supplies it separately, so an event producer cannot assign its own proposal influence.
/// `influence_weight_bps` affects downstream proposal decisions only; it does not change dispatch
/// order or invocation frequency.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{EventModuleFailureMode, EventModuleRegistryPolicy};
///
/// let policy = EventModuleRegistryPolicy {
///     influence_weight_bps: 7_500,
///     failure_mode: EventModuleFailureMode::FailFast,
/// };
/// assert_eq!(policy.influence_weight_bps, 7_500);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventModuleRegistryPolicy {
    pub influence_weight_bps: u16,
    /// Registry-owned response to handler or output-validation failure.
    #[serde(default)]
    pub failure_mode: EventModuleFailureMode,
}

impl Default for EventModuleRegistryPolicy {
    fn default() -> Self {
        Self {
            influence_weight_bps: EVENT_INFLUENCE_WEIGHT_SCALE,
            failure_mode: EventModuleFailureMode::FailFast,
        }
    }
}

/// Registry-owned failure boundary for one Event Ring module.
///
/// Built-in modules default to [`Self::FailFast`]. Untrusted adapters may be admitted with
/// [`Self::Isolate`], which quarantines the failing module while allowing the current dispatch to
/// continue without applying any of that module's output.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventModuleFailureMode {
    #[default]
    FailFast,
    Isolate,
}

/// Read-only snapshot of one module admitted to the Event Ring registry.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{
///     EventModuleDeclaration, EventModuleRegistryEntry, EventModuleRegistryPolicy,
/// };
///
/// let entry = EventModuleRegistryEntry {
///     declaration: EventModuleDeclaration {
///         module_id: "builtin.memory.observe".into(),
///         emissions: vec!["kernel.memory.recall.candidate".into()],
///         ..Default::default()
///     },
///     policy: EventModuleRegistryPolicy {
///         influence_weight_bps: 8_500,
///         ..Default::default()
///     },
/// };
/// assert_eq!(entry.policy.influence_weight_bps, 8_500);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventModuleRegistryEntry {
    pub declaration: EventModuleDeclaration,
    pub policy: EventModuleRegistryPolicy,
}

/// Runtime health of one registered Event Ring module.
///
/// Failure details and event content are intentionally excluded. A quarantined module remains in
/// the registry for diagnosis but is skipped by later dispatches until the host replaces it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventModuleRuntimeDiagnostic {
    pub module_id: String,
    pub quarantined: bool,
    pub failure_count: u64,
}

/// Privacy-minimized summary of one successfully dispatched event.
///
/// Payload content, metadata values, and the stream key are intentionally omitted. Diagnostics
/// retain only routing/causation identity, payload size, and metadata key names.
///
/// # Examples
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use oclive_kernel_types::EventRingEventDiagnostic;
///
/// let event = EventRingEventDiagnostic {
///     event_id: "event-1".into(),
///     kind: "kernel.memory.recall.candidate".into(),
///     source: "module.builtin.memory_recollection".into(),
///     source_weight_bps: 8_500,
///     correlation_id: "turn-1".into(),
///     causation_id: None,
///     sequence: 1,
///     depth: 0,
///     occurred_at: Utc.timestamp_opt(0, 0).single().expect("valid timestamp"),
///     payload_bytes: 24,
///     metadata_keys: vec!["trace.kind".into()],
/// };
/// assert_eq!(event.payload_bytes, 24);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRingEventDiagnostic {
    pub event_id: String,
    pub kind: String,
    pub source: String,
    pub source_weight_bps: u16,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    pub sequence: u64,
    pub depth: u16,
    pub occurred_at: DateTime<Utc>,
    pub payload_bytes: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata_keys: Vec<String>,
}

/// Read-only Event Ring registry and bounded-history diagnostics.
///
/// A snapshot is eventually consistent when registration or dispatch happens concurrently; it
/// never locks the registry and event history at the same time.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{EventRingDiagnostics, EVENT_RING_DIAGNOSTICS_SCHEMA_VERSION};
///
/// let diagnostics = EventRingDiagnostics {
///     schema_version: EVENT_RING_DIAGNOSTICS_SCHEMA_VERSION,
///     registry: Vec::new(),
///     module_runtime: Vec::new(),
///     history_len: 0,
///     history_capacity: 256,
///     last_allocated_sequence: 0,
///     recent_events: Vec::new(),
/// };
/// assert!(diagnostics.recent_events.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRingDiagnostics {
    pub schema_version: u16,
    pub registry: Vec<EventModuleRegistryEntry>,
    /// Runtime health in the same deterministic `(priority, module_id)` order as `registry`.
    pub module_runtime: Vec<EventModuleRuntimeDiagnostic>,
    pub history_len: u64,
    pub history_capacity: u64,
    pub last_allocated_sequence: u64,
    pub recent_events: Vec<EventRingEventDiagnostic>,
}

/// Compatibility name for child-event drafts used by the initial Event Ring slice.
pub type EventEmission = EventDraft;

/// Deterministic contribution returned by one Event Ring module.
///
/// `payload` replaces the current event payload when present. Metadata is merged with later
/// modules winning the same key, and emitted children are appended to the bounded dispatch queue.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::EventModuleOutput;
///
/// let output = EventModuleOutput {
///     payload: Some(serde_json::json!({"normalized": true})),
///     ..Default::default()
/// };
/// assert!(output.payload.is_some());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EventModuleOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emitted: Vec<EventDraft>,
}

/// Completed bounded dispatch, including the transformed primary event and processed children.
///
/// # Examples
///
/// ```
/// # use chrono::{TimeZone, Utc};
/// use oclive_kernel_types::{EventDispatchResult, EventEnvelope, EVENT_RING_SCHEMA_VERSION};
/// # let primary = EventEnvelope { schema_version: EVENT_RING_SCHEMA_VERSION, event_id: "e".into(), kind: "kernel.test".into(), source: "module.kernel.test".into(), source_weight_bps: 10_000, stream_key: "test".into(), correlation_id: "c".into(), causation_id: None, sequence: 1, depth: 0, occurred_at: Utc.timestamp_opt(0, 0).single().expect("valid timestamp"), payload: serde_json::Value::Null, metadata: Default::default() };
/// let result = EventDispatchResult { primary, emitted: Vec::new() };
/// assert!(result.emitted.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDispatchResult {
    pub primary: EventEnvelope,
    pub emitted: Vec<EventEnvelope>,
}
