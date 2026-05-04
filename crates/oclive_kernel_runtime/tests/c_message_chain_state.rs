//! 任务 C.2：`send_message` 链路状态一致性（记忆落库、场景/情绪、会话覆盖隔离）。

use oclive_kernel_runtime::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::models::plugin_backends::{MemoryBackend, PluginBackendsOverride};
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "chain_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn memories_persist_across_turns_with_stable_count() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "need roles/shimeng"
    );
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");
    let rid = "shimeng";
    let srid = conversation_state_role_id(rid, None);
    load_role(&state, rid, false).await.expect("load");

    state
        .memory_repo
        .save_memory(srid.as_str(), "integration_seed_memory", 0.9)
        .await
        .expect("save");

    let n0 = state
        .memory_repo
        .count_memories(srid.as_str())
        .await
        .expect("count");

    let req = SendMessageRequest {
        role_id: rid.into(),
        user_message: "hello persistence".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    process_message(&state, &req).await.expect("turn1");

    let n1 = state
        .memory_repo
        .count_memories(srid.as_str())
        .await
        .expect("count");
    assert!(
        n1 >= n0,
        "memory count should not drop after send_message (n0={} n1={})",
        n0,
        n1
    );

    process_message(&state, &req).await.expect("turn2");
    let n2 = state
        .memory_repo
        .count_memories(srid.as_str())
        .await
        .expect("count");
    assert!(
        n2 >= n1,
        "memory count should stay monotonic (n1={} n2={})",
        n1,
        n2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scene_switch_preserves_emotion_dto_non_empty() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");
    let rid = "shimeng";
    let srid = conversation_state_role_id(rid, None);
    load_role(&state, rid, false).await.expect("load");

    state
        .db_manager
        .set_remote_life_enabled(srid.as_str(), true)
        .await
        .expect("remote life");
    state
        .db_manager
        .set_current_scene(srid.as_str(), "default")
        .await
        .expect("char default");

    let r1 = process_message(
        &state,
        &SendMessageRequest {
            role_id: rid.into(),
            user_message: "co-present line".into(),
            scene_id: Some("default".into()),
            session_id: None,
        },
    )
    .await
    .expect("co");
    let s1 = r1.emotion.joy + r1.emotion.neutral + r1.emotion.sadness;
    assert!(s1.is_finite() && s1 > 0.0, "emotion vector should be populated");

    state
        .db_manager
        .set_current_scene(srid.as_str(), "school")
        .await
        .expect("char school");

    let r2 = process_message(
        &state,
        &SendMessageRequest {
            role_id: rid.into(),
            user_message: "remote line".into(),
            scene_id: Some("default".into()),
            session_id: None,
        },
    )
    .await
        .expect("remote");
    assert!(
        !r2.portrait_emotion.is_empty(),
        "portrait emotion should remain populated after scene switch path"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_plugin_backend_override_is_namespaced() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");
    let rid = "shimeng";
    let role = state.load_role_cached(rid).expect("role");

    let ns_a = conversation_state_role_id(rid, Some("sess_a"));
    let ns_b = conversation_state_role_id(rid, Some("sess_b"));

    let ov = PluginBackendsOverride {
        memory: Some(MemoryBackend::BuiltinV2),
        ..Default::default()
    };
    state.set_session_backend_override(ns_a.as_str(), ov);

    assert!(
        state.session_backend_override(ns_a.as_str()).is_some(),
        "sess_a should have override"
    );
    assert!(
        state.session_backend_override(ns_b.as_str()).is_none(),
        "sess_b must not inherit sess_a override"
    );

    let eff_a = state.effective_plugin_backends_for_session(role.as_ref(), ns_a.as_str());
    let eff_b = state.effective_plugin_backends_for_session(role.as_ref(), ns_b.as_str());
    assert_ne!(
        eff_a.memory, eff_b.memory,
        "memory backend should differ when only sess_a is overridden"
    );
}
