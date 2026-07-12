//! `oclive doctor config-resolve` — print effective six-slot resolution (no duplicate parser).
//!
//! **Dependency boundary**: reuses `oclive_kernel_host::AppState` + `build_plugin_resolution_debug_info`
//! (in-memory SQLite, no Tauri). Keeps CLI diagnostics aligned with desktop resolution without a
//! second parser. See `creator-docs/COMPATIBILITY.md` · `creator-docs/cli/OCLIVE_CLI_GUIDE.md`.

use anyhow::{Context, Result};
use clap::Parser;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::models::dto::PluginResolutionDebugInfo;
use oclive_kernel_host::service::role::slot_session::build_plugin_resolution_debug_info;
use oclive_kernel_host::state::AppState;
use oclive_kernel_runtime::resolve_project_roles_dir;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

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
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResolveReport {
    pub schema_version: u32,
    pub role_id: String,
    pub session_id: Option<String>,
    pub plugin_resolution: PluginResolutionDebugInfo,
}

/// Resolve effective six-slot backends without printing (test + programmatic use).
///
/// # Errors
///
/// Returns when roles dir resolution, in-memory bootstrap, role load, or debug resolution fails.
pub async fn resolve_report(args: &ConfigResolveArgs) -> Result<ConfigResolveReport> {
    let roles_dir = match &args.roles_dir {
        Some(p) => p.clone(),
        None => {
            let cwd = std::env::current_dir().context("cwd")?;
            resolve_project_roles_dir(&cwd)
        }
    };
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, &roles_dir)
        .await
        .context("bootstrap in-memory AppState")?;
    state
        .load_role_cached_async(args.role_id.as_str())
        .await
        .context("load role")?;
    let plugin_resolution = build_plugin_resolution_debug_info(
        &state,
        args.role_id.as_str(),
        args.session_id.as_deref(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
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
        })
        .await
        .expect_err("missing role");
        assert!(err.to_string().contains("load role"));
    }

    #[tokio::test]
    async fn json_roundtrip_shape() {
        let report = resolve_report(&ConfigResolveArgs {
            role_id: "mumu".into(),
            session_id: Some("snap".into()),
            roles_dir: Some(roles_dir()),
            json: true,
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
        use oclive_kernel_host::service::role::slot_session::build_plugin_resolution_debug_info;
        static ENV_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
            std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));
        let llm = Arc::new(MockLlmClient {
            reply: "ok".to_string(),
        });
        let state = AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state");
        state
            .load_role_cached_async("mumu")
            .await
            .expect("load mumu");
        let _guard = ENV_LOCK.lock().await;
        std::env::remove_var("OCLIVE_LLM_BACKEND");
        std::env::set_var("OCLIVE_LLM_BACKEND", "remote");
        let debug = build_plugin_resolution_debug_info(&state, "mumu", None)
            .await
            .expect("debug");
        std::env::remove_var("OCLIVE_LLM_BACKEND");
        drop(_guard);
        assert_eq!(debug.llm_env_override.as_deref(), Some("remote"));
    }

    #[test]
    fn cli_dependency_tree_excludes_tauri() {
        let out = std::process::Command::new("cargo")
            .args(["tree", "-p", "oclive-cli", "--depth", "1"])
            .output()
            .expect("cargo tree");
        assert!(
            out.status.success(),
            "cargo tree failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let tree = String::from_utf8_lossy(&out.stdout);
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
