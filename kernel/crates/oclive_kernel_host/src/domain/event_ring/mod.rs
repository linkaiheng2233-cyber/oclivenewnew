//! Kernel-owned, in-memory Event Ring and declarative module registry.
//!
//! The ring is generic infrastructure around the kernel. It provides deterministic, bounded
//! event transformation and child emission without turning the legacy dialogue `event` slot into
//! a second orchestration authority.

mod legacy_event_impact;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use oclive_kernel_contracts::{EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    AppError, EventDispatchResult, EventEmission, EventEnvelope, EventModuleDeclaration,
    EventModuleOutput, Result, EVENT_RING_SCHEMA_VERSION,
};
use parking_lot::{Mutex, RwLock};
use serde_json::Value;
use uuid::Uuid;

pub use legacy_event_impact::{
    publish_legacy_event_impact, LEGACY_EVENT_IMPACT_KIND, LEGACY_EVENT_IMPACT_SOURCE,
};

const EVENT_HISTORY_CAPACITY: usize = 256;
const MAX_EVENTS_PER_DISPATCH: usize = 64;
const MAX_EMISSIONS_PER_MODULE: usize = 16;
const MAX_EVENT_DEPTH: u16 = 8;
const MAX_EVENT_PAYLOAD_BYTES: usize = 256 * 1024;
const MAX_METADATA_ENTRIES: usize = 64;
const MAX_CANONICAL_ID_LEN: usize = 160;
const MAX_STREAM_KEY_LEN: usize = 256;
const MIN_MODULE_PRIORITY: i32 = -10_000;
const MAX_MODULE_PRIORITY: i32 = 10_000;

#[derive(Clone)]
struct RegisteredModule {
    declaration: EventModuleDeclaration,
    module: Arc<dyn EventModule>,
}

/// Per-kernel Event Ring with a bounded ephemeral history and module registry.
///
/// The ring owns no database writer and does not expose a distribution-specific protocol. A
/// caller publishes one generic envelope; matching modules run sequentially by `(priority,
/// module_id)`, then child events traverse the same bounded queue.
pub struct EventRing {
    modules: RwLock<BTreeMap<String, RegisteredModule>>,
    history: Mutex<VecDeque<EventEnvelope>>,
    sequence: AtomicU64,
}

impl Default for EventRing {
    fn default() -> Self {
        Self {
            modules: RwLock::new(BTreeMap::new()),
            history: Mutex::new(VecDeque::with_capacity(EVENT_HISTORY_CAPACITY)),
            sequence: AtomicU64::new(0),
        }
    }
}

impl EventRing {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publishes and fully dispatches one primary event.
    ///
    /// When no modules are registered, the returned primary payload is unchanged. The ring still
    /// assigns identity and records the envelope in its bounded in-memory history.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid identifiers, oversized payloads, module failures, or bounded
    /// dispatch limits. No history entries are committed for a failed dispatch.
    pub async fn publish(
        &self,
        kind: &str,
        source: &str,
        stream_key: &str,
        correlation_id: Option<&str>,
        payload: Value,
    ) -> Result<EventDispatchResult> {
        validate_event_kind(kind)?;
        validate_canonical_id(source, "event source")?;
        validate_stream_key(stream_key)?;
        validate_payload(&payload)?;
        if let Some(correlation_id) = correlation_id {
            validate_opaque_id(correlation_id, "correlation_id")?;
        }

        let event_id = Uuid::new_v4().to_string();
        let primary = EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: event_id.clone(),
            kind: kind.to_string(),
            source: source.to_string(),
            stream_key: stream_key.to_string(),
            correlation_id: correlation_id.unwrap_or(event_id.as_str()).to_string(),
            causation_id: None,
            sequence: self.next_sequence(),
            depth: 0,
            occurred_at: Utc::now(),
            payload,
            metadata: BTreeMap::new(),
        };
        self.dispatch(primary).await
    }

    /// Returns the newest `limit` successfully dispatched events in chronological order.
    #[must_use]
    pub fn recent_events(&self, limit: usize) -> Vec<EventEnvelope> {
        let history = self.history.lock();
        let limit = limit.min(EVENT_HISTORY_CAPACITY);
        let skip = history.len().saturating_sub(limit);
        history.iter().skip(skip).cloned().collect()
    }

    async fn dispatch(&self, primary: EventEnvelope) -> Result<EventDispatchResult> {
        validate_envelope(&primary)?;
        let mut queue = VecDeque::from([primary]);
        let mut processed = Vec::new();

        while let Some(mut event) = queue.pop_front() {
            if processed.len() >= MAX_EVENTS_PER_DISPATCH {
                return Err(event_ring_error("dispatch event limit exceeded"));
            }
            let modules = self.matching_modules(&event.kind);
            for registered in modules {
                let output = registered.module.handle(&event).await?;
                apply_module_output(
                    self,
                    &registered.declaration.module_id,
                    &mut event,
                    output,
                    &mut queue,
                    processed.len(),
                )?;
            }
            processed.push(event);
        }

        let Some(primary) = processed.first().cloned() else {
            return Err(event_ring_error("dispatch produced no primary event"));
        };
        self.record_history(&processed);
        let emitted = processed.into_iter().skip(1).collect();
        Ok(EventDispatchResult { primary, emitted })
    }

    fn matching_modules(&self, kind: &str) -> Vec<RegisteredModule> {
        let mut modules = self
            .modules
            .read()
            .values()
            .filter(|registered| {
                registered
                    .declaration
                    .subscriptions
                    .iter()
                    .any(|pattern| subscription_matches(pattern, kind))
            })
            .cloned()
            .collect::<Vec<_>>();
        modules.sort_by(|left, right| {
            left.declaration
                .priority
                .cmp(&right.declaration.priority)
                .then_with(|| left.declaration.module_id.cmp(&right.declaration.module_id))
        });
        modules
    }

    fn child_envelope(
        &self,
        module_id: &str,
        parent: &EventEnvelope,
        emission: EventEmission,
    ) -> Result<EventEnvelope> {
        validate_event_kind(&emission.kind)?;
        validate_payload(&emission.payload)?;
        validate_metadata(&emission.metadata)?;
        let depth = parent
            .depth
            .checked_add(1)
            .ok_or_else(|| event_ring_error("event depth overflow"))?;
        if depth > MAX_EVENT_DEPTH {
            return Err(event_ring_error("event depth limit exceeded"));
        }
        Ok(EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: Uuid::new_v4().to_string(),
            kind: emission.kind,
            source: format!("module.{module_id}"),
            stream_key: parent.stream_key.clone(),
            correlation_id: parent.correlation_id.clone(),
            causation_id: Some(parent.event_id.clone()),
            sequence: self.next_sequence(),
            depth,
            occurred_at: Utc::now(),
            payload: emission.payload,
            metadata: emission.metadata,
        })
    }

    fn next_sequence(&self) -> u64 {
        self.sequence
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }

    fn record_history(&self, events: &[EventEnvelope]) {
        let mut history = self.history.lock();
        for event in events {
            if history.len() == EVENT_HISTORY_CAPACITY {
                history.pop_front();
            }
            history.push_back(event.clone());
        }
    }
}

impl EventModuleRegistrar for EventRing {
    fn register_event_module(
        &self,
        module: Arc<dyn EventModule>,
    ) -> std::result::Result<(), String> {
        let declaration = module.declaration();
        validate_declaration(&declaration)?;
        let mut modules = self.modules.write();
        if let Some(existing) = modules.get(&declaration.module_id) {
            if existing.declaration == declaration && Arc::ptr_eq(&existing.module, &module) {
                return Ok(());
            }
            return Err(format!(
                "event module {} already registered with a different declaration or implementation",
                declaration.module_id
            ));
        }
        modules.insert(
            declaration.module_id.clone(),
            RegisteredModule {
                declaration,
                module,
            },
        );
        Ok(())
    }

    fn event_module_declarations(&self) -> Vec<EventModuleDeclaration> {
        let mut declarations = self
            .modules
            .read()
            .values()
            .map(|registered| registered.declaration.clone())
            .collect::<Vec<_>>();
        declarations.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.module_id.cmp(&right.module_id))
        });
        declarations
    }
}

fn apply_module_output(
    ring: &EventRing,
    module_id: &str,
    event: &mut EventEnvelope,
    output: EventModuleOutput,
    queue: &mut VecDeque<EventEnvelope>,
    processed_count: usize,
) -> Result<()> {
    if output.emitted.len() > MAX_EMISSIONS_PER_MODULE {
        return Err(event_ring_error("module emission limit exceeded"));
    }
    if let Some(payload) = output.payload {
        validate_payload(&payload)?;
        event.payload = payload;
    }
    let mut merged_metadata = event.metadata.clone();
    merged_metadata.extend(output.metadata);
    validate_metadata(&merged_metadata)?;
    event.metadata = merged_metadata;

    if processed_count
        .saturating_add(queue.len())
        .saturating_add(output.emitted.len())
        >= MAX_EVENTS_PER_DISPATCH
    {
        return Err(event_ring_error("dispatch event limit exceeded"));
    }
    for emission in output.emitted {
        queue.push_back(ring.child_envelope(module_id, event, emission)?);
    }
    Ok(())
}

fn validate_declaration(declaration: &EventModuleDeclaration) -> std::result::Result<(), String> {
    validate_canonical_id_text(&declaration.module_id, "module_id")?;
    validate_canonical_id_text(
        &format!("module.{}", declaration.module_id),
        "module event source",
    )?;
    if declaration.subscriptions.is_empty() {
        return Err("event module subscriptions cannot be empty".into());
    }
    if !(MIN_MODULE_PRIORITY..=MAX_MODULE_PRIORITY).contains(&declaration.priority) {
        return Err(format!(
            "event module priority must be between {MIN_MODULE_PRIORITY} and {MAX_MODULE_PRIORITY}"
        ));
    }
    let mut unique = BTreeSet::new();
    for pattern in &declaration.subscriptions {
        validate_subscription(pattern)?;
        if !unique.insert(pattern) {
            return Err(format!("duplicate event subscription: {pattern}"));
        }
    }
    Ok(())
}

fn validate_envelope(event: &EventEnvelope) -> Result<()> {
    if event.schema_version != EVENT_RING_SCHEMA_VERSION {
        return Err(event_ring_error("unsupported envelope schema version"));
    }
    validate_opaque_id(&event.event_id, "event_id")?;
    validate_event_kind(&event.kind)?;
    validate_canonical_id(&event.source, "event source")?;
    validate_stream_key(&event.stream_key)?;
    validate_opaque_id(&event.correlation_id, "correlation_id")?;
    if let Some(causation_id) = event.causation_id.as_deref() {
        validate_opaque_id(causation_id, "causation_id")?;
    }
    if event.depth > MAX_EVENT_DEPTH {
        return Err(event_ring_error("event depth limit exceeded"));
    }
    validate_payload(&event.payload)?;
    validate_metadata(&event.metadata)
}

fn validate_event_kind(kind: &str) -> Result<()> {
    validate_canonical_id(kind, "event kind")
}

fn validate_canonical_id(value: &str, label: &str) -> Result<()> {
    validate_canonical_id_text(value, label).map_err(event_ring_error)
}

fn validate_canonical_id_text(value: &str, label: &str) -> std::result::Result<(), String> {
    if value.is_empty() || value != value.trim() || value.len() > MAX_CANONICAL_ID_LEN {
        return Err(format!(
            "{label} must be non-empty, trimmed, and at most {MAX_CANONICAL_ID_LEN} bytes"
        ));
    }
    if value.split('.').any(|segment| {
        segment.is_empty()
            || !segment.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
    }) {
        return Err(format!(
            "{label} must use lowercase dotted identifier segments"
        ));
    }
    Ok(())
}

fn validate_subscription(pattern: &str) -> std::result::Result<(), String> {
    if pattern == "*" {
        return Ok(());
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return validate_canonical_id_text(prefix, "event subscription prefix");
    }
    validate_canonical_id_text(pattern, "event subscription")
}

fn subscription_matches(pattern: &str, kind: &str) -> bool {
    pattern == "*"
        || pattern == kind
        || pattern
            .strip_suffix(".*")
            .is_some_and(|prefix| kind.starts_with(&format!("{prefix}.")))
}

fn validate_stream_key(stream_key: &str) -> Result<()> {
    if stream_key.is_empty()
        || stream_key != stream_key.trim()
        || stream_key.len() > MAX_STREAM_KEY_LEN
    {
        return Err(event_ring_error(format!(
            "stream_key must be non-empty, trimmed, and at most {MAX_STREAM_KEY_LEN} bytes"
        )));
    }
    Ok(())
}

fn validate_opaque_id(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value != value.trim() || value.len() > MAX_CANONICAL_ID_LEN {
        return Err(event_ring_error(format!(
            "{label} must be non-empty, trimmed, and at most {MAX_CANONICAL_ID_LEN} bytes"
        )));
    }
    Ok(())
}

fn validate_payload(payload: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(payload)?;
    if bytes.len() > MAX_EVENT_PAYLOAD_BYTES {
        return Err(event_ring_error("event payload exceeds size limit"));
    }
    Ok(())
}

fn validate_metadata(metadata: &BTreeMap<String, Value>) -> Result<()> {
    if metadata.len() > MAX_METADATA_ENTRIES {
        return Err(event_ring_error("event metadata entry limit exceeded"));
    }
    for key in metadata.keys() {
        validate_canonical_id(key, "metadata key")?;
    }
    validate_payload(&serde_json::to_value(metadata)?)
}

fn event_ring_error(message: impl Into<String>) -> AppError {
    AppError::InvalidParameter(format!("event_ring: {}", message.into()))
}

#[cfg(test)]
mod tests;
