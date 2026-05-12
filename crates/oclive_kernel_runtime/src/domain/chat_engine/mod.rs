//! 对话编排入口（**权威实现位于本 crate**）。
//!
//! `process_message` 与分支（共景 / 异地占位 / 异地心声 / Agent 早退）为桌面与 OOCP 的共享编排；Tauri `api` 层仅做 `invoke` 与 DTO 映射，不再维护第二套对话逻辑。

pub mod co_present;
pub mod context;
pub mod favor;
pub mod llm_cancelable;
pub mod pipeline_actions;
pub mod pipeline_interpreter;
pub mod pipeline_loader;
pub mod turn_context;
pub mod presence;
pub mod process_message;
pub mod scene;

pub use process_message::process_message;

use crate::error::Result;
use crate::models::dto::EmotionDto;
use crate::models::plugin_backends::LlmBackend;
use crate::models::{PluginBackends, PluginBackendsSourceMap, Role};
use crate::state::KernelAppState;

/// 主对话 `generate` 的 `model` 参数：目录/Ollama 用解析后的 Ollama 名；云端用专家图会话覆盖，否则空串走宿主默认。
pub(crate) async fn resolve_main_llm_model_for_generate(
    state: &KernelAppState,
    role: &Role,
    session_ns: &str,
) -> Result<String> {
    let backends = state.effective_plugin_backends_for_session(role, session_ns);
    if backends.llm != LlmBackend::Remote {
        return Ok(role.resolve_ollama_model(state.global_chat_model().as_str()));
    }
    let row = state
        .db_manager
        .get_expert_cloud_model_session_override(session_ns)
        .await?;
    Ok(row
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_default())
}

pub(super) fn emotion_to_dto(r: &crate::domain::emotion_analyzer::EmotionResult) -> EmotionDto {
    EmotionDto {
        joy: r.joy as f32,
        sadness: r.sadness as f32,
        anger: r.anger as f32,
        fear: r.fear as f32,
        surprise: r.surprise as f32,
        disgust: r.disgust as f32,
        neutral: r.neutral as f32,
    }
}

fn backend_resolution_summary(
    effective: &PluginBackends,
    sources: &PluginBackendsSourceMap,
) -> String {
    format!(
        "mem={:?}({:?}) emotion={:?}({:?}) event={:?}({:?}) prompt={:?}({:?}) llm={:?}({:?}) agent={:?}({:?})",
        effective.memory,
        sources.memory,
        effective.emotion,
        sources.emotion,
        effective.event,
        sources.event,
        effective.prompt,
        sources.prompt,
        effective.llm,
        sources.llm,
        effective.agent,
        sources.agent
    )
}

/// 会话级 SQLite 命名空间：HTTP 试聊传入 `session_id` 时与无 `session_id` 的默认对话隔离。
pub fn conversation_state_role_id(manifest_role_id: &str, session_id: Option<&str>) -> String {
    /// 控制 SQLite 键与日志长度，避免异常长 `session_id` 撑爆存储。
    const MAX_SUFFIX_CHARS: usize = 64;
    const MAX_TOTAL_CHARS: usize = 256;

    let sid = session_id.map(str::trim).filter(|s| !s.is_empty());
    match sid {
        None => manifest_role_id.chars().take(MAX_TOTAL_CHARS).collect(),
        Some(s) => {
            let safe: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(MAX_SUFFIX_CHARS)
                .collect();
            let out = format!("{}__sess__{}", manifest_role_id, safe);
            out.chars().take(MAX_TOTAL_CHARS).collect()
        }
    }
}
