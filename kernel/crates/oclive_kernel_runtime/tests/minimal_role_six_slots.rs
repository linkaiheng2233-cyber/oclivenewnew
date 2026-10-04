#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::future::poll_fn;
use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{
    AgentBase, BaseCallFuture, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase,
};
use oclive_kernel_runtime::domain::base_emotion::KeywordEmotionBase;
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_runtime::domain::minimal_role_consumer::{
    MinimalRoleBaseBindings, MinimalRoleBaseConsumer,
};
use oclive_kernel_types::{
    AgentBaseRequest, BaseCallError, BaseCallErrorKind, EmotionBaseRequest, EventBaseRequest,
    LlmBaseRequest, MemoryBaseRequest, MinimalRoleDefinition, PromptBaseRequest,
};

#[derive(Debug, PartialEq, Eq)]
enum Call {
    Memory(Vec<String>, String),
    Emotion(String, Option<String>),
    Event(String, Option<String>),
    Prompt(Vec<String>, String),
    Llm(String),
    Agent(String, Option<String>),
}

#[derive(Default)]
struct Slots {
    calls: RefCell<Vec<Call>>,
    error: Option<BaseCallError>,
    empty: bool,
}

impl Slots {
    fn result<T>(&self, value: T) -> Result<T, BaseCallError> {
        match &self.error {
            Some(error) => Err(error.clone()),
            None => Ok(value),
        }
    }

    fn bindings(&self) -> MinimalRoleBaseBindings<'_> {
        MinimalRoleBaseBindings {
            memory: self,
            emotion: self,
            event: self,
            prompt: self,
            llm: self,
            agent: self,
        }
    }
}

impl MemoryBase for Slots {
    fn retrieve<'a>(&'a self, req: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        self.calls.borrow_mut().push(Call::Memory(
            req.materials.iter().map(|s| (*s).into()).collect(),
            req.query.into(),
        ));
        Box::pin(async move { self.result(Vec::new()) })
    }
}

impl EmotionBase for Slots {
    fn analyze<'a>(&'a self, req: EmotionBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        self.calls.borrow_mut().push(Call::Emotion(
            req.material.into(),
            req.context.map(Into::into),
        ));
        Box::pin(async move {
            self.result((!self.empty && req.material.contains("开心")).then(|| {
                format!(
                    "Material contains 开心; quotation/subject not resolved: {}",
                    req.material
                )
            }))
        })
    }
}

impl EventBase for Slots {
    fn analyze<'a>(&'a self, req: EventBaseRequest<'a>) -> BaseCallFuture<'a, Option<String>> {
        self.calls.borrow_mut().push(Call::Event(
            req.material.into(),
            req.context.map(Into::into),
        ));
        Box::pin(async move {
            self.result((!self.empty).then(|| format!("Described event: {}", req.material)))
        })
    }
}

impl PromptBase for Slots {
    fn assemble<'a>(&'a self, req: PromptBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.borrow_mut().push(Call::Prompt(
            req.materials.iter().map(|s| (*s).into()).collect(),
            req.requirements.into(),
        ));
        Box::pin(async move {
            self.result(if self.empty {
                String::new()
            } else {
                req.materials.concat()
            })
        })
    }
}

impl LlmBase for Slots {
    fn generate<'a>(&'a self, req: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls.borrow_mut().push(Call::Llm(req.input.into()));
        Box::pin(async move {
            self.result(if self.empty {
                String::new()
            } else {
                req.input.into()
            })
        })
    }
}

impl AgentBase for Slots {
    fn execute<'a>(&'a self, req: AgentBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        self.calls
            .borrow_mut()
            .push(Call::Agent(req.task.into(), req.context.map(Into::into)));
        Box::pin(async move {
            self.result(if self.empty {
                String::new()
            } else {
                format!("No external action; delegated task: {}", req.task)
            })
        })
    }
}

fn role() -> MinimalRoleDefinition {
    MinimalRoleDefinition {
        persona_prompt: "A patient guide.\r\n".into(),
        visual_assets: vec!["host-owned:portrait".into()],
    }
}

// Only immediately-ready fixtures use this helper; the pending test drives polls separately.
fn ready<T>(mut future: BaseCallFuture<'_, T>) -> Result<T, BaseCallError> {
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("fixture unexpectedly suspended"),
    }
}

struct ReverseKeywordMemory;

impl MemoryBase for ReverseKeywordMemory {
    fn retrieve<'a>(&'a self, req: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        Box::pin(async move {
            let mut hits = KeywordMemoryBase.retrieve(req).await?;
            hits.reverse();
            Ok(hits)
        })
    }
}

#[test]
fn the_same_minimal_role_consumes_six_bases_with_replaced_memory_and_emotion() {
    let definition = role();
    let material = "她说：我很开心。";
    let candidates = [
        "Alice likes tea.",
        "Bob likes tea.",
        "Alice dislikes coffee.",
    ];
    for alternate in [false, true] {
        let slots = Slots::default();
        let memory: &dyn MemoryBase = if alternate {
            &ReverseKeywordMemory
        } else {
            &KeywordMemoryBase
        };
        let emotion: &dyn EmotionBase = if alternate {
            &slots
        } else {
            &KeywordEmotionBase
        };
        let consumer = MinimalRoleBaseConsumer::new(
            &definition,
            MinimalRoleBaseBindings {
                memory,
                emotion,
                ..slots.bindings()
            },
        )
        .unwrap();

        // Two caller-owned dependency graphs, not a runtime-prescribed six-stage order.
        let event_req = EventBaseRequest {
            material: "She may return tomorrow.",
            context: None,
        };
        let early_event =
            alternate.then(|| ready(EventBase::analyze(&consumer, event_req)).unwrap());
        let hits = ready(MemoryBase::retrieve(
            &consumer,
            MemoryBaseRequest {
                materials: &candidates,
                query: "tea",
            },
        ))
        .unwrap();
        assert_eq!(
            hits,
            if alternate {
                vec![candidates[1], candidates[0]]
            } else {
                vec![candidates[0], candidates[1]]
            }
        );
        let emotion_report = ready(EmotionBase::analyze(
            &consumer,
            EmotionBaseRequest {
                material,
                context: None,
            },
        ))
        .unwrap()
        .unwrap();
        if alternate {
            assert_eq!(
                emotion_report,
                format!("Material contains 开心; quotation/subject not resolved: {material}")
            );
        } else {
            assert!(emotion_report.contains("开心"));
            assert_ne!(
                emotion_report,
                format!("Material contains 开心; quotation/subject not resolved: {material}")
            );
        }
        let event_report = early_event
            .unwrap_or_else(|| ready(EventBase::analyze(&consumer, event_req)).unwrap())
            .unwrap();
        let early_task = (!alternate).then(|| {
            ready(AgentBase::execute(
                &consumer,
                AgentBaseRequest {
                    task: "Report without external action.",
                    context: Some(&event_report),
                },
            ))
            .unwrap()
        });
        let mut materials: Vec<&str> = hits.iter().map(String::as_str).collect();
        materials.extend([material, emotion_report.as_str(), event_report.as_str()]);
        if let Some(report) = &early_task {
            materials.push(report);
        }
        let prepared = ready(PromptBase::assemble(
            &consumer,
            PromptBaseRequest {
                materials: &materials,
                requirements: "",
            },
        ))
        .unwrap();
        let reply = ready(LlmBase::generate(
            &consumer,
            LlmBaseRequest { input: &prepared },
        ))
        .unwrap();
        assert_eq!(reply, prepared);
        assert_eq!(reply.matches(&definition.persona_prompt).count(), 1);
        for item in &materials {
            assert!(reply.contains(item));
        }
        if alternate {
            let report = ready(AgentBase::execute(
                &consumer,
                AgentBaseRequest {
                    task: "Report without external action.",
                    context: Some(&reply),
                },
            ))
            .unwrap();
            assert!(report.starts_with("No external action;"));
        }
        let calls = slots.calls.borrow();
        let order: Vec<&str> = calls
            .iter()
            .map(|call| match call {
                Call::Emotion(..) => "emotion",
                Call::Event(..) => "event",
                Call::Prompt(..) => "prompt",
                Call::Llm(..) => "llm",
                Call::Agent(..) => "agent",
                Call::Memory(..) => "memory",
            })
            .collect();
        assert_eq!(
            order,
            if alternate {
                vec!["event", "emotion", "prompt", "llm", "agent"]
            } else {
                vec!["event", "agent", "prompt", "llm"]
            }
        );
    }
    assert_eq!(definition, role());
}

#[test]
fn exact_requests_and_independent_calls_do_not_inject_role_state_or_schedule_other_slots() {
    let definition = role();
    let slots = Slots::default();
    let consumer = MinimalRoleBaseConsumer::new(&definition, slots.bindings()).unwrap();
    let materials = ["", " café\r\n", "repeat", "repeat"];
    ready(MemoryBase::retrieve(
        &consumer,
        MemoryBaseRequest {
            materials: &materials,
            query: " \r\n",
        },
    ))
    .unwrap();
    ready(EmotionBase::analyze(
        &consumer,
        EmotionBaseRequest {
            material: materials[1],
            context: None,
        },
    ))
    .unwrap();
    ready(EventBase::analyze(
        &consumer,
        EventBaseRequest {
            material: materials[1],
            context: Some(""),
        },
    ))
    .unwrap();
    ready(LlmBase::generate(
        &consumer,
        LlmBaseRequest {
            input: "already prepared\r\n",
        },
    ))
    .unwrap();
    ready(AgentBase::execute(
        &consumer,
        AgentBaseRequest {
            task: "Do not act.",
            context: Some("explicit caller constraint\r\n"),
        },
    ))
    .unwrap();
    assert_eq!(
        *slots.calls.borrow(),
        vec![
            Call::Memory(
                materials.iter().map(|s| (*s).into()).collect(),
                " \r\n".into()
            ),
            Call::Emotion(materials[1].into(), None),
            Call::Event(materials[1].into(), Some("".into())),
            Call::Llm("already prepared\r\n".into()),
            Call::Agent(
                "Do not act.".into(),
                Some("explicit caller constraint\r\n".into())
            ),
        ]
    );
    ready(PromptBase::assemble(
        &consumer,
        PromptBaseRequest {
            materials: &materials,
            requirements: "  quote material\r\n",
        },
    ))
    .unwrap();
    assert_eq!(slots.calls.borrow().len(), 6);
    assert_eq!(
        slots.calls.borrow().last(),
        Some(&Call::Prompt(
            vec![
                "【角色设定】\n".into(),
                definition.persona_prompt.clone(),
                "\n\n【输入材料】\n".into(),
                "".into(),
                materials[1].into(),
                "repeat".into(),
                "repeat".into()
            ],
            "  quote material\r\n".into()
        ))
    );
}

#[test]
fn normal_empty_results_and_all_failure_reasons_remain_distinguishable_without_retry() {
    let definition = role();
    for kind in [
        None,
        Some(BaseCallErrorKind::Failed),
        Some(BaseCallErrorKind::Unavailable),
        Some(BaseCallErrorKind::Unsupported),
        Some(BaseCallErrorKind::Cancelled),
        Some(BaseCallErrorKind::TimedOut),
    ] {
        let error = kind.map(|kind| BaseCallError {
            kind,
            detail: Some("provider detail\r\n".into()),
        });
        let slots = Slots {
            error: error.clone(),
            empty: true,
            ..Slots::default()
        };
        let consumer = MinimalRoleBaseConsumer::new(&definition, slots.bindings()).unwrap();
        assert_eq!(
            ready(MemoryBase::retrieve(
                &consumer,
                MemoryBaseRequest {
                    materials: &[],
                    query: ""
                }
            )),
            error.clone().map_or_else(|| Ok(vec![]), Err)
        );
        assert_eq!(
            ready(EmotionBase::analyze(
                &consumer,
                EmotionBaseRequest {
                    material: "",
                    context: None
                }
            )),
            error.clone().map_or_else(|| Ok(None), Err)
        );
        assert_eq!(
            ready(EventBase::analyze(
                &consumer,
                EventBaseRequest {
                    material: "",
                    context: None
                }
            )),
            error.clone().map_or_else(|| Ok(None), Err)
        );
        assert_eq!(
            ready(PromptBase::assemble(
                &consumer,
                PromptBaseRequest {
                    materials: &[],
                    requirements: ""
                }
            )),
            error.clone().map_or_else(|| Ok(String::new()), Err)
        );
        assert_eq!(
            ready(LlmBase::generate(&consumer, LlmBaseRequest { input: "" })),
            error.clone().map_or_else(|| Ok(String::new()), Err)
        );
        assert_eq!(
            ready(AgentBase::execute(
                &consumer,
                AgentBaseRequest {
                    task: "",
                    context: None
                }
            )),
            error.map_or_else(|| Ok(String::new()), Err)
        );
        assert_eq!(slots.calls.borrow().len(), 6);
    }
}

#[test]
fn invalid_minimal_content_is_rejected_before_any_of_the_six_capabilities_is_called() {
    let slots = Slots::default();
    for definition in [
        MinimalRoleDefinition {
            persona_prompt: " ".into(),
            ..role()
        },
        MinimalRoleDefinition {
            visual_assets: vec![],
            ..role()
        },
    ] {
        assert!(MinimalRoleBaseConsumer::new(&definition, slots.bindings()).is_err());
    }
    assert!(slots.calls.borrow().is_empty());
}

struct PendingMemory {
    released: Cell<bool>,
    calls: Cell<usize>,
}

impl MemoryBase for PendingMemory {
    fn retrieve<'a>(&'a self, req: MemoryBaseRequest<'a>) -> BaseCallFuture<'a, Vec<String>> {
        self.calls.set(self.calls.get() + 1);
        Box::pin(async move {
            poll_fn(|cx| {
                if self.released.get() {
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await;
            KeywordMemoryBase.retrieve(req).await
        })
    }
}

#[test]
fn a_local_pending_capability_keeps_borrowed_inputs_and_never_reenters_or_calls_another_slot() {
    let definition = role();
    let slots = Slots::default();
    let memory = PendingMemory {
        released: Cell::new(false),
        calls: Cell::new(0),
    };
    let consumer = MinimalRoleBaseConsumer::new(
        &definition,
        MinimalRoleBaseBindings {
            memory: &memory,
            ..slots.bindings()
        },
    )
    .unwrap();
    let owned = String::from("Local café\r\n");
    let materials = [owned.as_str()];
    let query = String::from("café");
    let mut future = MemoryBase::retrieve(
        &consumer,
        MemoryBaseRequest {
            materials: &materials,
            query: &query,
        },
    );
    let mut cx = Context::from_waker(Waker::noop());
    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert_eq!(memory.calls.get(), 1);
    assert!(slots.calls.borrow().is_empty());
    memory.released.set(true);
    assert_eq!(
        future.as_mut().poll(&mut cx),
        Poll::Ready(Ok(vec![owned.clone()]))
    );
    assert_eq!(memory.calls.get(), 1);
}
