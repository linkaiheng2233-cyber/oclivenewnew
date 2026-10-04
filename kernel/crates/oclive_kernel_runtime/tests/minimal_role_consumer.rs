#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::future::{poll_fn, Future};
use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{BaseCallFuture, LlmBase, PromptBase};
use oclive_kernel_runtime::domain::minimal_role_consumer::{
    MinimalRolePromptConsumer, MinimalRoleTextConsumer, MinimalRoleTextError,
};
use oclive_kernel_runtime::domain::minimal_role_prompt::MinimalRolePrompt;
use oclive_kernel_types::{
    BaseCallError, BaseCallErrorKind, LlmBaseRequest, MinimalRoleDefinition, PromptBaseRequest,
};

#[derive(Debug, PartialEq, Eq)]
struct Call {
    materials: Vec<String>,
    requirements: String,
}

impl Call {
    fn capture(request: PromptBaseRequest<'_>) -> Self {
        Self {
            materials: request
                .materials
                .iter()
                .map(|text| (*text).into())
                .collect(),
            requirements: request.requirements.into(),
        }
    }
}

struct RecordingPrompt {
    calls: RefCell<Vec<Call>>,
    outcome: Result<String, BaseCallError>,
}

impl RecordingPrompt {
    fn new(outcome: Result<String, BaseCallError>) -> Self {
        Self {
            calls: RefCell::default(),
            outcome,
        }
    }
}

impl PromptBase for RecordingPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.borrow_mut().push(Call::capture(request));
        Box::pin(async move { self.outcome.clone() })
    }
}

fn definition() -> MinimalRoleDefinition {
    MinimalRoleDefinition {
        persona_prompt: "  A guide.\r\n".into(),
        visual_assets: vec!["host-owned:portrait".into()],
    }
}

fn complete(mut future: BaseCallFuture<'_, String>) -> Result<String, BaseCallError> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("this fixture must complete without waiting"),
    }
}

#[test]
fn host_selected_prompt_receives_role_materials_and_requirements_without_loss() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MinimalRolePrompt<'_>>();

    let role = definition();
    let prompt = RecordingPrompt::new(Ok("  selected reply\r\n".into()));
    let consumer = MinimalRolePromptConsumer::new(&role, &prompt).unwrap();
    let owned_text = String::from("User: café\r\n");
    let materials = ["", owned_text.as_str(), "repeat", "repeat", ""];
    let requirements = String::from("  Preserve the subject.\r\n");

    let result = complete(consumer.assemble(PromptBaseRequest {
        materials: &materials,
        requirements: &requirements,
    }));

    assert_eq!(result.unwrap(), "  selected reply\r\n");
    assert_eq!(
        *prompt.calls.borrow(),
        vec![Call {
            materials: vec![
                "【角色设定】\n".into(),
                "  A guide.\r\n".into(),
                "\n\n【输入材料】\n".into(),
                "".into(),
                "User: café\r\n".into(),
                "repeat".into(),
                "repeat".into(),
                "".into(),
            ],
            requirements,
        }]
    );
}

#[test]
fn normal_empty_text_and_complete_errors_pass_through_without_retry() {
    let role = definition();
    let materials = ["Hello."];
    let mut outcomes = vec![Ok(String::new())];
    for kind in [
        BaseCallErrorKind::Failed,
        BaseCallErrorKind::Unavailable,
        BaseCallErrorKind::Unsupported,
        BaseCallErrorKind::Cancelled,
        BaseCallErrorKind::TimedOut,
    ] {
        outcomes.push(Err(BaseCallError {
            kind,
            detail: Some("provider-owned detail\r\n".into()),
        }));
    }
    for outcome in outcomes {
        let prompt = RecordingPrompt::new(outcome.clone());
        let consumer = MinimalRolePromptConsumer::new(&role, &prompt).unwrap();
        assert_eq!(
            complete(consumer.assemble(PromptBaseRequest {
                materials: &materials,
                requirements: " ",
            })),
            outcome
        );
        assert_eq!(prompt.calls.borrow().len(), 1);
        assert_eq!(prompt.calls.borrow()[0].requirements, " ");
    }

    // The legacy implementation retains its narrower agreement and exact error.
    let legacy = MinimalRolePrompt::new(&role).unwrap();
    assert_eq!(
        complete(legacy.assemble(PromptBaseRequest {
            materials: &materials,
            requirements: " ",
        })),
        Err(BaseCallError {
            kind: BaseCallErrorKind::Unsupported,
            detail: Some("minimal role prompt does not process additional requirements".into()),
        })
    );
}

#[test]
fn invalid_role_content_is_rejected_without_invoking_a_capability() {
    let prompt = RecordingPrompt::new(Ok("must not be called".into()));
    for role in [
        MinimalRoleDefinition {
            persona_prompt: " ".into(),
            visual_assets: vec!["portrait".into()],
        },
        MinimalRoleDefinition {
            persona_prompt: "Guide.".into(),
            visual_assets: vec![],
        },
    ] {
        assert!(MinimalRolePromptConsumer::new(&role, &prompt).is_err());
    }
    assert!(prompt.calls.borrow().is_empty());
}

struct PendingPrompt {
    calls: Cell<usize>,
    released: Cell<bool>,
    waker: RefCell<Option<Waker>>,
    observations: RefCell<Vec<Call>>,
}

impl PromptBase for PendingPrompt {
    fn assemble<'a>(&'a self, request: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.set(self.calls.get() + 1);
        Box::pin(async move {
            self.observations.borrow_mut().push(Call::capture(request));
            poll_fn(|context| {
                if self.released.get() {
                    Poll::Ready(())
                } else {
                    *self.waker.borrow_mut() = Some(context.waker().clone());
                    Poll::Pending
                }
            })
            .await;
            self.observations.borrow_mut().push(Call::capture(request));
            Ok(request.materials[3].into())
        })
    }
}

#[test]
fn pending_call_keeps_borrowed_materials_and_does_not_reinvoke_the_provider() {
    let role = definition();
    let prompt = PendingPrompt {
        calls: Cell::new(0),
        released: Cell::new(false),
        waker: RefCell::default(),
        observations: RefCell::default(),
    };
    let consumer = MinimalRolePromptConsumer::new(&role, &prompt).unwrap();
    let owned_text = String::from("Local borrowed text.\r\n");
    let owned_requirements = String::from("Keep this explicit requirement.");
    let materials = [owned_text.as_str()];
    let mut future = consumer.assemble(PromptBaseRequest {
        materials: &materials,
        requirements: &owned_requirements,
    });
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);

    assert!(future.as_mut().poll(&mut context).is_pending());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(prompt.calls.get(), 1);
    prompt.released.set(true);
    prompt.waker.borrow_mut().take().unwrap().wake();
    assert_eq!(
        future.as_mut().poll(&mut context),
        Poll::Ready(Ok(owned_text.clone()))
    );
    assert_eq!(prompt.calls.get(), 1);
    let observations = prompt.observations.borrow();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0], observations[1]);
    assert_eq!(observations[1].materials[1], "  A guide.\r\n");
    assert_eq!(observations[1].materials[3], owned_text);
    assert_eq!(observations[1].requirements, owned_requirements);
}

struct RecordingLlm {
    calls: RefCell<Vec<String>>,
    outcome: Result<String, BaseCallError>,
}

impl RecordingLlm {
    fn new(outcome: Result<String, BaseCallError>) -> Self {
        Self {
            calls: RefCell::default(),
            outcome,
        }
    }
}

impl LlmBase for RecordingLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.borrow_mut().push(request.input.into());
        Box::pin(async move { self.outcome.clone() })
    }
}

fn complete_text(
    future: impl std::future::Future<Output = Result<String, MinimalRoleTextError>>,
) -> Result<String, MinimalRoleTextError> {
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("this fixture must complete without waiting"),
    }
}

#[test]
fn shared_text_call_uses_the_bound_bases_once_and_preserves_each_boundary() {
    let role = definition();
    let prompt = RecordingPrompt::new(Ok("  prepared model input\r\n".into()));
    let llm = RecordingLlm::new(Ok("  model response\r\n".into()));
    let consumer = MinimalRoleTextConsumer::new(&role, &prompt, &llm).unwrap();
    let materials = ["User: café\r\n", "", "repeated", "repeated"];
    let response = complete_text(consumer.generate(PromptBaseRequest {
        materials: &materials,
        requirements: "  Preserve the subject.\r\n",
    }));

    assert_eq!(response.unwrap(), "  model response\r\n");
    let calls = prompt.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].materials,
        [
            "【角色设定】\n",
            "  A guide.\r\n",
            "\n\n【输入材料】\n",
            "User: café\r\n",
            "",
            "repeated",
            "repeated",
        ]
    );
    assert_eq!(calls[0].requirements, "  Preserve the subject.\r\n");
    assert_eq!(*llm.calls.borrow(), ["  prepared model input\r\n"]);
}

#[test]
fn shared_text_errors_preserve_the_phase_and_complete_provider_reason_without_retry() {
    let role = definition();
    for kind in [
        BaseCallErrorKind::Failed,
        BaseCallErrorKind::Unavailable,
        BaseCallErrorKind::Unsupported,
        BaseCallErrorKind::Cancelled,
        BaseCallErrorKind::TimedOut,
    ] {
        let reason = BaseCallError {
            kind,
            detail: Some("provider-owned detail\r\n".into()),
        };
        let prompt = RecordingPrompt::new(Err(reason.clone()));
        let llm = RecordingLlm::new(Ok("must not generate".into()));
        let consumer = MinimalRoleTextConsumer::new(&role, &prompt, &llm).unwrap();
        assert_eq!(
            complete_text(consumer.generate(PromptBaseRequest {
                materials: &["Hello."],
                requirements: "Keep this requirement.",
            })),
            Err(MinimalRoleTextError::Prompt(reason.clone()))
        );
        assert_eq!(prompt.calls.borrow().len(), 1);
        assert!(llm.calls.borrow().is_empty());

        let prompt = RecordingPrompt::new(Ok("prepared".into()));
        let llm = RecordingLlm::new(Err(reason.clone()));
        let consumer = MinimalRoleTextConsumer::new(&role, &prompt, &llm).unwrap();
        assert_eq!(
            complete_text(consumer.generate(PromptBaseRequest {
                materials: &["Hello."],
                requirements: "Keep this requirement.",
            })),
            Err(MinimalRoleTextError::Llm(reason))
        );
        assert_eq!(prompt.calls.borrow().len(), 1);
        assert_eq!(*llm.calls.borrow(), ["prepared"]);
    }
}

#[test]
fn shared_text_validates_content_but_does_not_rewrite_normal_empty_results() {
    let prompt = RecordingPrompt::new(Ok(String::new()));
    let llm = RecordingLlm::new(Ok(String::new()));
    for invalid in [
        MinimalRoleDefinition {
            persona_prompt: " ".into(),
            visual_assets: vec!["portrait".into()],
        },
        MinimalRoleDefinition {
            persona_prompt: "Guide.".into(),
            visual_assets: vec![],
        },
    ] {
        assert!(MinimalRoleTextConsumer::new(&invalid, &prompt, &llm).is_err());
    }
    assert!(prompt.calls.borrow().is_empty());
    assert!(llm.calls.borrow().is_empty());

    let role = definition();
    let consumer = MinimalRoleTextConsumer::new(&role, &prompt, &llm).unwrap();
    assert_eq!(
        complete_text(consumer.generate(PromptBaseRequest {
            materials: &[],
            requirements: " ",
        })),
        Ok(String::new())
    );
    assert_eq!(prompt.calls.borrow().len(), 1);
    assert_eq!(prompt.calls.borrow()[0].requirements, " ");
    assert_eq!(*llm.calls.borrow(), [""]);
}

struct PendingLlm {
    calls: Cell<usize>,
    released: Cell<bool>,
    waker: RefCell<Option<Waker>>,
    observations: RefCell<Vec<String>>,
}

impl LlmBase for PendingLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.set(self.calls.get() + 1);
        Box::pin(async move {
            self.observations.borrow_mut().push(request.input.into());
            poll_fn(|context| {
                if self.released.get() {
                    Poll::Ready(())
                } else {
                    *self.waker.borrow_mut() = Some(context.waker().clone());
                    Poll::Pending
                }
            })
            .await;
            self.observations.borrow_mut().push(request.input.into());
            Ok(request.input.into())
        })
    }
}

#[test]
fn shared_text_waits_for_each_local_base_and_keeps_model_input_alive() {
    let role = definition();
    let prompt = PendingPrompt {
        calls: Cell::new(0),
        released: Cell::new(false),
        waker: RefCell::default(),
        observations: RefCell::default(),
    };
    let llm = PendingLlm {
        calls: Cell::new(0),
        released: Cell::new(false),
        waker: RefCell::default(),
        observations: RefCell::default(),
    };
    let consumer = MinimalRoleTextConsumer::new(&role, &prompt, &llm).unwrap();
    let owned_text = String::from("Local model input.\r\n");
    let materials = [owned_text.as_str()];
    let mut future = Box::pin(consumer.generate(PromptBaseRequest {
        materials: &materials,
        requirements: "Keep the subject.",
    }));
    let mut context = Context::from_waker(Waker::noop());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(prompt.calls.get(), 1);
    assert_eq!(llm.calls.get(), 0);
    prompt.released.set(true);
    prompt.waker.borrow_mut().take().unwrap().wake();
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(prompt.calls.get(), 1);
    assert_eq!(llm.calls.get(), 1);
    assert_eq!(
        llm.observations.borrow().as_slice(),
        std::slice::from_ref(&owned_text)
    );
    llm.released.set(true);
    llm.waker.borrow_mut().take().unwrap().wake();
    assert_eq!(
        future.as_mut().poll(&mut context),
        Poll::Ready(Ok(owned_text.clone()))
    );
    assert_eq!(prompt.calls.get(), 1);
    assert_eq!(llm.calls.get(), 1);
    assert_eq!(
        *llm.observations.borrow(),
        [owned_text.clone(), owned_text.clone()]
    );
    assert_eq!(prompt.observations.borrow().len(), 2);
}
