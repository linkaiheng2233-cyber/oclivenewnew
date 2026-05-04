//! `plugin_backends.agent = none`：serde 与 `DisabledAgentProvider` 行为。

use oclive_kernel_runtime::domain::agent::{
    AgentInput, AgentProvider, DisabledAgentProvider, AGENT_BACKEND_NONE_REPLY,
};
use oclive_kernel_runtime::models::plugin_backends::AgentBackend;

#[test]
fn agent_backend_deserializes_none_snake_case() {
    let v: AgentBackend = serde_json::from_value(serde_json::json!("none")).unwrap();
    assert_eq!(v, AgentBackend::None);
}

#[tokio::test]
async fn disabled_agent_provider_returns_fixed_reply_not_empty() {
    let p = DisabledAgentProvider;
    let out = p
        .process(AgentInput {
            role_id: "r1".to_string(),
            session_namespace: "ns".to_string(),
            message: "user secret".to_string(),
            model: "m".to_string(),
        })
        .await
        .unwrap();
    assert!(!out.handled);
    assert_eq!(out.reply, AGENT_BACKEND_NONE_REPLY);
    assert!(!out.reply.trim().is_empty());
    assert!(!out.reply.contains("user secret"));
}
