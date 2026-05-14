//! 首次 `process_message` 前的启动期完整性自检（致命错误短路，避免首条消息才暴露配置问题）。

use crate::error::{AppError, Result};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use crate::models::Role;
use crate::state::AppState;

/// 与 [`AppState::startup_health`] 配合：仅首轮对话执行一次（锁不跨越 `.await`）。
pub async fn ensure_once(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    if std::env::var("OCLIVE_SKIP_STARTUP_HEALTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }
    {
        let g = state.startup_health.lock();
        match &*g {
            Some(Err(msg)) => {
                return Err(AppError::InvalidParameter(format!(
                    "startup_health: {}",
                    msg
                )));
            }
            Some(Ok(())) => return Ok(()),
            None => {}
        }
    }
    let outcome = run_checks(state, role, effective).await;
    let mut g = state.startup_health.lock();
    if g.is_none() {
        *g = Some(
            outcome
                .as_ref()
                .map(|_| ())
                .map_err(|e| e.to_frontend_error()),
        );
    }
    outcome
}

async fn run_checks(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    validate_plugin_backends_slots(effective)?;
    verify_role_pack_files(state, role)?;
    state.db_manager.health_ping().await?;
    if std::env::var("OCLIVE_SKIP_LLM_STARTUP_PROBE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        log::info!(
            target: "oclive_startup",
            "OCLIVE_SKIP_LLM_STARTUP_PROBE set; skipping LLM probe"
        );
    } else if let Err(e) = state.llm.startup_probe().await {
        log::warn!(
            target: "oclive_startup",
            "LLM startup_probe non-fatal: {}",
            e.to_frontend_error()
        );
    }
    log::info!(
        target: "oclive_startup",
        "startup_health ok role_id={}",
        role.id
    );
    Ok(())
}

fn non_empty_slot(id: &Option<String>, slot: &str) -> Result<()> {
    let ok = id.as_ref().is_some_and(|s| !s.trim().is_empty());
    if ok {
        Ok(())
    } else {
        Err(AppError::InvalidParameter(format!(
            "plugin_backends: `{}` 使用 directory 时必须配置 `directory_plugins.{}` 为非空插件 id",
            slot, slot
        )))
    }
}

/// 校验 `directory` 槽位与 `directory_plugins` 的 id 非空对应。
pub fn validate_plugin_backends_slots(pb: &PluginBackends) -> Result<()> {
    if matches!(pb.memory, MemoryBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.memory, "memory")?;
    }
    if matches!(pb.emotion, EmotionBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.emotion, "emotion")?;
    }
    if matches!(pb.event, EventBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.event, "event")?;
    }
    if matches!(pb.prompt, PromptBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.prompt, "prompt")?;
    }
    if matches!(pb.llm, LlmBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.llm, "llm")?;
    }
    if matches!(pb.agent, AgentBackend::Directory) {
        non_empty_slot(&pb.directory_plugins.agent, "agent")?;
    }
    Ok(())
}

fn verify_role_pack_files(state: &AppState, role: &Role) -> Result<()> {
    let dir = state.storage.roles_dir().join(role.id.as_str());
    let manifest = dir.join("manifest.json");
    if !manifest.is_file() {
        return Err(AppError::RoleNotFound(format!(
            "startup_health: 缺少 manifest.json: {}",
            manifest.display()
        )));
    }
    let settings = dir.join("settings.json");
    if !settings.is_file() {
        log::warn!(
            target: "oclive_startup",
            "role pack 无 settings.json（可选）: {}",
            settings.display()
        );
    }
    Ok(())
}

/// 供 HTTP `--api` 等在 [`AppState`] 构造后做一次与角色无关的 DB 探活。
pub async fn run_global_db_ping(db: &crate::infrastructure::db::DbManager) -> Result<()> {
    db.health_ping().await
}
