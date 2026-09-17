//! B2-C1(R1) public-surface check for the concrete Ollama Base adapter.
//!
//! This target exists because the Host crate sets `[lib] doctest = false`, so the workspace doctest
//! run cannot compile the example in
//! `oclive_kernel_host::infrastructure::base_llm::OllamaBaseAdapter`. It proves exactly two things:
//! the public type can be imported and constructed from outside the crate with a concrete client,
//! a model given as `&str` and as `String`, and `None` / `Some` options; and it can be used through
//! `&dyn LlmBase` with a caller-side borrowed `input`.
//!
//! Nothing here polls or awaits the returned future, and nothing calls `health_check`: the future
//! is dropped immediately, so no request is sent. That also means this target proves nothing about
//! network reachability, completion, cancellation or timing.
//!
//! The public surface is intentionally just the type and its constructor plus the `LlmBase` impl;
//! anything else (bound model, bound options) is private state and is covered by unit tests.

use std::sync::Arc;

use oclive_kernel_contracts::{LlmBase, LlmGenerateOpts};
use oclive_kernel_host::infrastructure::base_llm::OllamaBaseAdapter;
use oclive_kernel_host::infrastructure::ollama_client::OllamaClient;
use oclive_kernel_types::LlmBaseRequest;

#[test]
fn b2_c1_public_adapter_is_constructible_and_usable_through_dyn() {
    let client = Arc::new(OllamaClient::new("http://127.0.0.1:1"));
    let without_options = OllamaBaseAdapter::new(Arc::clone(&client), "qwen2.5:7b", None);
    let with_options = OllamaBaseAdapter::new(
        client,
        "qwen2.5:7b".to_string(),
        Some(LlmGenerateOpts::interactive()),
    );

    // Caller-side borrowed input, used through the Base trait object.
    let input = format!("prepared input {}", 1 + 1);
    let slot: &dyn LlmBase = &without_options;
    let future = slot.generate(LlmBaseRequest {
        input: input.as_str(),
    });
    // Dropped without polling: this target must not send anything.
    drop(future);

    let typed: &dyn LlmBase = &with_options;
    let _ = typed;
}
