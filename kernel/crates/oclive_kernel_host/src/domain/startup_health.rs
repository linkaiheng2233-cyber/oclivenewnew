//! Integrity self-check before `process_message` (fatal errors short-circuit so config issues surface before the first message).

use crate::domain::ports::DbHealthPort;
use crate::error::{AppError, Result};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use crate::models::Role;
use crate::state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const FAILURE_RETRY_TTL: Duration = Duration::from_secs(120);

/// Per-role startup health cache: successes are permanent; failures retry after TTL.
#[derive(Default)]
pub(crate) struct StartupHealthCache {
    successes: HashMap<String, ()>,
    failures: HashMap<String, (String, Instant)>,
    per_role_warnings: HashMap<String, Vec<String>>,
}

impl StartupHealthCache {
    fn cached_outcome(&self, role_id: &str) -> Option<std::result::Result<(), String>> {
        if self.successes.contains_key(role_id) {
            return Some(Ok(()));
        }
        if let Some((msg, at)) = self.failures.get(role_id) {
            if at.elapsed() < FAILURE_RETRY_TTL {
                return Some(Err(msg.clone()));
            }
        }
        None
    }

    fn record(
        &mut self,
        role_id: &str,
        outcome: std::result::Result<(), String>,
        warnings: Vec<String>,
    ) {
        self.failures.remove(role_id);
        if warnings.is_empty() {
            self.per_role_warnings.remove(role_id);
        } else {
            self.per_role_warnings.insert(role_id.to_string(), warnings);
        }
        match outcome {
            Ok(()) => {
                self.successes.insert(role_id.to_string(), ());
            }
            Err(msg) => {
                self.failures
                    .insert(role_id.to_string(), (msg, Instant::now()));
            }
        }
    }

    /// Deduped union of non-fatal startup warnings recorded across roles.
    pub(crate) fn aggregated_warnings(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for warnings in self.per_role_warnings.values() {
            for w in warnings {
                if !out.iter().any(|existing| existing == w) {
                    out.push(w.clone());
                }
            }
        }
        out
    }
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn ensure_once(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    if std::env::var("OCLIVE_SKIP_STARTUP_HEALTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }
    let role_id = role.id.as_str();
    if let Some(cached) = state.startup_health.read().cached_outcome(role_id) {
        return cached.map_err(AppError::StartupHealthFailed);
    }
    let (outcome, warnings) = match run_checks(state, role, effective).await {
        Ok(()) => (
            Ok(()),
            collect_remote_backend_placeholder_warnings(effective),
        ),
        Err(e) => (Err(e.to_string()), Vec::new()),
    };
    state
        .startup_health
        .write()
        .record(role_id, outcome.clone(), warnings);
    outcome.map_err(AppError::StartupHealthFailed)
}

async fn run_checks(state: &AppState, role: &Role, effective: &PluginBackends) -> Result<()> {
    validate_plugin_backends_slots(effective)?;
    validate_co_present_dialogue_backends(effective)?;
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
    } else {
        let llm = Arc::clone(&state.llm);
        tokio::spawn(async move {
            if let Err(e) = llm.startup_probe().await {
                tracing::warn!(
                    target: "oclive_startup",
                    "LLM startup_probe non-fatal: {}",
                    e.to_frontend_error()
                );
            }
        });
    }
    warn_remote_backend_placeholders(effective);
    tracing::info!(
        target: "oclive_startup",
        "startup_health ok role_id={}",
        role.id
    );
    Ok(())
}

fn env_nonempty(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| !v.trim().is_empty())
}

/// Non-fatal: remote backends without env endpoints degrade to placeholders (see `remote_plugin`).
fn collect_remote_backend_placeholder_warnings(pb: &PluginBackends) -> Vec<String> {
    let mut out = Vec::new();
    let plugin_remote = matches!(pb.memory, MemoryBackend::Remote)
        || matches!(pb.emotion, EmotionBackend::Remote)
        || matches!(pb.event, EventBackend::Remote)
        || matches!(pb.prompt, PromptBackend::Remote);
    if plugin_remote && !env_nonempty("OCLIVE_REMOTE_PLUGIN_URL") {
        out.push(
            "plugin_backends 含 remote（memory/emotion/event/prompt），但未配置 OCLIVE_REMOTE_PLUGIN_URL；将使用占位实现，对话质量受限"
                .to_string(),
        );
    }
    if matches!(pb.llm, LlmBackend::Remote) && !env_nonempty("OCLIVE_REMOTE_LLM_URL") {
        out.push(
            "plugin_backends.llm=remote，但未配置 OCLIVE_REMOTE_LLM_URL；将使用占位 LLM 实现"
                .to_string(),
        );
    }
    if matches!(pb.agent, AgentBackend::Remote) && !env_nonempty("OCLIVE_REMOTE_AGENT_URL") {
        out.push(
            "plugin_backends.agent=remote，但未配置 OCLIVE_REMOTE_AGENT_URL；将使用占位 Agent 实现"
                .to_string(),
        );
    }
    out
}

/// Non-fatal: remote backends without env endpoints degrade to placeholders (see `remote_plugin`).
fn warn_remote_backend_placeholders(pb: &PluginBackends) {
    for msg in collect_remote_backend_placeholder_warnings(pb) {
        tracing::warn!(target: "oclive_startup", "{msg}");
    }
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

/// Co-present dialogue requires prompt assembly and LLM generation.
///
/// # Errors
///
/// Returns [`Err`] when `prompt` or `llm` is set to `none`.
pub fn validate_co_present_dialogue_backends(pb: &PluginBackends) -> Result<()> {
    if matches!(pb.llm, LlmBackend::None) {
        return Err(AppError::InvalidParameter(
            "plugin_backends.llm=none is not allowed on the co-present dialogue path".into(),
        ));
    }
    if matches!(pb.prompt, PromptBackend::None) {
        return Err(AppError::InvalidParameter(
            "plugin_backends.prompt=none is not allowed on the co-present dialogue path".into(),
        ));
    }
    Ok(())
}

fn verify_role_pack_files(state: &AppState, role: &Role) -> Result<()> {
    let dir = role
        .source_dir
        .as_deref()
        .filter(|p| p.is_dir())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| state.storage.roles_dir().join(role.id.as_str()));
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
pub async fn run_global_db_ping(db: &impl DbHealthPort) -> Result<()> {
    db.health_ping().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plugin_backends::{LlmBackend, MemoryBackend, PluginBackends};

    #[test]
    fn collect_remote_placeholder_warnings_when_env_missing() {
        let pb = PluginBackends {
            memory: MemoryBackend::Remote,
            ..Default::default()
        };
        let warnings = collect_remote_backend_placeholder_warnings(&pb);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("OCLIVE_REMOTE_PLUGIN_URL"));
    }

    #[test]
    fn collect_remote_llm_warning_when_url_missing() {
        let pb = PluginBackends {
            llm: LlmBackend::Remote,
            ..Default::default()
        };
        let warnings = collect_remote_backend_placeholder_warnings(&pb);
        assert!(warnings.iter().any(|w| w.contains("OCLIVE_REMOTE_LLM_URL")));
    }

    #[test]
    fn aggregated_warnings_dedupes_across_roles() {
        let mut cache = StartupHealthCache::default();
        cache.record("role_a", Ok(()), vec!["shared".into(), "only_a".into()]);
        cache.record("role_b", Ok(()), vec!["shared".into(), "only_b".into()]);
        let agg = cache.aggregated_warnings();
        assert_eq!(agg.len(), 3);
        assert!(agg.contains(&"shared".to_string()));
        assert!(agg.contains(&"only_a".to_string()));
        assert!(agg.contains(&"only_b".to_string()));
    }

    #[test]
    fn aggregated_warnings_empty_when_no_roles() {
        let cache = StartupHealthCache::default();
        assert!(cache.aggregated_warnings().is_empty());
    }

    #[tokio::test]
    async fn verify_role_pack_files_uses_source_dir_outside_roles_dir() {
        use std::fs;
        use std::sync::Arc;
        use tempfile::TempDir;

        let roles_root = TempDir::new().unwrap();
        let fixture_parent = TempDir::new().unwrap();
        let fixture_dir = fixture_parent.path().join("external-fixture");
        fs::create_dir_all(&fixture_dir).unwrap();
        fs::write(
            fixture_dir.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME),
            "{}",
        )
        .unwrap();

        let role = Role {
            id: "external-fixture".to_string(),
            source_dir: Some(fixture_dir),
            ..Default::default()
        };

        let state = AppState::new_in_memory_with_llm(
            Arc::new(crate::infrastructure::llm::MockLlmClient {
                reply: "ok".to_string(),
            }),
            roles_root.path().to_path_buf(),
        )
        .await
        .expect("state");

        verify_role_pack_files(&state, &role).expect("verify with source_dir");
    }
}
