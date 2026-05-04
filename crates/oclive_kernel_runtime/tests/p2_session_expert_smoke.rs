//! P2 补强：专家模型参数校验、`process_message` 场景字段与 `plugin_state` 异步读盘烟测。

use async_trait::async_trait;
use oclive_kernel_runtime::domain::chat_engine::conversation_state_role_id;
use oclive_kernel_runtime::domain::chat_engine::process_message;
use oclive_kernel_runtime::domain::expert_models_admin::{
    expert_models_set_role_default, expert_models_set_session_override,
};
use oclive_kernel_runtime::error::Result;
use oclive_kernel_runtime::infrastructure::llm::{LlmClient, MockLlmClient};
use oclive_kernel_runtime::infrastructure::plugin_state::PluginStateStore;
use oclive_kernel_runtime::models::dto::{
    ExpertModelsSetRoleDefaultRequest, ExpertModelsSetSessionOverrideRequest, SendMessageRequest,
};
use oclive_kernel_runtime::models::expert_models::{ExpertGraph, PromptStyleOverride};
use oclive_kernel_runtime::state::KernelAppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

/// 记录所有 `generate` prompt（主对话与辅助任务共用同一 client），用于从中筛出含【复杂情感复盘】的主轮次。
struct AllPromptsCapturingLlm {
    reply: String,
    prompts: Arc<Mutex<Vec<String>>>,
}

impl AllPromptsCapturingLlm {
    fn pair(reply: impl Into<String>) -> (Arc<dyn LlmClient>, Arc<Mutex<Vec<String>>>) {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let llm: Arc<dyn LlmClient> = Arc::new(Self {
            reply: reply.into(),
            prompts: prompts.clone(),
        });
        (llm, prompts)
    }
}

#[async_trait]
impl LlmClient for AllPromptsCapturingLlm {
    async fn generate(&self, _model: &str, prompt: &str) -> Result<String> {
        self.prompts.lock().await.push(prompt.to_string());
        Ok(self.reply.clone())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("neutral".to_string())
    }
}

fn workspace_roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roles")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(p) = to.parent() {
                fs::create_dir_all(p)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn isolated_roles_shimeng_clone(role_dir_name: &str) -> tempfile::TempDir {
    let src = workspace_roles_dir().join("shimeng");
    assert!(src.join("manifest.json").is_file(), "need roles/shimeng");
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join(role_dir_name);
    copy_dir_recursive(&src, &dest).expect("copy");
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    v["id"] = serde_json::Value::String(role_dir_name.to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&v).expect("serialize"),
    )
    .expect("write manifest");
    tmp
}

fn mock_llm() -> Arc<dyn oclive_kernel_runtime::infrastructure::llm::LlmClient> {
    Arc::new(MockLlmClient {
        reply: "p2_ok".to_string(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expert_models_set_session_override_rejects_empty_role_id() {
    let roles = workspace_roles_dir();
    assert!(
        roles.join("shimeng/manifest.json").is_file(),
        "need roles/shimeng"
    );
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let err = expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: "   ".into(),
            session_id: None,
            graph: Default::default(),
            prompt_style: None,
        },
    )
    .await
    .expect_err("empty role_id");

    assert!(
        err.contains("role_id"),
        "expected role_id validation, got {:?}",
        err
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_message_echoes_requested_scene_id() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "shimeng".to_string(),
        user_message: "scene probe".into(),
        scene_id: Some("default".into()),
        session_id: Some("p2_scene_sess".into()),
    };
    let res = process_message(&state, &req)
        .await
        .expect("process_message");
    assert_eq!(res.scene_id, "default");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plugin_state_store_load_async_missing_file_is_default() {
    let dir = tempdir().expect("tmp");
    let p = dir.path().join("no_such_plugin_state.json");
    let s = PluginStateStore::load_async(&p).await;
    assert_eq!(s.schema_version, 3);
    assert!(s.roles.is_empty());
}

/// 共景路径：`resolve_turn` 写入 DB → 下一轮 `PromptInput` 携带 `complex_emotion_hint`（经主 LLM prompt 可见）。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complex_emotion_hint_roundtrip_co_present_prompt() {
    let tmp = isolated_roles_shimeng_clone("p2ce_iso");
    let roles = tmp.path().to_path_buf();
    let rid = "p2ce_iso";
    let (llm, prompts) = AllPromptsCapturingLlm::pair("ce_round_ok".to_string());
    let state = KernelAppState::new_in_memory_with_llm(llm, roles)
        .await
        .expect("state");

    // 无 session：与 `process_message_echoes_requested_scene_id` 一致。
    let srid = conversation_state_role_id(rid, None);

    let req1 = SendMessageRequest {
        role_id: rid.into(),
        user_message: "都行".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    process_message(&state, &req1).await.expect("turn1");

    let hint = state
        .db_manager
        .get_complex_emotion_hint(srid.as_str())
        .await
        .expect("read hint");
    let hint = hint.expect("narrative_hint should persist for builtin_keyword_v1");
    assert!(
        hint.contains("缺乏兴致") || hint.contains("主动提供"),
        "unexpected hint: {}",
        hint
    );

    let req2 = SendMessageRequest {
        role_id: rid.into(),
        user_message: "那我们聊点别的".into(),
        scene_id: Some("default".into()),
        session_id: None,
    };
    process_message(&state, &req2).await.expect("turn2");

    let captured = prompts.lock().await;
    let with_block: Vec<&str> = captured
        .iter()
        .filter(|p| p.contains("【复杂情感复盘】"))
        .map(|s| s.as_str())
        .collect();
    assert!(
        with_block.iter().any(|p| p.contains(hint.trim())),
        "expected narrative_hint in prompt block 【复杂情感复盘】; n_prompts={} snippets={:?}",
        captured.len(),
        with_block
            .iter()
            .map(|p| p.chars().take(260).collect::<String>())
            .collect::<Vec<_>>()
    );
}

/// `effective_prompt_style_override`：会话覆盖非空时优先于角色默认。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn effective_prompt_style_session_override_over_role_default() {
    let roles = workspace_roles_dir();
    let state = KernelAppState::new_in_memory_with_llm(mock_llm(), roles)
        .await
        .expect("state");

    let role_id = "shimeng";
    let session_id = "p2_style_sess";
    let session_ns = conversation_state_role_id(role_id, Some(session_id));

    expert_models_set_role_default(
        &state,
        &ExpertModelsSetRoleDefaultRequest {
            role_id: role_id.into(),
            graph: ExpertGraph {
                version: 7,
                ..Default::default()
            },
            prompt_style: Some(PromptStyleOverride {
                reply_quality_anchor: Some("role_anchor_p2".into()),
                ..Default::default()
            }),
        },
    )
    .await
    .expect("set role default");

    expert_models_set_session_override(
        &state,
        &ExpertModelsSetSessionOverrideRequest {
            role_id: role_id.into(),
            session_id: Some(session_id.into()),
            graph: ExpertGraph {
                version: 99,
                ..Default::default()
            },
            prompt_style: Some(PromptStyleOverride {
                reply_quality_anchor: Some("session_anchor_p2".into()),
                ..Default::default()
            }),
        },
    )
    .await
    .expect("set session override");

    let eff = state
        .effective_prompt_style_override(role_id, session_ns.as_str())
        .await
        .expect("effective style");
    let style = eff.expect("session style present");
    assert_eq!(
        style.reply_quality_anchor.as_deref(),
        Some("session_anchor_p2")
    );
}
