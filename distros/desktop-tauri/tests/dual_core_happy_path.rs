#![cfg(feature = "dual_core")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::domain::chat_engine::plugin_resolve::resolve_plugins_for_session;
use oclive_kernel_host::domain::chat_engine::process_message_stream;
use oclive_kernel_host::domain::chat_engine::turn_context::TurnContext;
use oclive_kernel_host::domain::chat_engine::turn_prefetch::build_turn_prefetch;
use oclive_kernel_host::domain::dual_pipeline::DualPipelineRunner;
use oclive_kernel_host::domain::process_message;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::service::role::session_namespace;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

fn dual_core_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/oocp-test-suite/fixtures")
}

#[tokio::test]
async fn dual_pipeline_run_experimental_happy_path_returns_ok() {
    let llm = Arc::new(MockLlmClient {
        reply: "dual-core-happy-path".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dual_core_fixture_root())
        .await
        .expect("state");
    let role = state
        .load_role_cached_async("dual-core-success")
        .await
        .expect("load role");

    let req = SendMessageRequest {
        role_id: "dual-core-success".to_string(),
        user_message: "dual-core success integration".to_string(),
        scene_id: Some("default".to_string()),
        session_id: Some("dual-core-success-session".to_string()),
        ..Default::default()
    };
    let mrid = req.role_id.as_str();
    let srid = "dual-core-success-session";
    let scene_id = "default".to_string();
    state
        .db_manager
        .ensure_role_runtime(srid)
        .await
        .expect("ensure_role_runtime");
    state
        .db_manager
        .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref(), None)
        .await
        .expect("ensure_interaction_mode_seeded");
    state
        .db_manager
        .set_user_presence_scene(srid, scene_id.as_str())
        .await
        .expect("set_user_presence_scene");

    let session_config = state.effective_session_config_for(role.as_ref(), srid);
    let effective_backends = state.effective_plugin_backends_for_session(role.as_ref(), srid);
    let pl = resolve_plugins_for_session(
        state.plugin_host_port(),
        role.as_ref(),
        Some(srid),
        &effective_backends,
        state
            .effective_slot_registry_for_session(role.as_ref(), srid)
            .as_ref(),
    );
    let runtime_snapshot = state
        .db_manager
        .preflight_turn_runtime(srid, scene_id.as_str(), false)
        .await
        .expect("runtime snapshot");
    let prefetch = build_turn_prefetch(&state, role.as_ref(), srid, scene_id.as_str(), false)
        .await
        .expect("prefetch");
    let scenes = Arc::clone(&role.scene_ids);
    let role_arc = Arc::clone(&role);
    let role_for_turn = Arc::clone(&role_arc);

    let turn = TurnContext {
        state: &state,
        req: &req,
        role: role_for_turn.as_ref(),
        scene_id: scene_id.as_str(),
        scenes,
        mrid,
        srid,
        t0: Instant::now(),
        preflight_ms: 0,
        session_config,
        effective_backends,
        pl,
        immersive: false,
        character_scene_id: None,
        virtual_time_ms: 0,
        dual_core_degraded: false,
        runtime_snapshot,
        role_arc,
        prefetch,
    };

    let res = DualPipelineRunner::run_experimental(&turn)
        .await
        .expect("experimental happy path");
    assert_eq!(res.reply, "dual-core-happy-path");
    assert_eq!(res.scene_id, "default");
}

#[tokio::test]
async fn expert_lora_selection_drives_stable_completion_through_directory_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: "default-llm-should-not-win".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dual_core_fixture_root())
        .await
        .expect("state");
    let req = SendMessageRequest {
        role_id: "dual-core-lora".to_string(),
        user_message: "activate-lora".to_string(),
        scene_id: Some("default".to_string()),
        session_id: Some("dual-core-lora-session".to_string()),
        ..Default::default()
    };

    let response = process_message(&state, &req).await.expect("LoRA turn");

    assert_eq!(response.reply, "lora-adapter-selected");
    let srid = session_namespace("dual-core-lora", Some("dual-core-lora-session"));
    assert_eq!(
        state.session_cache.expert_lora_plugin_id(&srid).as_deref(),
        Some("com.oclive.test.lora")
    );

    let (token_tx, mut token_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let stream_request = SendMessageRequest {
        user_message: "continue-with-session-lora".to_string(),
        ..req
    };
    let streamed = process_message_stream(
        &state,
        &stream_request,
        Arc::new(move |token| {
            let _ = token_tx.send(token.to_string());
        }),
    );
    tokio::pin!(streamed);
    let first_token = tokio::select! {
        result = &mut streamed => panic!("stream completed before first token: {result:?}"),
        token = token_rx.recv() => token.expect("first token"),
        () = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            panic!("timed out waiting for first LoRA token")
        }
    };
    assert_eq!(first_token, "lora-");
    let streamed = streamed.await.expect("streaming LoRA turn");

    assert_eq!(streamed.reply, "lora-adapter-selected");
    assert_eq!(token_rx.recv().await.as_deref(), Some("adapter-selected"));
}

#[tokio::test]
async fn expert_lora_failure_clears_selection_and_retries_normal_llm() {
    let llm = Arc::new(MockLlmClient {
        reply: "normal-llm-retry".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dual_core_fixture_root())
        .await
        .expect("state");
    let req = SendMessageRequest {
        role_id: "dual-core-lora".to_string(),
        user_message: "activate-lora force-lora-failure".to_string(),
        scene_id: Some("default".to_string()),
        session_id: Some("dual-core-lora-failure".to_string()),
        ..Default::default()
    };

    let response = process_message(&state, &req)
        .await
        .expect("normal LLM retry");

    assert_eq!(response.reply, "normal-llm-retry");
    let srid = session_namespace("dual-core-lora", Some("dual-core-lora-failure"));
    assert_eq!(state.session_cache.expert_lora_plugin_id(&srid), None);
}

#[tokio::test]
async fn expert_lora_partial_stream_is_not_duplicated_by_normal_llm_fallback() {
    let llm = Arc::new(MockLlmClient {
        reply: "normal-llm-must-not-append".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dual_core_fixture_root())
        .await
        .expect("state");
    let activate = SendMessageRequest {
        role_id: "dual-core-lora".to_string(),
        user_message: "activate-lora".to_string(),
        scene_id: Some("default".to_string()),
        session_id: Some("dual-core-lora-partial".to_string()),
        ..Default::default()
    };
    process_message(&state, &activate)
        .await
        .expect("activate LoRA");

    let emitted = Arc::new(Mutex::new(Vec::<String>::new()));
    let emitted_for_sink = Arc::clone(&emitted);
    let partial_request = SendMessageRequest {
        user_message: "force-lora-partial-failure".to_string(),
        ..activate
    };
    let response = process_message_stream(
        &state,
        &partial_request,
        Arc::new(move |token| {
            emitted_for_sink
                .lock()
                .expect("token sink lock")
                .push(token.to_string());
        }),
    )
    .await
    .expect("preserve partial LoRA stream");

    assert_eq!(response.reply, "lora-");
    assert_eq!(emitted.lock().expect("emitted lock").as_slice(), ["lora-"]);
    let srid = session_namespace("dual-core-lora", Some("dual-core-lora-partial"));
    assert_eq!(state.session_cache.expert_lora_plugin_id(&srid), None);
}

#[tokio::test]
async fn experimental_lora_slot_does_not_join_normal_stable_llm_merge() {
    let llm = Arc::new(MockLlmClient {
        reply: "normal-without-lora".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, dual_core_fixture_root())
        .await
        .expect("state");
    let req = SendMessageRequest {
        role_id: "dual-core-lora".to_string(),
        user_message: "ordinary message".to_string(),
        scene_id: Some("default".to_string()),
        session_id: Some("dual-core-no-lora".to_string()),
        ..Default::default()
    };

    let response = process_message(&state, &req)
        .await
        .expect("normal LLM turn");

    assert_eq!(response.reply, "normal-without-lora");
}
