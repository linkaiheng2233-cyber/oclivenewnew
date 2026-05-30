#![cfg(feature = "dual_core")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclivenewnew_tauri::domain::chat_engine::plugin_resolve::resolve_plugins_for_session;
use oclivenewnew_tauri::domain::chat_engine::turn_context::TurnContext;
use oclivenewnew_tauri::domain::dual_pipeline::DualPipelineRunner;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::SendMessageRequest;
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

fn dual_core_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/oocp-test-suite/fixtures")
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
        .ensure_interaction_mode_seeded(srid, role.interaction_mode.as_deref())
        .await
        .expect("ensure_interaction_mode_seeded");
    state
        .db_manager
        .set_user_presence_scene(srid, scene_id.as_str())
        .await
        .expect("set_user_presence_scene");

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
    let turn = TurnContext {
        state: &state,
        req: &req,
        role: role.as_ref(),
        scene_id: scene_id.as_str(),
        scenes: Arc::clone(&role.scene_ids),
        mrid,
        srid,
        t0: Instant::now(),
        preflight_ms: 0,
        effective_backends,
        pl,
        immersive: false,
        character_scene_id: None,
        virtual_time_ms: 0,
    };

    let res = DualPipelineRunner::run_experimental(&turn)
        .await
        .expect("experimental happy path");
    assert_eq!(res.reply, "dual-core-happy-path");
    assert_eq!(res.scene_id, "default");
}

