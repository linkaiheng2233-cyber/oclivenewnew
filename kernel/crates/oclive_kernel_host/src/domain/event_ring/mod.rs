//! Kernel-owned, in-memory Event Ring and declarative module registry.
//!
//! The ring is generic infrastructure around the kernel. It provides deterministic, bounded
//! event transformation and child emission without turning the legacy dialogue `event` slot into
//! a second orchestration authority.

mod legacy_event_impact;
mod memory_recollection;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use oclive_kernel_contracts::{EventEmitter, EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    AppError, EventDispatchResult, EventDraft, EventEnvelope, EventModuleDeclaration,
    EventModuleOutput, Result, EVENT_INFLUENCE_WEIGHT_SCALE, EVENT_RING_SCHEMA_VERSION,
};
use parking_lot::{Mutex, RwLock};
use serde_json::Value;
use uuid::Uuid;

pub(crate) use legacy_event_impact::register_legacy_event_impact_source;
pub use legacy_event_impact::{
    publish_legacy_event_impact, LEGACY_EVENT_IMPACT_KIND, LEGACY_EVENT_IMPACT_MODULE_ID,
};
pub(crate) use memory_recollection::{
    propose_memory_recollection, recollection_prompt_body, register_memory_recollection_modules,
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
/// registered module submits a draft through its source-bound emitter; the ring signs the
/// envelope, then matching modules run sequentially by `(priority, module_id)` and child drafts
/// traverse the same bounded queue.
#[derive(Clone)]
pub struct EventRing {
    modules: Arc<RwLock<BTreeMap<String, RegisteredModule>>>,
    history: Arc<Mutex<VecDeque<EventEnvelope>>>,
    sequence: Arc<AtomicU64>,
}

struct BoundEventEmitter {
    ring: EventRing,
    module_id: String,
}

impl Default for EventRing {
    fn default() -> Self {
        Self {
            modules: Arc::new(RwLock::new(BTreeMap::new())),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(EVENT_HISTORY_CAPACITY))),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl EventRing {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    async fn emit_from_registered(
        &self,
        module_id: &str,
        stream_key: &str,
        correlation_id: Option<&str>,
        draft: EventDraft,
    ) -> Result<EventDispatchResult> {
        let declaration = self.registered_declaration(module_id)?;
        validate_declared_emission(&declaration, &draft.kind)?;
        validate_event_draft(&draft)?;
        validate_stream_key(stream_key)?;
        if let Some(correlation_id) = correlation_id {
            validate_opaque_id(correlation_id, "correlation_id")?;
        }

        let event_id = Uuid::new_v4().to_string();
        let primary = EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: event_id.clone(),
            kind: draft.kind,
            source: module_event_source(module_id),
            source_weight_bps: declaration.influence_weight_bps,
            stream_key: stream_key.to_string(),
            correlation_id: correlation_id.unwrap_or(event_id.as_str()).to_string(),
            causation_id: None,
            sequence: self.next_sequence(),
            depth: 0,
            occurred_at: Utc::now(),
            payload: draft.payload,
            metadata: draft.metadata,
        };
        self.dispatch(primary).await
    }

    fn registered_declaration(&self, module_id: &str) -> Result<EventModuleDeclaration> {
        self.modules
            .read()
            .get(module_id)
            .map(|registered| registered.declaration.clone())
            .ok_or_else(|| event_ring_error(format!("event module {module_id} is not registered")))
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
                    &registered.declaration,
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
        declaration: &EventModuleDeclaration,
        parent: &EventEnvelope,
        draft: EventDraft,
    ) -> Result<EventEnvelope> {
        validate_declared_emission(declaration, &draft.kind)?;
        validate_event_draft(&draft)?;
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
            kind: draft.kind,
            source: module_event_source(&declaration.module_id),
            source_weight_bps: declaration.influence_weight_bps,
            stream_key: parent.stream_key.clone(),
            correlation_id: parent.correlation_id.clone(),
            causation_id: Some(parent.event_id.clone()),
            sequence: self.next_sequence(),
            depth,
            occurred_at: Utc::now(),
            payload: draft.payload,
            metadata: draft.metadata,
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
    ) -> std::result::Result<Arc<dyn EventEmitter>, String> {
        let declaration = module.declaration();
        validate_declaration(&declaration)?;
        let module_id = declaration.module_id.clone();
        let mut modules = self.modules.write();
        if let Some(existing) = modules.get(&declaration.module_id) {
            if existing.declaration == declaration && Arc::ptr_eq(&existing.module, &module) {
                return Ok(Arc::new(BoundEventEmitter {
                    ring: self.clone(),
                    module_id,
                }));
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
        Ok(Arc::new(BoundEventEmitter {
            ring: self.clone(),
            module_id,
        }))
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

#[async_trait]
impl EventEmitter for BoundEventEmitter {
    async fn emit(
        &self,
        stream_key: &str,
        correlation_id: Option<&str>,
        draft: EventDraft,
    ) -> Result<EventDispatchResult> {
        self.ring
            .emit_from_registered(self.module_id.as_str(), stream_key, correlation_id, draft)
            .await
    }
}

fn apply_module_output(
    ring: &EventRing,
    declaration: &EventModuleDeclaration,
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
    for draft in output.emitted {
        queue.push_back(ring.child_envelope(declaration, event, draft)?);
    }
    Ok(())
}

fn validate_declaration(declaration: &EventModuleDeclaration) -> std::result::Result<(), String> {
    validate_canonical_id_text(&declaration.module_id, "module_id")?;
    validate_canonical_id_text(
        &module_event_source(&declaration.module_id),
        "module event source",
    )?;
    if declaration.subscriptions.is_empty() && declaration.emissions.is_empty() {
        return Err("event module must declare at least one subscription or emission".into());
    }
    if !(MIN_MODULE_PRIORITY..=MAX_MODULE_PRIORITY).contains(&declaration.priority) {
        return Err(format!(
            "event module priority must be between {MIN_MODULE_PRIORITY} and {MAX_MODULE_PRIORITY}"
        ));
    }
    if declaration.influence_weight_bps > EVENT_INFLUENCE_WEIGHT_SCALE {
        return Err(format!(
            "event module influence_weight_bps must be at most {EVENT_INFLUENCE_WEIGHT_SCALE}"
        ));
    }
    let mut unique = BTreeSet::new();
    for pattern in &declaration.subscriptions {
        validate_subscription(pattern)?;
        if !unique.insert(pattern) {
            return Err(format!("duplicate event subscription: {pattern}"));
        }
    }
    unique.clear();
    for pattern in &declaration.emissions {
        validate_event_pattern(pattern, "event emission")?;
        if !unique.insert(pattern) {
            return Err(format!("duplicate event emission: {pattern}"));
        }
    }
    Ok(())
}

fn validate_declared_emission(declaration: &EventModuleDeclaration, kind: &str) -> Result<()> {
    if declaration
        .emissions
        .iter()
        .any(|pattern| event_pattern_matches(pattern, kind))
    {
        Ok(())
    } else {
        Err(event_ring_error(format!(
            "module {} is not allowed to emit {kind}",
            declaration.module_id
        )))
    }
}

fn validate_event_draft(draft: &EventDraft) -> Result<()> {
    validate_event_kind(&draft.kind)?;
    validate_payload(&draft.payload)?;
    validate_metadata(&draft.metadata)
}

fn validate_envelope(event: &EventEnvelope) -> Result<()> {
    if event.schema_version != EVENT_RING_SCHEMA_VERSION {
        return Err(event_ring_error("unsupported envelope schema version"));
    }
    validate_opaque_id(&event.event_id, "event_id")?;
    validate_event_kind(&event.kind)?;
    validate_canonical_id(&event.source, "event source")?;
    if event.source_weight_bps > EVENT_INFLUENCE_WEIGHT_SCALE {
        return Err(event_ring_error("event source weight exceeds scale"));
    }
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
    validate_event_pattern(pattern, "event subscription")
}

fn validate_event_pattern(pattern: &str, label: &str) -> std::result::Result<(), String> {
    if pattern == "*" {
        return Ok(());
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return validate_canonical_id_text(prefix, &format!("{label} prefix"));
    }
    validate_canonical_id_text(pattern, label)
}

fn subscription_matches(pattern: &str, kind: &str) -> bool {
    event_pattern_matches(pattern, kind)
}

fn event_pattern_matches(pattern: &str, kind: &str) -> bool {
    pattern == "*"
        || pattern == kind
        || pattern
            .strip_suffix(".*")
            .is_some_and(|prefix| kind.starts_with(&format!("{prefix}.")))
}

fn module_event_source(module_id: &str) -> String {
    format!("module.{module_id}")
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
