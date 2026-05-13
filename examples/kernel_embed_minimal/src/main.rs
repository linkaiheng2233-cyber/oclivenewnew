//! 嵌入式最小闭环：内存库 + Mock LLM + `process_message`。
//!
//! 从**仓库根**运行（需存在 `roles/shimeng`，与集成测试一致）：
//! `cargo run -p kernel_embed_minimal`

use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::infrastructure::llm::MockLlmClient;
use oclive_kernel_runtime::models::dto::SendMessageRequest;
use oclive_kernel_runtime::state::KernelAppState;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let roles = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles");
    if !roles.join("shimeng/manifest.json").is_file() {
        eprintln!(
            "missing roles/shimeng (expected under {:?}); clone repo with roles/",
            roles
        );
        std::process::exit(2);
    }

    let state = KernelAppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "（示例）我在听。".to_string(),
        }),
        roles,
    )
    .await?;

    let req = SendMessageRequest {
        role_id: "shimeng".to_string(),
        user_message: "你好 oclive".to_string(),
        scene_id: None,
        session_id: Some("embed_minimal".to_string()),
    };
    let res = process_message(&state, &req).await?;
    println!("{}", res.reply);
    Ok(())
}
