//! `process_message` 入口直编后的细粒度回归（与 `p3_complex_emotion_three_paths` 互补）。
//!
//! 覆盖：`chat_generation_cancel` 在每条消息入口被复位（原先由默认入口蓝图中的 `init_turn` 完成）。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_resets_chat_generation_cancel_at_entry() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "expected roles/shimeng (roles dir = {:?})",
        roles
    );

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "ce_refine_cancel_ok".to_string(),
        }),
        roles,
    )
    .await
    .expect("KernelAppState");

    state.chat_generation_cancel.store(true, Ordering::Release);

    let req = SendMessageRequest {
        role_id: "shimeng".to_string(),
        user_message: "ping".to_string(),
        scene_id: None,
        session_id: Some("ce_refine_cancel_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message should succeed");
    assert!(
        res.reply.contains("ce_refine_cancel_ok"),
        "unexpected reply: {:?}",
        res.reply
    );
    assert!(
        !state.chat_generation_cancel.load(Ordering::Acquire),
        "chat_generation_cancel should be cleared at process_message entry"
    );
}
