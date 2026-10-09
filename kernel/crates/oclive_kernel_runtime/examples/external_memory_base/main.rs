//! A separately compiled example crate implements Memory using only public imports.
//!
//! Run from the repository root:
//! `cargo run --locked -p oclive_kernel_runtime --example external_memory_base`
//! The implementation and its literal-query agreement belong to this caller, not
//! to the Base contract. Only Memory is called; no reference Host or rich Role is used.

mod memory;

use std::cell::Cell;
use std::task::{Context, Poll, Waker};

use memory::RecentLiteralMemory;
use oclive_kernel_contracts::{BaseCallFuture, LlmBase, MemoryBase};
use oclive_kernel_runtime::domain::base_agent::ScalarCountAgent;
use oclive_kernel_runtime::domain::base_emotion::KeywordEmotionBase;
use oclive_kernel_runtime::domain::base_event::LlmEventAnalyzer;
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_runtime::domain::base_prompt::LiteralMaterialAssembler;
use oclive_kernel_runtime::domain::minimal_role_consumer::{
    MinimalRoleBaseBindings, MinimalRoleBaseConsumer,
};
use oclive_kernel_types::{
    BaseCallError, BaseCallErrorKind, LlmBaseRequest, MemoryBaseRequest, MinimalRoleDefinition,
};

#[derive(Default)]
struct NoGeneration {
    calls: Cell<usize>,
}

impl LlmBase for NoGeneration {
    fn generate<'a>(&'a self, _request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move {
            self.calls.set(self.calls.get() + 1);
            Err(failure(
                BaseCallErrorKind::Unavailable,
                "This Memory example authorizes no generation.",
            ))
        })
    }
}

// Both selected Memory implementations are immediate. This is deliberately not
// a general executor: a real asynchronous provider needs the caller's executor.
fn failure(kind: BaseCallErrorKind, detail: &str) -> BaseCallError {
    BaseCallError {
        kind,
        detail: Some(detail.into()),
    }
}

fn immediate<T>(mut future: BaseCallFuture<'_, T>) -> Result<T, BaseCallError> {
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => Err(failure(
            BaseCallErrorKind::Failed,
            "This example only drives immediate in-memory implementations.",
        )),
    }
}

fn definition() -> MinimalRoleDefinition {
    MinimalRoleDefinition {
        persona_prompt: "A careful guide; preserve the subject and uncertainty of quotations."
            .into(),
        visual_assets: vec!["caller-owned:portrait".into()],
    }
}

fn retrieve(
    memory: &dyn MemoryBase,
    request: MemoryBaseRequest<'_>,
    llm: &NoGeneration,
) -> Result<Vec<String>, BaseCallError> {
    let role = definition();
    let emotion = KeywordEmotionBase;
    let event = LlmEventAnalyzer::new(llm);
    let prompt = LiteralMaterialAssembler;
    let agent = ScalarCountAgent;
    let consumer = MinimalRoleBaseConsumer::new(
        &role,
        MinimalRoleBaseBindings {
            memory,
            emotion: &emotion,
            event: &event,
            prompt: &prompt,
            llm,
            agent: &agent,
        },
    )
    .map_err(|diagnostics| failure(BaseCallErrorKind::Failed, &diagnostics.join("; ")))?;
    // No private runtime imports, direct bypass, slot sequence or implicit fallback.
    immediate(MemoryBase::retrieve(&consumer, request))
}

fn main() -> Result<(), BaseCallError> {
    let memory = RecentLiteralMemory::new(2);
    let llm = NoGeneration::default();
    let materials = [
        "旧：Alice 没喝咖啡。",
        "Bob 在等茶。",
        "新：Carol 引述“有人喝咖啡”，未核实。\r\n",
        "刚才：Dave 仍未喝咖啡。",
    ];
    let selected = retrieve(
        &memory,
        MemoryBaseRequest {
            materials: &materials,
            query: "literal:咖啡",
        },
        &llm,
    )?;
    let baseline = retrieve(
        &KeywordMemoryBase,
        MemoryBaseRequest {
            materials: &materials,
            query: "咖啡",
        },
        &llm,
    )?;
    if selected != materials[2..] || baseline.len() != 3 || llm.calls.get() != 0 {
        return Err(failure(
            BaseCallErrorKind::Failed,
            "The consumer did not preserve the selected Memory behavior.",
        ));
    }
    println!(
        "external memory: selected={} baseline={} memory_calls={} llm_calls={}",
        selected.len(),
        baseline.len(),
        memory.calls(),
        llm.calls.get()
    );
    for material in selected {
        println!("{material}");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn public_consumer_calls_the_external_selection_without_rewriting_material() {
        let memory = RecentLiteralMemory::new(2);
        let llm = NoGeneration::default();
        let materials = [
            "旧：A 没喝咖啡。",
            "B 喝茶。",
            "C 引述咖啡。\r\n",
            "D 未喝咖啡。",
        ];
        let selected = retrieve(
            &memory,
            MemoryBaseRequest {
                materials: &materials,
                query: "literal:咖啡",
            },
            &llm,
        )
        .unwrap();
        let baseline = retrieve(
            &KeywordMemoryBase,
            MemoryBaseRequest {
                materials: &materials,
                query: "咖啡",
            },
            &llm,
        )
        .unwrap();
        assert_eq!(selected, materials[2..]);
        assert_eq!(baseline, [materials[0], materials[2], materials[3]]);
        assert_eq!(memory.calls(), 1);
        assert_eq!(llm.calls.get(), 0);
    }

    #[test]
    fn an_empty_selection_is_success_but_unsupported_requirements_keep_the_full_error() {
        let memory = RecentLiteralMemory::new(2);
        let llm = NoGeneration::default();
        let materials = ["Alice did not drink coffee"];
        assert_eq!(
            retrieve(
                &memory,
                MemoryBaseRequest {
                    materials: &materials,
                    query: "literal:tea",
                },
                &llm
            ),
            Ok(Vec::new())
        );
        let error = retrieve(
            &memory,
            MemoryBaseRequest {
                materials: &materials,
                query: "Summarize every person's history",
            },
            &llm,
        );
        assert_eq!(
            error,
            Err(BaseCallError {
                kind: BaseCallErrorKind::Unsupported,
                detail: Some("RecentLiteralMemory requires literal:<exact substring>.".into()),
            })
        );
        assert_eq!(memory.calls(), 2); // The consumer did not retry or fall back.
        assert_eq!(llm.calls.get(), 0);
    }

    #[test]
    fn only_the_current_call_material_is_in_scope_and_the_caller_controls_the_window() {
        let memory = RecentLiteralMemory::new(1);
        let llm = NoGeneration::default();
        let prior = ["prior coffee", "newer coffee"];
        assert_eq!(
            retrieve(
                &memory,
                MemoryBaseRequest {
                    materials: &prior,
                    query: "literal:coffee",
                },
                &llm
            )
            .unwrap(),
            ["newer coffee"]
        );
        assert_eq!(
            retrieve(
                &memory,
                MemoryBaseRequest {
                    materials: &["another caller's tea"],
                    query: "literal:coffee",
                },
                &llm
            ),
            Ok(Vec::new())
        );
        assert_eq!(
            retrieve(
                &memory,
                MemoryBaseRequest {
                    materials: &prior,
                    query: "literal:",
                },
                &llm
            )
            .unwrap(),
            ["newer coffee"]
        ); // Empty literal matches all supplied items.
        assert_eq!(memory.calls(), 3);
        assert_eq!(llm.calls.get(), 0);
    }
}
