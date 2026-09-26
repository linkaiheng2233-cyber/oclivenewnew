//! An external Host can select a few Base capabilities without adopting the reference Host.
//!
//! This intentionally uses immediate, in-memory implementations. A real Host must drive
//! BaseCallFuture with its own executor and decide resource permissions, domain effects,
//! fallback and persistence separately. Nothing here defines a mandatory slot order.

use std::task::{Context, Poll, Waker};

use oclive_kernel_contracts::{BaseCallFuture, LlmBase, MemoryBase, PromptBase};
use oclive_kernel_runtime::domain::base_memory::KeywordMemoryBase;
use oclive_kernel_runtime::domain::base_prompt::LiteralMaterialAssembler;
use oclive_kernel_types::{BaseCallError, LlmBaseRequest, MemoryBaseRequest, PromptBaseRequest};

// The two reference adapters and the local LLM below complete on their first poll.
// This helper is for this example only, not a general-purpose executor.
fn poll_immediate<T>(mut future: BaseCallFuture<'_, T>) -> Result<T, BaseCallError> {
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("this example requires immediate in-memory adapters"),
    }
}

struct EchoLlm;

impl LlmBase for EchoLlm {
    fn generate<'a>(&'a self, request: LlmBaseRequest<'a>) -> BaseCallFuture<'a, String> {
        Box::pin(async move { Ok(format!("echo: {}", request.input)) })
    }
}

struct SmallHost<'a> {
    memory: Option<&'a dyn MemoryBase>,
    prompt: &'a dyn PromptBase,
    llm: &'a dyn LlmBase,
}

impl SmallHost<'_> {
    fn reply(
        &self,
        candidates: &[&str],
        retrieval_need: &str,
        user_text: &str,
    ) -> Result<String, BaseCallError> {
        let selected = match self.memory {
            Some(memory) => poll_immediate(memory.retrieve(MemoryBaseRequest {
                materials: candidates,
                query: retrieval_need,
            }))?,
            None => Vec::new(),
        };

        // This formatting is the Host's choice; the Base contracts prescribe no role,
        // prompt template, state store, or six-stage pipeline.
        let mut fragments: Vec<&str> = Vec::new();
        for item in &selected {
            fragments.push(item);
            fragments.push("\n");
        }
        fragments.extend(["User: ", user_text]);
        let prepared = poll_immediate(self.prompt.assemble(PromptBaseRequest {
            materials: &fragments,
            requirements: "",
        }))?;
        poll_immediate(self.llm.generate(LlmBaseRequest { input: &prepared }))
    }
}

fn main() -> Result<(), BaseCallError> {
    let prompt = LiteralMaterialAssembler;
    let llm = EchoLlm;
    let memory = KeywordMemoryBase;
    let candidates = ["Alice did not drink coffee", "Bob asked about tea"];

    let with_memory = SmallHost {
        memory: Some(&memory),
        prompt: &prompt,
        llm: &llm,
    }
    .reply(&candidates, "coffee", "What happened?")?;
    assert_eq!(
        with_memory,
        "echo: Alice did not drink coffee\nUser: What happened?"
    );

    let without_memory = SmallHost {
        memory: None,
        prompt: &prompt,
        llm: &llm,
    }
    .reply(&candidates, "coffee", "What happened?")?;
    assert_eq!(without_memory, "echo: User: What happened?");
    println!("base-first host: optional Memory + Prompt + LLM passed");
    Ok(())
}
