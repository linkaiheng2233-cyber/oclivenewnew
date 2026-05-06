//! 长对话冒烟：多轮 `process_message` 后记忆条数应保持在合理上限内（非 RSS 剖析，用于防明显逻辑泄漏）。
//!
//! 仅当启用 crate feature **`slow-long-tests`** 时本 target 才会被编译（避免默认 `cargo test --workspace` 额外链接）。
//! 运行（仍 **`#[ignore]`**，需显式 `--ignored`）：
//! `cargo test -p oclive_kernel_runtime --features "full,slow-long-tests" --test p_long_chat_memory_bounds -- --ignored --nocapture`

use oclive_kernel_runtime::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::role_lifecycle::load_role;
use oclive_kernel_runtime::infrastructure::llm::{LlmClient, MockLlmClient};
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn mock_llm() -> Arc<dyn LlmClient> {
    Arc::new(MockLlmClient {
        reply: "long_run_ok".into(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "slow: run locally or in manual CI with --ignored"]
async fn p_long_chat_memory_stays_bounded() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "need roles/shimeng"
    );
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), &roles)
        .await
        .expect("state");
    let rid = "shimeng";
    let srid = conversation_state_role_id(rid, None);
    load_role(&state, rid, false).await.expect("load");

    let rounds: usize = 100;
    for i in 0..rounds {
        let req = SendMessageRequest {
            role_id: rid.into(),
            user_message: format!("round {i} ping"),
            session_id: None,
            scene_id: None,
        };
        process_message(&state, &req)
            .await
            .expect("process_message");
    }

    let n = state
        .memory_repo
        .count_memories(srid.as_str())
        .await
        .expect("count");
    assert!(
        n <= 600,
        "memory count unexpectedly large after {rounds} turns: {n}"
    );
}
