//! Generic in-process Event Ring data contracts.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current wire schema for [`EventEnvelope`].
pub const EVENT_RING_SCHEMA_VERSION: u16 = 1;

/// One immutable-identity event travelling through the kernel Event Ring.
///
/// Modules may replace `payload`, merge `metadata`, or emit child events, but the ring keeps
/// identity, source, stream, correlation, causation, and ordering fields authoritative.
///
/// # Examples
///
/// ```
/// use chrono::{TimeZone, Utc};
/// use oclive_kernel_types::{EventEnvelope, EVENT_RING_SCHEMA_VERSION};
///
/// let event = EventEnvelope {
///     schema_version: EVENT_RING_SCHEMA_VERSION,
///     event_id: "event-1".into(),
///     kind: "kernel.chat.message.received".into(),
///     source: "kernel.chat".into(),
///     stream_key: "chat:mumu".into(),
///     correlation_id: "turn-1".into(),
///     causation_id: None,
///     sequence: 1,
///     depth: 0,
///     occurred_at: Utc.timestamp_opt(0, 0).single().expect("valid timestamp"),
///     payload: serde_json::json!({"text": "hello"}),
///     metadata: Default::default(),
/// };
/// assert_eq!(event.schema_version, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub event_id: String,
    pub kind: String,
    pub source: String,
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

/// Declarative registration supplied by an Event Ring module.
///
/// Subscriptions are canonical dotted event kinds, a trailing namespace wildcard such as
/// `kernel.chat.*`, or `*`. Lower priorities run first; ties use `module_id` ordering.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::EventModuleDeclaration;
///
/// let declaration = EventModuleDeclaration {
///     module_id: "builtin.memory.observe".into(),
///     subscriptions: vec!["kernel.chat.*".into()],
///     priority: 100,
/// };
/// assert_eq!(declaration.subscriptions.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventModuleDeclaration {
    pub module_id: String,
    pub subscriptions: Vec<String>,
    pub priority: i32,
}

/// Child event requested by a module while handling another event.
///
/// The ring supplies source, stream, correlation, causation, sequence, depth, and timestamp.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::EventEmission;
///
/// let emission = EventEmission {
///     kind: "kernel.memory.candidate".into(),
///     payload: serde_json::json!({"importance": 0.8}),
///     metadata: Default::default(),
/// };
/// assert_eq!(emission.kind, "kernel.memory.candidate");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEmission {
    pub kind: String,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

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
    pub emitted: Vec<EventEmission>,
}

/// Completed bounded dispatch, including the transformed primary event and processed children.
///
/// # Examples
///
/// ```
/// # use chrono::{TimeZone, Utc};
/// use oclive_kernel_types::{EventDispatchResult, EventEnvelope, EVENT_RING_SCHEMA_VERSION};
/// # let primary = EventEnvelope { schema_version: EVENT_RING_SCHEMA_VERSION, event_id: "e".into(), kind: "kernel.test".into(), source: "kernel.test".into(), stream_key: "test".into(), correlation_id: "c".into(), causation_id: None, sequence: 1, depth: 0, occurred_at: Utc.timestamp_opt(0, 0).single().expect("valid timestamp"), payload: serde_json::Value::Null, metadata: Default::default() };
/// let result = EventDispatchResult { primary, emitted: Vec::new() };
/// assert!(result.emitted.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDispatchResult {
    pub primary: EventEnvelope,
    pub emitted: Vec<EventEnvelope>,
}
