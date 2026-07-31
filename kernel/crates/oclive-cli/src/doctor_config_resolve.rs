//! `oclive doctor config-resolve` — print effective six-slot resolution (no duplicate parser).
//!
//! **Default path**: `oclive_kernel_runtime::resolve_session_plugin_backends` + on-disk role pack
//! (no SQLite / Axum). **`--via-host`** (feature `diagnostics-host`): full `AppState` bootstrap for
//! deep parity with desktop. See `creator-docs/COMPATIBILITY.md` · `creator-docs/cli/OCLIVE_CLI_GUIDE.md`.

use anyhow::{Context, Result};
use clap::Parser;
use oclive_kernel_runtime::domain::plugin_resolution::{
    host_ceiling_from_distro_file, pick_llm_backend_env_override, remote_llm_url_token_configured,
    resolve_session_plugin_backends, session_namespace_for_role, SessionPluginResolutionInput,
};
use oclive_kernel_runtime::{
    parse_distro_oclive_file, resolve_project_roles_dir, ENV_DISTRO_PROFILE,
};
use oclive_kernel_types::models::dto::{PluginResolutionDebugInfo, API_VERSION, SCHEMA_VERSION};
use oclive_validation::{
    load_blueprint_slot_registry_for_role_dir, slot_registry_to_plugin_backends, DiskRoleSettings,
    PluginBackends, SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct ConfigResolveArgs {
    /// Manifest role id (e.g. mumu)
    pub role_id: String,

    /// Optional session id (namespace isolation)
    #[arg(long)]
    pub session_id: Option<String>,

    /// Roles root (default: monorepo distros/chat-pro/roles)
    #[arg(short = 'o', long)]
    pub roles_dir: Option<PathBuf>,

    /// Machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Deep diagnostic via full in-memory host bootstrap (requires `diagnostics-host` build)
    #[arg(long)]
    pub via_host: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResolveReport {
    pub schema_version: u32,
    pub role_id: String,
    pub session_id: Option<String>,
    pub plugin_resolution: PluginResolutionDebugInfo,
}

struct RolePackResolutionSnapshot {
    pack_default: PluginBackends,
    slot_registry: Option<BTreeMap<String, SlotRegistryEntry>>,
}

fn load_role_pack_snapshot(role_dir: &Path) -> Result<RolePackResolutionSnapshot> {
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if blueprint_path.is_file() {
        let slot_registry =
            load_blueprint_slot_registry_for_role_dir(role_dir, "0.0.0").map_err(|errors| {
                anyhow::anyhow!(
                    "load blueprint {}: {}",
                    blueprint_path.display(),
                    errors.join("; ")
                )
            })?;
        let pack_default = slot_registry_to_plugin_backends(&slot_registry);
        return Ok(RolePackResolutionSnapshot {
            pack_default,
            slot_registry: Some(slot_registry),
        });
    }
    let settings_path = role_dir.join("settings.json");
    let raw = std::fs::read_to_string(&settings_path)
        .with_context(|| format!("read {}", settings_path.display()))?;
    let settings: DiskRoleSettings = serde_json::from_str(&raw).context("parse settings.json")?;
    let pack_default = settings.plugin_backends.clone().unwrap_or_default();
    Ok(RolePackResolutionSnapshot {
        pack_default,
        slot_registry: None,
    })
}

fn default_host_profile_path(cwd: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(ENV_DISTRO_PROFILE) {
        let t = p.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    let roles_dir = resolve_project_roles_dir(cwd);
    let repo_root = roles_dir.parent()?.parent()?.parent()?;
    let candidate =
        repo_root.join("distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml");
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

fn build_runtime_debug_info(
    role_id: &str,
    session_id: Option<&str>,
    roles_dir: &Path,
) -> Result<PluginResolutionDebugInfo> {
    let role_dir = roles_dir.join(role_id);
    if !role_dir.is_dir() {
        anyhow::bail!("role directory not found: {}", role_dir.display());
    }
    let pack = load_role_pack_snapshot(&role_dir)?;
    let session_ns = session_namespace_for_role(role_id, session_id);
    let host_ceiling = default_host_profile_path(roles_dir)
        .and_then(|p| parse_distro_oclive_file(&p).ok())
        .map(|f| host_ceiling_from_distro_file(&f))
        .unwrap_or_default();
    let resolved = resolve_session_plugin_backends(&SessionPluginResolutionInput {
        pack_plugin_backends: pack.pack_default.clone(),
        pack_slot_registry: pack.slot_registry.clone(),
        session_slot_overrides: BTreeMap::new(),
        user_llm_provider: String::new(),
        llm_env_override: pick_llm_backend_env_override(),
        remote_llm_url_token_configured: remote_llm_url_token_configured(),
        host_ceiling,
    });
    let session_override = None;
    let llm_env_override = pick_llm_backend_env_override().map(|b| match b {
        oclive_validation::LlmBackend::Ollama => "ollama".to_string(),
        oclive_validation::LlmBackend::Remote => "remote".to_string(),
        oclive_validation::LlmBackend::Directory => "directory".to_string(),
        oclive_validation::LlmBackend::None => "none".to_string(),
    });
    let remote_plugin_url_configured = std::env::var("OCLIVE_REMOTE_PLUGIN_URL")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());
    let remote_llm_url_configured = std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());

    Ok(PluginResolutionDebugInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: API_VERSION,
        schema_version: SCHEMA_VERSION,
        role_id: role_id.to_string(),
        session_namespace: session_ns,
        plugin_backends_pack_default: pack.pack_default,
        plugin_backends_session_override: session_override,
        plugin_backends_effective: resolved.backends,
        plugin_backends_effective_sources: resolved.sources,
        llm_env_override,
        remote_plugin_url_configured,
        remote_llm_url_configured,
        local_provider_ids: Vec::new(),
        local_provider_count: 0,
    })
}

#[cfg(feature = "diagnostics-host")]
async fn resolve_via_host(
    args: &ConfigResolveArgs,
    roles_dir: &Path,
) -> Result<PluginResolutionDebugInfo> {
    use oclive_kernel_host::infrastructure::MockLlmClient;
    use oclive_kernel_host::service::role::slot_session::build_plugin_resolution_debug_info;
    use oclive_kernel_host::state::AppState;
    use std::sync::Arc;

    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir)
        .await
        .context("bootstrap in-memory AppState")?;
    state
        .load_role_cached_async(args.role_id.as_str())
        .await
        .context("load role")?;
    build_plugin_resolution_debug_info(&state, args.role_id.as_str(), args.session_id.as_deref())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Resolve effective six-slot backends without printing (test + programmatic use).
///
/// # Errors
///
/// Returns when roles dir resolution, role load, or debug resolution fails.
pub async fn resolve_report(args: &ConfigResolveArgs) -> Result<ConfigResolveReport> {
    let roles_dir = match &args.roles_dir {
        Some(p) => p.clone(),
        None => {
            let cwd = std::env::current_dir().context("cwd")?;
            resolve_project_roles_dir(&cwd)
        }
    };

    let plugin_resolution = if args.via_host {
        #[cfg(feature = "diagnostics-host")]
        {
            resolve_via_host(args, &roles_dir).await?
        }
        #[cfg(not(feature = "diagnostics-host"))]
        {
            anyhow::bail!(
                "`--via-host` requires building oclive-cli with feature `diagnostics-host`"
            );
        }
    } else {
        build_runtime_debug_info(
            args.role_id.as_str(),
            args.session_id.as_deref(),
            &roles_dir,
        )?
    };

    Ok(ConfigResolveReport {
        schema_version: 1,
        role_id: args.role_id.clone(),
        session_id: args.session_id.clone(),
        plugin_resolution,
    })
}

pub async fn run(args: ConfigResolveArgs) -> Result<()> {
    let report = resolve_report(&args).await?;
    if args.json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
    } else {
        let p = &report.plugin_resolution;
        eprintln!("oclive doctor config-resolve — {}", report.role_id);
        println!("  session_namespace: {}", p.session_namespace);
        println!(
            "  memory: {:?} ({:?})",
            p.plugin_backends_effective.memory, p.plugin_backends_effective_sources.memory
        );
        println!(
            "  llm: {:?} ({:?})",
            p.plugin_backends_effective.llm, p.plugin_backends_effective_sources.llm
        );
        println!(
            "  emotion: {:?} ({:?})",
            p.plugin_backends_effective.emotion, p.plugin_backends_effective_sources.emotion
        );
        println!(
            "  event: {:?} ({:?})",
            p.plugin_backends_effective.event, p.plugin_backends_effective_sources.event
        );
        println!(
            "  prompt: {:?} ({:?})",
            p.plugin_backends_effective.prompt, p.plugin_backends_effective_sources.prompt
        );
        println!(
            "  agent: {:?} ({:?})",
            p.plugin_backends_effective.agent, p.plugin_backends_effective_sources.agent
        );
        if let Some(ref llm) = p.llm_env_override {
            println!("  llm_env_override: {llm}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_runtime::chat_pro_roles_dir;

    fn roles_dir() -> PathBuf {
        chat_pro_roles_dir(&[PathBuf::from(env!("CARGO_MANIFEST_DIR"))]).expect("roles")
    }

    #[tokio::test]
    async fn resolve_mumu_defaults() {
        let report = resolve_report(&ConfigResolveArgs {
            role_id: "mumu".into(),
            session_id: None,
            roles_dir: Some(roles_dir()),
            json: false,
            via_host: false,
        })
        .await
        .expect("mumu");
        assert_eq!(report.schema_version, 1);
        assert!(!report.plugin_resolution.session_namespace.is_empty());
    }

    #[tokio::test]
    async fn invalid_role_fails() {
        let err = resolve_report(&ConfigResolveArgs {
            role_id: "not_a_real_role_pack_xyz".into(),
            session_id: None,
            roles_dir: Some(roles_dir()),
            json: true,
            via_host: false,
        })
        .await
        .expect_err("missing role");
        assert!(err.to_string().contains("role directory not found"));
    }

    #[tokio::test]
    async fn json_roundtrip_shape() {
        let report = resolve_report(&ConfigResolveArgs {
            role_id: "mumu".into(),
            session_id: Some("snap".into()),
            roles_dir: Some(roles_dir()),
            json: true,
            via_host: false,
        })
        .await
        .expect("report");
        let json = serde_json::to_string_pretty(&report).expect("serialize");
        let v: serde_json::Value = serde_json::from_str(&json).expect("single JSON document");
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["role_id"], "mumu");
        assert_eq!(v["session_id"], "snap");
        assert!(v.get("plugin_resolution").is_some());
    }

    #[tokio::test]
    async fn env_llm_override_surfaces_in_report() {
        static ENV_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
            std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));
        let _guard = ENV_LOCK.lock().await;
        std::env::remove_var("OCLIVE_LLM_BACKEND");
        std::env::set_var("OCLIVE_LLM_BACKEND", "remote");
        let report = resolve_report(&ConfigResolveArgs {
            role_id: "mumu".into(),
            session_id: None,
            roles_dir: Some(roles_dir()),
            json: false,
            via_host: false,
        })
        .await
        .expect("report");
        std::env::remove_var("OCLIVE_LLM_BACKEND");
        drop(_guard);
        assert_eq!(
            report.plugin_resolution.llm_env_override.as_deref(),
            Some("remote")
        );
    }

    #[test]
    fn cli_dependency_tree_excludes_sqlite_without_diagnostics_host() {
        let out = std::process::Command::new("cargo")
            .args([
                "tree",
                "-p",
                "oclive-cli",
                "--no-default-features",
                "--depth",
                "4",
            ])
            .output()
            .expect("cargo tree");
        assert!(
            out.status.success(),
            "cargo tree failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let tree = String::from_utf8_lossy(&out.stdout);
        assert!(
            !tree.contains("libsqlite3-sys"),
            "oclive-cli default build must not pull SQLite:\n{tree}"
        );
        assert!(
            !tree.contains("axum"),
            "oclive-cli default build must not pull axum:\n{tree}"
        );
        assert!(
            !tree.contains("tauri"),
            "oclive-cli must not activate desktop Tauri dependency:\n{tree}"
        );
    }

    #[test]
    fn config_resolve_json_stdout_is_single_document() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir.join("../..");
        let roles = chat_pro_roles_dir(std::slice::from_ref(&manifest_dir)).expect("roles");
        let exe = std::env::var("CARGO_BIN_EXE_oclive-cli").unwrap_or_else(|_| {
            manifest_dir
                .join("../../target/debug/oclive-cli.exe")
                .to_string_lossy()
                .into_owned()
        });
        if !std::path::Path::new(&exe).is_file() {
            eprintln!("skip config_resolve_json_stdout_is_single_document: {exe} missing");
            return;
        }
        let output = std::process::Command::new(&exe)
            .current_dir(&repo_root)
            .args([
                "doctor",
                "config-resolve",
                "mumu",
                "--json",
                "-o",
                roles.to_string_lossy().as_ref(),
            ])
            .output()
            .expect("spawn oclive-cli");
        assert!(
            output.status.success(),
            "config-resolve failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        assert!(
            trimmed.starts_with('{') && trimmed.ends_with('}'),
            "stdout must be a single JSON document"
        );
        let v: serde_json::Value = serde_json::from_str(trimmed).expect("parse stdout json");
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["role_id"], "mumu");
        assert!(v.get("plugin_resolution").is_some());
    }
}
