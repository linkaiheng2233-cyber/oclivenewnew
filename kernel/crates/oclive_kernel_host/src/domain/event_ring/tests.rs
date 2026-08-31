use super::*;
use async_trait::async_trait;

struct StepModule {
    declaration: EventModuleDeclaration,
    step: &'static str,
    emit: bool,
}

struct LoopModule;
struct SourceModule;

#[derive(Default)]
struct RecordingTraceSink {
    dispatches: Mutex<Vec<Vec<EventEnvelope>>>,
}

impl EventDispatchTraceSink for RecordingTraceSink {
    fn record_successful_dispatch(&self, events: &[EventEnvelope]) {
        self.dispatches.lock().push(events.to_vec());
    }
}

struct InvalidIsolatedModule {
    calls: Arc<AtomicU64>,
}

#[async_trait]
impl EventModule for SourceModule {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.source".into(),
            emissions: vec!["kernel.test.*".into()],
            ..Default::default()
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

#[async_trait]
impl EventModule for LoopModule {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.loop".into(),
            subscriptions: vec!["kernel.test.loop".into()],
            emissions: vec!["kernel.test.loop".into()],
            priority: 0,
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput {
            emitted: vec![EventDraft {
                kind: "kernel.test.loop".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            }],
            ..Default::default()
        })
    }
}

#[async_trait]
impl EventModule for InvalidIsolatedModule {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "directory.test.invalid".into(),
            subscriptions: vec!["kernel.test.isolation".into()],
            emissions: vec!["plugin.test.allowed".into()],
            priority: 0,
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let mut metadata = BTreeMap::new();
        metadata.insert("plugin.test.changed".into(), Value::Bool(true));
        Ok(EventModuleOutput {
            payload: Some(serde_json::json!({"changed": true})),
            metadata,
            emitted: vec![EventDraft {
                kind: "plugin.test.forbidden".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            }],
        })
    }
}

#[async_trait]
impl EventModule for StepModule {
    fn declaration(&self) -> EventModuleDeclaration {
        self.declaration.clone()
    }

    async fn handle(&self, event: &EventEnvelope) -> Result<EventModuleOutput> {
        let mut steps = event
            .payload
            .get("steps")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        steps.push(Value::String(self.step.into()));
        let emitted = if self.emit && event.kind == "kernel.test.primary" {
            vec![EventDraft {
                kind: "kernel.test.child".into(),
                payload: serde_json::json!({"steps": []}),
                metadata: BTreeMap::new(),
            }]
        } else {
            Vec::new()
        };
        Ok(EventModuleOutput {
            payload: Some(serde_json::json!({"steps": steps})),
            metadata: BTreeMap::new(),
            emitted,
        })
    }
}

fn step_module(
    module_id: &str,
    subscription: &str,
    priority: i32,
    step: &'static str,
    emit: bool,
) -> Arc<dyn EventModule> {
    Arc::new(StepModule {
        declaration: EventModuleDeclaration {
            module_id: module_id.into(),
            subscriptions: vec![subscription.into()],
            emissions: if emit {
                vec!["kernel.test.child".into()]
            } else {
                Vec::new()
            },
            priority,
        },
        step,
        emit,
    })
}

fn source_emitter(ring: &EventRing) -> Result<Arc<dyn EventEmitter>> {
    ring.register_event_module_with_policy(
        Arc::new(SourceModule),
        EventModuleRegistryPolicy {
            influence_weight_bps: 7_000,
            ..Default::default()
        },
    )
    .map_err(AppError::InvalidParameter)
}

#[tokio::test]
async fn modules_transform_in_priority_then_id_order() -> Result<()> {
    let ring = EventRing::new();
    let emitter = source_emitter(&ring)?;
    ring.register_event_module(step_module(
        "test.beta",
        "kernel.test.*",
        200,
        "beta",
        false,
    ))
    .map_err(AppError::InvalidParameter)?;
    ring.register_event_module(step_module(
        "test.early",
        "kernel.test.primary",
        100,
        "early",
        false,
    ))
    .map_err(AppError::InvalidParameter)?;
    ring.register_event_module(step_module(
        "test.alpha",
        "kernel.test.*",
        200,
        "alpha",
        false,
    ))
    .map_err(AppError::InvalidParameter)?;

    let result = emitter
        .emit(
            "test-stream",
            Some("correlation-1"),
            EventDraft {
                kind: "kernel.test.primary".into(),
                payload: serde_json::json!({"steps": []}),
                metadata: BTreeMap::new(),
            },
        )
        .await?;

    assert_eq!(
        result.primary.payload,
        serde_json::json!({"steps": ["early", "alpha", "beta"]})
    );
    assert_eq!(result.primary.source, "module.test.source");
    assert_eq!(result.primary.source_weight_bps, 7_000);
    assert_eq!(ring.recent_events(10), vec![result.primary]);
    Ok(())
}

#[tokio::test]
async fn source_bound_emitter_rejects_undeclared_event_kind() -> Result<()> {
    let ring = EventRing::new();
    let emitter = source_emitter(&ring)?;

    let result = emitter
        .emit(
            "test-stream",
            None,
            EventDraft {
                kind: "kernel.other.event".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            },
        )
        .await;

    assert!(result.is_err());
    assert!(ring.recent_events(10).is_empty());
    Ok(())
}

#[tokio::test]
async fn emitted_children_reenter_the_ring_with_causation() -> Result<()> {
    let ring = EventRing::new();
    let emitter = source_emitter(&ring)?;
    ring.register_event_module(step_module(
        "test.emit",
        "kernel.test.primary",
        0,
        "parent",
        true,
    ))
    .map_err(AppError::InvalidParameter)?;
    ring.register_event_module(step_module(
        "test.child",
        "kernel.test.child",
        0,
        "child",
        false,
    ))
    .map_err(AppError::InvalidParameter)?;

    let result = emitter
        .emit(
            "test-stream",
            None,
            EventDraft {
                kind: "kernel.test.primary".into(),
                payload: serde_json::json!({"steps": []}),
                metadata: BTreeMap::new(),
            },
        )
        .await?;

    assert_eq!(result.emitted.len(), 1);
    let child = &result.emitted[0];
    assert_eq!(
        child.causation_id.as_deref(),
        Some(result.primary.event_id.as_str())
    );
    assert_eq!(child.correlation_id, result.primary.correlation_id);
    assert_eq!(child.payload, serde_json::json!({"steps": ["child"]}));
    Ok(())
}

#[test]
fn conflicting_module_registration_is_rejected() {
    let ring = EventRing::new();
    assert!(ring
        .register_event_module(step_module("test.same", "kernel.test", 0, "one", false))
        .is_ok());
    assert!(ring
        .register_event_module(step_module("test.same", "kernel.other", 0, "two", false))
        .is_err());
}

#[test]
fn registration_rejects_module_id_that_cannot_form_a_valid_event_source() {
    let ring = EventRing::new();
    let module_id = format!("test.{}", "a".repeat(MAX_CANONICAL_ID_LEN - 5));

    let error = ring
        .register_event_module(step_module(&module_id, "kernel.test", 0, "too-long", false))
        .err()
        .expect("module source prefix must fit the canonical identifier limit");

    assert!(error.contains("module event source"));
}

#[test]
fn registry_rejects_out_of_range_authority_weight() {
    let ring = EventRing::new();

    let error = ring
        .register_event_module_with_policy(
            step_module("test.weight", "kernel.test", 0, "weight", false),
            EventModuleRegistryPolicy {
                influence_weight_bps: EVENT_INFLUENCE_WEIGHT_SCALE + 1,
                ..Default::default()
            },
        )
        .err()
        .expect("out-of-range registry weight must be rejected");

    assert!(error.contains("event registry influence_weight_bps"));
    assert!(ring.event_module_registry().is_empty());
}

#[test]
fn registry_snapshot_exposes_authority_policy_separately_from_capabilities() -> Result<()> {
    let ring = EventRing::new();
    let _emitter = source_emitter(&ring)?;
    ring.register_event_module(step_module(
        "test.observer",
        "kernel.test.*",
        100,
        "observer",
        false,
    ))
    .map_err(AppError::InvalidParameter)?;

    let entries = ring.event_module_registry();

    assert_eq!(entries.len(), 2);
    let source = entries
        .iter()
        .find(|entry| entry.declaration.module_id == "test.source")
        .expect("source registry entry");
    assert_eq!(source.policy.influence_weight_bps, 7_000);
    let observer = entries
        .iter()
        .find(|entry| entry.declaration.module_id == "test.observer")
        .expect("observer registry entry");
    assert_eq!(
        observer.policy.influence_weight_bps,
        EVENT_INFLUENCE_WEIGHT_SCALE
    );
    assert_eq!(
        ring.event_module_declarations(),
        entries
            .into_iter()
            .map(|entry| entry.declaration)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
async fn diagnostics_snapshot_redacts_content_and_keeps_routing_evidence() -> Result<()> {
    let ring = EventRing::new();
    let emitter = source_emitter(&ring)?;
    let payload = serde_json::json!({"private_memory": "secret-memory-text"});
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "trace.label".into(),
        Value::String("secret-metadata-value".into()),
    );

    let result = emitter
        .emit(
            "private-session-stream",
            Some("turn-diagnostics"),
            EventDraft {
                kind: "kernel.test.diagnostics".into(),
                payload: payload.clone(),
                metadata,
            },
        )
        .await?;

    let diagnostics = ring.diagnostics_snapshot(1);
    assert_eq!(
        diagnostics.schema_version,
        EVENT_RING_DIAGNOSTICS_SCHEMA_VERSION
    );
    assert_eq!(diagnostics.registry.len(), 1);
    assert_eq!(diagnostics.module_runtime.len(), 1);
    assert!(!diagnostics.module_runtime[0].quarantined);
    assert_eq!(diagnostics.module_runtime[0].failure_count, 0);
    assert_eq!(diagnostics.history_len, 1);
    assert_eq!(
        diagnostics.history_capacity,
        usize_to_u64(EVENT_HISTORY_CAPACITY)
    );
    assert_eq!(diagnostics.last_allocated_sequence, result.primary.sequence);
    assert_eq!(diagnostics.recent_events.len(), 1);
    let event = &diagnostics.recent_events[0];
    assert_eq!(event.event_id, result.primary.event_id);
    assert_eq!(event.kind, "kernel.test.diagnostics");
    assert_eq!(event.correlation_id, "turn-diagnostics");
    assert_eq!(event.metadata_keys, vec!["trace.label"]);
    assert_eq!(
        event.payload_bytes,
        usize_to_u64(serde_json::to_vec(&payload)?.len())
    );
    let encoded = serde_json::to_string(&diagnostics)?;
    assert!(!encoded.contains("secret-memory-text"));
    assert!(!encoded.contains("secret-metadata-value"));
    assert!(!encoded.contains("private-session-stream"));

    let without_recent = ring.diagnostics_snapshot(0);
    assert_eq!(without_recent.history_len, 1);
    assert!(without_recent.recent_events.is_empty());
    Ok(())
}

#[tokio::test]
async fn isolate_policy_atomically_rejects_output_and_quarantines_module() -> Result<()> {
    let ring = EventRing::new();
    let emitter = source_emitter(&ring)?;
    let calls = Arc::new(AtomicU64::new(0));
    ring.register_event_module_with_policy(
        Arc::new(InvalidIsolatedModule {
            calls: Arc::clone(&calls),
        }),
        EventModuleRegistryPolicy {
            influence_weight_bps: 4_000,
            failure_mode: EventModuleFailureMode::Isolate,
        },
    )
    .map_err(AppError::InvalidParameter)?;

    for correlation_id in ["isolation-1", "isolation-2"] {
        let result = emitter
            .emit(
                "test-stream",
                Some(correlation_id),
                EventDraft {
                    kind: "kernel.test.isolation".into(),
                    payload: serde_json::json!({"original": true}),
                    metadata: BTreeMap::new(),
                },
            )
            .await?;
        assert_eq!(
            result.primary.payload,
            serde_json::json!({"original": true})
        );
        assert!(result.primary.metadata.is_empty());
        assert!(result.emitted.is_empty());
    }

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let diagnostics = ring.diagnostics_snapshot(0);
    let runtime = diagnostics
        .module_runtime
        .iter()
        .find(|entry| entry.module_id == "directory.test.invalid")
        .expect("directory module runtime diagnostic");
    assert!(runtime.quarantined);
    assert_eq!(runtime.failure_count, 1);
    let registry = diagnostics
        .registry
        .iter()
        .find(|entry| entry.declaration.module_id == "directory.test.invalid")
        .expect("directory module registry entry");
    assert_eq!(
        registry.policy.failure_mode,
        EventModuleFailureMode::Isolate
    );
    Ok(())
}

#[tokio::test]
async fn recursive_emission_stops_without_committing_partial_history() {
    let ring = EventRing::new();
    let emitter = ring
        .register_event_module(Arc::new(LoopModule))
        .expect("loop module registration");

    let result = emitter
        .emit(
            "test-stream",
            None,
            EventDraft {
                kind: "kernel.test.loop".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            },
        )
        .await;

    assert!(result.is_err());
    assert!(ring.recent_events(10).is_empty());
}

#[tokio::test]
async fn trace_sink_observes_success_without_changing_dispatch_result() -> Result<()> {
    let trace_sink = Arc::new(RecordingTraceSink::default());
    let ring = EventRing::with_trace_sink(trace_sink.clone());
    let emitter = source_emitter(&ring)?;
    ring.register_event_module(step_module(
        "test.trace_child",
        "kernel.test.primary",
        100,
        "trace-child",
        true,
    ))
    .map_err(AppError::InvalidParameter)?;

    let result = emitter
        .emit(
            "trace-stream",
            Some("trace-correlation"),
            EventDraft {
                kind: "kernel.test.primary".into(),
                payload: serde_json::json!({"steps": []}),
                metadata: BTreeMap::new(),
            },
        )
        .await?;

    let dispatches = trace_sink.dispatches.lock();
    assert_eq!(dispatches.len(), 1);
    let expected = std::iter::once(result.primary.clone())
        .chain(result.emitted.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(dispatches[0], expected);
    assert_eq!(
        result.primary.payload,
        serde_json::json!({"steps": ["trace-child"]})
    );
    assert_eq!(result.emitted.len(), 1);
    Ok(())
}

#[tokio::test]
async fn trace_sink_does_not_observe_failed_dispatch() {
    let trace_sink = Arc::new(RecordingTraceSink::default());
    let ring = EventRing::with_trace_sink(trace_sink.clone());
    let emitter = ring
        .register_event_module(Arc::new(LoopModule))
        .expect("loop module registration");

    let result = emitter
        .emit(
            "trace-stream",
            None,
            EventDraft {
                kind: "kernel.test.loop".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            },
        )
        .await;

    assert!(result.is_err());
    assert!(trace_sink.dispatches.lock().is_empty());
}
