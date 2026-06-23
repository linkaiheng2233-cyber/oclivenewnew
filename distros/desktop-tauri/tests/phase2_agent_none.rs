//! Agent noop backend (agent = none).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::domain::agent::{AgentInput, AgentProvider};
use oclive_kernel_host::domain::noop_slot_backends::NoopAgentProvider;

#[tokio::test]
async fn noop_agent_returns_unhandled() {
    let p = NoopAgentProvider;
    let out = p
        .process(AgentInput {
            role_id: "r".into(),
            session_namespace: "r".into(),
            message: "hi".into(),
            model: "m".into(),
            ..Default::default()
        })
        .await
        .expect("ok");
    assert!(!out.handled);
    assert!(out.reply.is_empty());
}
