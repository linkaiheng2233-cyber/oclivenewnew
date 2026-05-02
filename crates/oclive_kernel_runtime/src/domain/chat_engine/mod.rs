//! 对话编排入口（迁移中）。
//!
//! 当前阶段先确保 kernel runtime 能复用既有 `process_message`，并逐步将实现从 `src-tauri`
//! 迁入本 crate。

pub mod co_present;
pub mod context;
pub mod favor;
pub mod llm_cancelable;
pub mod presence;
pub mod process_message;
pub mod scene;

pub use process_message::process_message;

use crate::models::dto::EmotionDto;
use crate::models::{PluginBackends, PluginBackendsSourceMap};

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
