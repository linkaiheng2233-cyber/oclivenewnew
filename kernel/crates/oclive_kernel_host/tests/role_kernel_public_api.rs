#![allow(clippy::expect_used)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use oclive_kernel_contracts::{EventModule, EventModuleRegistrar, LlmTokenSink};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::{OcliveKernel, OcliveKernelConfig};
use oclive_kernel_types::models::dto::{SendMessageRequest, TurnOrigin};
use oclive_kernel_types::{
    EventEnvelope, EventModuleDeclaration, EventModuleOutput, EventModuleRegistryPolicy,
    ProactiveSignalKind, ProactiveTurnProposal, PROACTIVE_TURN_PROPOSED_EVENT_KIND,
};

struct EmbeddedSensorSource;

#[async_trait]
impl EventModule for EmbeddedSensorSource {
    fn declaration(&self) -> EventModuleDeclaration {
        EventModuleDeclaration {
            module_id: "test.embedded_sensor".into(),
            emissions: vec![PROACTIVE_TURN_PROPOSED_EVENT_KIND.into()],
            ..Default::default()
        }
    }

    async fn handle(
        &self,
        _event: &EventEnvelope,
    ) -> oclive_kernel_types::Result<EventModuleOutput> {
        Ok(EventModuleOutput::default())
    }
}

fn roles_dir() -> std::path::PathBuf {
    oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("distros/chat-pro/roles")
}

#[tokio::test]
async fn stable_facade_runs_user_stream_and_event_authorized_turns() {
    let temp = tempfile::tempdir().expect("temp app data");
    let config = OcliveKernelConfig::new(temp.path(), roles_dir());
    let kernel = OcliveKernel::builder(config.clone())
        .with_host_profile(HostProfile::default())
        .with_llm_client(Arc::new(MockLlmClient {
            reply: "stable facade reply".into(),
        }))
        .build()
        .await
        .expect("build role kernel");

    assert_eq!(kernel.config(), &config);
    assert!(kernel
        .list_roles()
        .await
        .expect("list roles")
        .iter()
        .any(|role| role.id == "mumu"));
    assert_eq!(
        kernel.load_role("mumu").await.expect("load role").role_id,
        "mumu"
    );

    let request = SendMessageRequest {
        role_id: "mumu".into(),
        user_message: "你好".into(),
        session_id: Some("stable-facade".into()),
        ..Default::default()
    };
    let response = kernel
        .process_message(&request)
        .await
        .expect("ordinary turn");
    assert_eq!(response.reply, "stable facade reply");

    let streamed = Arc::new(Mutex::new(String::new()));
    let stream_target = Arc::clone(&streamed);
    let sink: LlmTokenSink = Arc::new(move |token| {
        stream_target.lock().expect("stream lock").push_str(token);
    });
    let stream_response = kernel
        .process_message_stream(&request, sink)
        .await
        .expect("stream turn");
    assert_eq!(stream_response.reply, "stable facade reply");
    assert_eq!(
        streamed.lock().expect("stream output").as_str(),
        "stable facade reply"
    );

    let emitter = kernel
        .register_event_module_with_policy(
            Arc::new(EmbeddedSensorSource),
            EventModuleRegistryPolicy {
                influence_weight_bps: 9_000,
                ..Default::default()
            },
        )
        .expect("register sensor source");
    let permit = kernel
        .propose_proactive_turn(
            emitter.as_ref(),
            "role:mumu:stable-facade",
            "stable-facade-proactive-1",
            ProactiveTurnProposal {
                role_id: "mumu".into(),
                session_id: Some("stable-facade".into()),
                scene_id: Some("default".into()),
                origin: TurnOrigin::Sensor,
                signal_kind: ProactiveSignalKind::SensorObservation,
                observation: "carrier_state=held".into(),
                confidence_bps: 9_000,
                urgency_bps: 8_000,
            },
        )
        .await
        .expect("propose proactive turn")
        .expect("proposal admitted");
    let proactive = kernel
        .process_proactive_turn(permit)
        .await
        .expect("proactive turn");
    assert!(!proactive.reply.trim().is_empty());
    assert!(proactive.user_message_id.is_none());
    assert!(proactive.assistant_message_id.is_none());

    let diagnostics = kernel.event_ring_diagnostics(8);
    assert!(diagnostics.registry.iter().any(|entry| {
        entry.declaration.module_id == "test.embedded_sensor"
            && entry.policy.influence_weight_bps == 9_000
    }));
    assert!(diagnostics
        .recent_events
        .iter()
        .any(|event| { event.correlation_id == "stable-facade-proactive-1" }));

    kernel.shutdown().await;
    assert!(config.database_path().is_file());

    let reopened = OcliveKernel::builder(config)
        .with_host_profile(HostProfile::default())
        .with_llm_client(Arc::new(MockLlmClient {
            reply: "reopened".into(),
        }))
        .build()
        .await
        .expect("reopen role kernel");
    assert_eq!(
        reopened
            .role_info("mumu", Some("stable-facade"))
            .await
            .expect("persisted role info")
            .role_id,
        "mumu"
    );
    reopened.shutdown().await;
}
