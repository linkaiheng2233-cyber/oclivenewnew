use super::*;
use async_trait::async_trait;

struct StepModule {
    declaration: EventModuleDeclaration,
    step: &'static str,
    emit: bool,
}

struct LoopModule;

#[async_trait]
impl EventModule for LoopModule {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.loop".into(),
            subscriptions: vec!["kernel.test.loop".into()],
            priority: 0,
        }
    }

    async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
        Ok(EventModuleOutput {
            emitted: vec![EventEmission {
                kind: "kernel.test.loop".into(),
                payload: Value::Null,
                metadata: BTreeMap::new(),
            }],
            ..Default::default()
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
            vec![EventEmission {
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
            priority,
        },
        step,
        emit,
    })
}

#[tokio::test]
async fn modules_transform_in_priority_then_id_order() -> Result<()> {
    let ring = EventRing::new();
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

    let result = ring
        .publish(
            "kernel.test.primary",
            "kernel.test",
            "test-stream",
            Some("correlation-1"),
            serde_json::json!({"steps": []}),
        )
        .await?;

    assert_eq!(
        result.primary.payload,
        serde_json::json!({"steps": ["early", "alpha", "beta"]})
    );
    assert_eq!(ring.recent_events(10), vec![result.primary]);
    Ok(())
}

#[tokio::test]
async fn emitted_children_reenter_the_ring_with_causation() -> Result<()> {
    let ring = EventRing::new();
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

    let result = ring
        .publish(
            "kernel.test.primary",
            "kernel.test",
            "test-stream",
            None,
            serde_json::json!({"steps": []}),
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
        .expect_err("module source prefix must fit the canonical identifier limit");

    assert!(error.contains("module event source"));
}

#[tokio::test]
async fn recursive_emission_stops_without_committing_partial_history() {
    let ring = EventRing::new();
    assert!(ring.register_event_module(Arc::new(LoopModule)).is_ok());

    let result = ring
        .publish(
            "kernel.test.loop",
            "kernel.test",
            "test-stream",
            None,
            Value::Null,
        )
        .await;

    assert!(result.is_err());
    assert!(ring.recent_events(10).is_empty());
}
