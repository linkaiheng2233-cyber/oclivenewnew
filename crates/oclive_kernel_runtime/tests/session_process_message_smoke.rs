//! 会话最小闭环（P0）：内存 SQLite + Mock LLM + 仓库内角色包，走 `process_message`。
//! 依赖仓库根 `roles/shimeng`（与 CI checkout 一致）。
//! Windows：若整包 `cargo test -p oclive_kernel_runtime` 并行链接偶发 `LNK1104`，可设环境变量 `CARGO_BUILD_JOBS=1` 或加 `-j 1`。

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::{SendMessageRequest, API_VERSION};
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_smoke_with_shimeng_and_mock_llm() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "expected roles/shimeng in repo checkout (roles dir = {:?})",
        roles
    );

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "smoke_reply_ok".to_string(),
        }),
        roles,
    )
    .await
    .expect("KernelAppState");

    let req = SendMessageRequest {
        role_id: "shimeng".to_string(),
        user_message: "你好".to_string(),
        scene_id: None,
        session_id: Some("kernel_smoke_sess".to_string()),
    };

    let res = process_message(&state, &req)
        .await
        .expect("process_message should succeed for shimeng + mock llm");

    assert_eq!(res.api_version, API_VERSION);
    assert!(
        res.reply.contains("smoke_reply_ok"),
        "expected mock reply in {:?}",
        res.reply
    );
    assert!(!res.reply.is_empty());
}
