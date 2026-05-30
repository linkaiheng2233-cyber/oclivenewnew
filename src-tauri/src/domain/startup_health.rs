//! One-time integrity self-check before the first `process_message` (fatal errors short-circuit so config issues surface before the first message).

use crate::error::{AppError, Result};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use crate::models::Role;
use crate::state::AppState;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Used with [`AppState::startup_health`]: runs only on the first conversation (`OnceLock`, lock-free hot path).
pub async fn ensure_once(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    if std::env::var("OCLIVE_SKIP_STARTUP_HEALTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }
    if let Some(cached) = state.startup_health.get() {
        return cached
            .clone()
            .map_err(|msg| AppError::StartupHealthFailed(msg.clone()));
    }
    let outcome = run_checks(state, role, effective)
        .await
        .map_err(|e| e.to_string());
    let cached = state
        .startup_health
        .get_or_init(|| outcome.clone());
    cached
        .clone()
        .map_err(AppError::StartupHealthFailed)
}

async fn run_checks(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    validate_plugin_backends_slots(effective)?;
    verify_role_pack_files(state, role)?;
    state.db_manager.health_ping().await?;
    if std::env::var("OCLIVE_SKIP_LLM_STARTUP_PROBE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        tracing::info!(
            target: "oclive_startup",
            "OCLIVE_SKIP_LLM_STARTUP_PROBE set; skipping LLM probe"
        );
    } else if let Err(e) = state.llm.startup_probe().await {
        tracing::warn!(
            target: "oclive_startup",
            "LLM startup_probe non-fatal: {}",
            e.to_frontend_error()
        );
    }
    tracing::info!(
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Validates that `directory` slots have non-empty matching ids in `directory_plugins`.
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
    let blueprint = dir.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME);
    let manifest = dir.join("manifest.json");
    if !blueprint.is_file() && !manifest.is_file() {
        return Err(AppError::RoleNotFound(format!(
            "startup_health: 缺少 {} 或 manifest.json: {}",
            oclive_validation::PIPELINE_BLUEPRINT_FILENAME,
            dir.display()
        )));
    }
    let settings = dir.join("settings.json");
    if !settings.is_file() {
        tracing::warn!(
            target: "oclive_startup",
            "role pack 无 settings.json（可选）: {}",
            settings.display()
        );
    }
    Ok(())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// DB ping for HTTP `--api` etc. after [`AppState`] construction (role-independent).
pub async fn run_global_db_ping(db: &crate::infrastructure::db::DbManager) -> Result<()> {
    db.health_ping().await
}
