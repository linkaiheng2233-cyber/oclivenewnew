//! New and legacy paths must name the same Base definitions, not copies.

use std::cell::Cell;
use std::task::{Context, Poll, Waker};

use oclive_kernel_base as base;
use oclive_kernel_contracts as legacy;
use oclive_kernel_types as types;

struct ExternalMemory(Cell<usize>);

impl base::MemoryBase for ExternalMemory {
    fn retrieve<'a>(
        &'a self,
        request: base::MemoryBaseRequest<'a>,
    ) -> base::BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move {
            self.0.set(self.0.get() + 1);
            Ok(request
                .materials
                .iter()
                .map(|text| (*text).into())
                .collect())
        })
    }
}

#[test]
fn six_trait_objects_coerce_in_both_directions() {
    let _: fn(&dyn base::MemoryBase) -> &dyn legacy::MemoryBase = |slot| slot;
    let _: fn(&dyn legacy::MemoryBase) -> &dyn base::MemoryBase = |slot| slot;
    let _: fn(&dyn base::EmotionBase) -> &dyn legacy::EmotionBase = |slot| slot;
    let _: fn(&dyn legacy::EmotionBase) -> &dyn base::EmotionBase = |slot| slot;
    let _: fn(&dyn base::EventBase) -> &dyn legacy::EventBase = |slot| slot;
    let _: fn(&dyn legacy::EventBase) -> &dyn base::EventBase = |slot| slot;
    let _: fn(&dyn base::PromptBase) -> &dyn legacy::PromptBase = |slot| slot;
    let _: fn(&dyn legacy::PromptBase) -> &dyn base::PromptBase = |slot| slot;
    let _: fn(&dyn base::LlmBase) -> &dyn legacy::LlmBase = |slot| slot;
    let _: fn(&dyn legacy::LlmBase) -> &dyn base::LlmBase = |slot| slot;
    let _: fn(&dyn base::AgentBase) -> &dyn legacy::AgentBase = |slot| slot;
    let _: fn(&dyn legacy::AgentBase) -> &dyn base::AgentBase = |slot| slot;
}

#[test]
fn request_paths_and_call_failure_are_identical() {
    let materials = ["Alice did not leave."];
    let memory: base::MemoryBaseRequest<'_> = types::slot_base::MemoryBaseRequest {
        materials: &materials,
        query: "Alice",
    };
    let _: types::MemoryBaseRequest<'_> = memory;
    let emotion: base::EmotionBaseRequest<'_> = types::EmotionBaseRequest {
        material: "quoted joy",
        context: None,
    };
    let _: types::slot_base::EmotionBaseRequest<'_> = emotion;
    let event: base::EventBaseRequest<'_> = types::EventBaseRequest {
        material: "may leave",
        context: None,
    };
    let _: types::slot_base::EventBaseRequest<'_> = event;
    let prompt: base::PromptBaseRequest<'_> = types::PromptBaseRequest {
        materials: &materials,
        requirements: "preserve negation",
    };
    let _: types::slot_base::PromptBaseRequest<'_> = prompt;
    let llm: base::LlmBaseRequest<'_> = types::LlmBaseRequest { input: "prepared" };
    let _: types::slot_base::LlmBaseRequest<'_> = llm;
    let agent: base::AgentBaseRequest<'_> = types::AgentBaseRequest {
        task: "report",
        context: None,
    };
    let _: types::slot_base::AgentBaseRequest<'_> = agent;
    let failure: types::BaseCallError = base::BaseCallError {
        kind: base::BaseCallErrorKind::Unavailable,
        detail: Some("original detail".into()),
    };
    let failure: base::BaseCallError = failure;
    assert_eq!(failure.kind, types::BaseCallErrorKind::Unavailable);
    assert_eq!(failure.detail.as_deref(), Some("original detail"));
}

#[test]
fn an_external_implementation_is_called_through_legacy_trait_and_future() {
    let external = ExternalMemory(Cell::new(0));
    let legacy_slot: &dyn legacy::MemoryBase = &external;
    let materials = ["Alice did not leave."];
    let mut call: legacy::BaseCallFuture<'_, Vec<String>> =
        legacy_slot.retrieve(types::MemoryBaseRequest {
            materials: &materials,
            query: "Alice",
        });
    let mut context = Context::from_waker(Waker::noop());
    assert!(
        matches!(call.as_mut().poll(&mut context), Poll::Ready(Ok(result)) if result == materials)
    );
    assert_eq!(external.0.get(), 1);
}

#[test]
fn minimal_role_type_and_validation_keep_all_legacy_paths() {
    let role = base::MinimalRoleDefinition {
        persona_prompt: "A careful guide.".into(),
        visual_assets: vec!["caller-owned:portrait".into()],
    };
    let old_validation: oclive_validation::minimal_role::MinimalRoleDefinition = role;
    let old_types: types::MinimalRoleDefinition = old_validation;
    let role: base::MinimalRoleDefinition = old_types;
    assert!(oclive_validation::validate_minimal_role_definition(&role).is_ok());
    let parsed: base::MinimalRoleDefinition =
        match oclive_validation::minimal_role::parse_minimal_role_definition(
            r#"{"persona_prompt":"A careful guide.","visual_assets":["caller-owned:portrait"]}"#,
        ) {
            Ok(role) => role,
            Err(errors) => panic!("unexpected validation error: {errors:?}"),
        };
    assert_eq!(parsed, role);
}
