//! `oclive kernel ensure` — shared policy + optional execution.

use anyhow::{Context, Result};
use clap::Parser;
use oclive_kernel_runtime::{
    apply_promote_to_candidate, build_resolve_plan, discover_spawn_kernel_candidates,
    ensure_app_data_dir, promote_with_backup, find_app_data_dir_for_host, ActiveProfileSummary,
    DistroProfileRequirements, KernelHealthJson, PolicyContext, ProfileCompat, KernelActionKind,
    KernelBinaryManifest, KernelCandidate, DEFAULT_API_PORT, ENV_DISTRO_ID, ENV_DISTRO_PROFILE,
    ENV_HTTP_API_MOCK_LLM, ENV_ROLES_DIR, terminate_listeners_on_port,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct KernelEnsureArgs {
    #[arg(long, default_value_t = DEFAULT_API_PORT)]
    pub port: u16,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub plan_only: bool,
    #[arg(long)]
    pub distro: Option<String>,
    #[arg(long)]
    pub roles_dir: Option<PathBuf>,
    #[arg(long)]
    pub bundled_binary: Option<PathBuf>,
    #[arg(long)]
    pub settings_binary: Option<PathBuf>,
    #[arg(long)]
    pub kernel_pinned: bool,
    #[arg(long, default_value_t = true)]
    pub allow_replace: bool,
    #[arg(long)]
    pub lock_running: bool,
    #[arg(long, default_value_t = true)]
    pub promote_shared: bool,
    #[arg(long)]
    pub distro_profile: Option<PathBuf>,
    #[arg(long)]
    pub mock_llm: bool,
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct HealthProbeJson {
    #[serde(flatten)]
    base: KernelHealthJson,
    #[serde(default)]
    kernel_manifest: Option<KernelBinaryManifest>,
}

#[derive(Serialize)]
struct EnsureReport {
    schema_version: u32,
    plan: oclive_kernel_runtime::KernelActionPlan,
    profile_compat: ProfileCompat,
    caller_requirements: DistroProfileRequirements,
    running_profile_summary: Option<ActiveProfileSummary>,
    executed: bool,
    health_ok: bool,
    running_distro_id: Option<String>,
}

pub fn run_ensure(args: KernelEnsureArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let anchors = vec![root.clone()];
    let settings = args.settings_binary.as_deref();
    let bundled = args.bundled_binary.as_deref();
    let mut candidates = discover_spawn_kernel_candidates(&anchors, settings, bundled);

    let health_probe = fetch_health_json(args.port);
    let health_base = health_probe.as_ref().map(|h| &h.base);
    let running_distro_id = health_probe
        .as_ref()
        .and_then(|h| h.base.distro_id.clone());

    let mut ctx = PolicyContext::from_health(health_base);
    ctx.running_manifest = health_probe
        .as_ref()
        .and_then(|h| h.kernel_manifest.clone());

    let distro_id = args.distro.as_deref().unwrap_or("desktop");
    let resolution = build_resolve_plan(
        &ctx,
        &candidates,
        distro_id,
        args.distro_profile.as_deref(),
        args.kernel_pinned,
        args.allow_replace && !args.lock_running,
        args.promote_shared,
    );

    let mut executed = false;
    if !args.plan_only {
        executed = execute_plan(&args, &resolution.plan, &mut candidates)?;
    }

    let health_ok_after = fetch_health_json(args.port).is_some_and(|h| h.base.ok);

    let report = EnsureReport {
        schema_version: 2,
        plan: resolution.plan,
        profile_compat: resolution.profile_compat,
        caller_requirements: resolution.caller_requirements,
        running_profile_summary: resolution.running_profile_summary,
        executed,
        health_ok: health_ok_after,
        running_distro_id,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("oclive kernel ensure");
        println!("  action:   {:?}", report.plan.action);
        if let Some(ref c) = report.plan.candidate {
            println!("  binary:   {}", c.binary);
            println!("  tier:     {:?}", c.tier);
        }
        if let Some(ref r) = report.plan.attach_reason {
            println!("  attach:   {r:?}");
        }
        if let Some(ref r) = report.plan.replace_reason {
            println!("  replace:  {r:?}");
        }
        println!("  compat:   {:?}", report.profile_compat);
        println!(
            "  health:   {}",
            if report.health_ok { "ok" } else { "offline" }
        );
        if args.plan_only {
            println!("  (plan-only, no execution)");
        }
    }
    Ok(())
}

fn execute_plan(
    args: &KernelEnsureArgs,
    plan: &oclive_kernel_runtime::KernelActionPlan,
    candidates: &mut [KernelCandidate],
) -> Result<bool> {
    match plan.action {
        KernelActionKind::Attach => return Ok(false),
        KernelActionKind::ReplaceAndAttach
        | KernelActionKind::SpawnBest
        | KernelActionKind::FallbackBundled => {}
    }

    let Some(ref sel) = plan.candidate else {
        anyhow::bail!("plan has no candidate for {:?}", plan.action);
    };

    let mut candidate = candidates
        .iter()
        .find(|c| c.binary.display().to_string() == sel.binary)
        .cloned()
        .context("candidate binary not in discovery list")?;

    if sel.promote_to_shared {
        if let Some(m) = KernelBinaryManifest::read_sidecar(&candidate.binary) {
            let _ = promote_with_backup(&candidate.binary, Some(&m));
        } else {
            let _ = promote_with_backup(&candidate.binary, None);
        }
        apply_promote_to_candidate(&mut candidate);
    }

    if matches!(plan.action, KernelActionKind::ReplaceAndAttach) {
        terminate_listeners_on_port(args.port);
        thread::sleep(Duration::from_millis(400));
    }

    let roles_dir = args
        .roles_dir
        .clone()
        .context("--roles-dir required to spawn kernel")?;
    spawn_kernel(
        &candidate.binary,
        args.port,
        &roles_dir,
        args.distro.as_deref(),
        args.distro_profile.as_deref(),
        args.mock_llm,
    )?;
    Ok(true)
}

fn fetch_health_json(port: u16) -> Option<HealthProbeJson> {
    let url = format!("http://127.0.0.1:{port}/health");
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .build();
    let resp = agent
        .get(&url)
        .set("Accept", "application/json")
        .call()
        .ok()?;
    if resp.status() != 200 {
        return None;
    }
    serde_json::from_str(&resp.into_string().ok()?).ok()
}

fn spawn_kernel(
    binary: &Path,
    port: u16,
    roles_dir: &Path,
    distro_id: Option<&str>,
    distro_profile: Option<&Path>,
    mock_llm: bool,
) -> Result<()> {
    let app_data = find_app_data_dir_for_host();
    ensure_app_data_dir(&app_data).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let mut cmd = Command::new(binary);
    cmd.arg("--api")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .env("OCLIVE_API_PORT", port.to_string())
        .env("OCLIVE_APP_DATA", app_data)
        .env("OCLIVE_USE_CANONICAL_APP_DATA", "1")
        .env(ENV_ROLES_DIR, roles_dir);

    if mock_llm {
        cmd.env(ENV_HTTP_API_MOCK_LLM, "1");
    }
    if let Some(id) = distro_id.filter(|s| !s.is_empty()) {
        cmd.env(ENV_DISTRO_ID, id);
    }
    if let Some(p) = distro_profile.filter(|p| p.is_file()) {
        cmd.env(ENV_DISTRO_PROFILE, p);
    }

    let mut child = cmd.spawn().with_context(|| format!("spawn {}", binary.display()))?;

    for _ in 0..40 {
        if fetch_health_json(port).is_some_and(|h| h.base.ok) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    let _ = child.kill();
    anyhow::bail!("spawned {} but /health did not become ready", binary.display())
}
