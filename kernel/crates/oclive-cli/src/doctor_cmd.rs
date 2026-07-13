//! `oclive doctor` — one-click environment diagnostics.

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct DoctorArgs {
    #[command(subcommand)]
    pub subcommand: Option<DoctorSubcommand>,

    /// Machine-readable JSON (`schema_version` + `checks[]`)
    #[arg(long)]
    pub json: bool,

    /// Workspace probe directory (writable check; default: current directory)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Apply auto-fixable checks (interactive confirmation)
    #[arg(long)]
    pub fix: bool,

    /// With `--fix`: skip confirmation prompts
    #[arg(long)]
    pub yes: bool,

    /// Poll environment every 60s; press q to quit
    #[arg(long)]
    pub watch: bool,

    /// Generate SBOM (requires cargo-cyclonedx)
    #[arg(long)]
    pub sbom: bool,

    /// SBOM format: cyclonedx (default) or spdx
    #[arg(long = "sbom-format", default_value = "cyclonedx")]
    pub sbom_format: String,
}

#[derive(clap::Subcommand, Debug)]
pub enum DoctorSubcommand {
    /// Print effective six-slot backend resolution for a role/session
    #[command(name = "config-resolve")]
    ConfigResolve(crate::doctor_config_resolve::ConfigResolveArgs),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorReport {
    pub schema_version: u32,
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorCheck {
    pub id: String,
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip)]
    pub fix_command: Option<Vec<String>>,
}

impl DoctorCheck {
    pub(crate) fn ok(id: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "ok".into(),
            message: message.into(),
            detail: None,
            fix_command: None,
        }
    }

    pub(crate) fn warn(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "warn".into(),
            message: message.into(),
            detail,
            fix_command: None,
        }
    }

    pub(crate) fn fail(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "fail".into(),
            message: message.into(),
            detail,
            fix_command: None,
        }
    }
}

pub fn run(args: DoctorArgs) -> Result<()> {
    if let Some(sub) = args.subcommand {
        match sub {
            DoctorSubcommand::ConfigResolve(cfg) => {
                let rt = tokio::runtime::Runtime::new().context("tokio runtime")?;
                rt.block_on(crate::doctor_config_resolve::run(cfg))
            }
        }?;
        return Ok(());
    }
    let root = if args.path.is_absolute() {
        args.path.clone()
    } else {
        std::env::current_dir()?.join(&args.path)
    };
    if args.watch {
        return run_watch(&root, args.json);
    }
    if args.sbom {
        return crate::doctor_sbom::run_sbom(&root, &args.sbom_format);
    }
    let mut checks = vec![
        check_rust_toolchain(),
        check_cargo(),
        check_cpp_toolchain(),
        check_system_memory(),
        check_disk_space(&root),
        check_ollama(),
        check_network_github(),
        check_workspace_writable(&root),
    ];
    checks.extend(crate::doctor_blueprint::blueprint_v2_checks(&root));
    checks.extend(crate::doctor_kernel_contracts::kernel_contract_impl_checks(
        &root,
    ));
    if args.fix {
        apply_fixes(&checks, args.yes)?;
        checks = vec![
            check_rust_toolchain(),
            check_cargo(),
            check_cpp_toolchain(),
            check_system_memory(),
            check_disk_space(&root),
            check_ollama(),
            check_network_github(),
            check_workspace_writable(&root),
        ];
    }
    let ok = checks.iter().all(|c| c.status != "fail");
    let report = DoctorReport {
        schema_version: 1,
        ok,
        checks,
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}

fn apply_fixes(checks: &[DoctorCheck], yes: bool) -> Result<()> {
    use dialoguer::Confirm;
    for c in checks {
        let Some(cmd) = &c.fix_command else { continue };
        let label = cmd.join(" ");
        let run_it = if yes {
            true
        } else {
            Confirm::new()
                .with_prompt(format!("Fix [{}]: run `{label}`?", c.id))
                .default(true)
                .interact()?
        };
        if !run_it {
            continue;
        }
        if c.id == "disk_space" || c.id == "workspace_writable" || c.id == "cpp_toolchain" {
            println!("  → {}", c.detail.as_deref().unwrap_or(&label));
            continue;
        }
        let st = Command::new(&cmd[0]).args(&cmd[1..]).status();
        match st {
            Ok(s) if s.success() => println!("  ✓ Ran: {label}"),
            Ok(s) => println!("  ⚠ Command exit code {:?}: {label}", s.code()),
            Err(e) => println!("  ⚠ Could not run: {e}"),
        }
    }
    Ok(())
}

fn print_human(report: &DoctorReport) {
    println!("oclive doctor — environment diagnostics\n");
    for c in &report.checks {
        let icon = match c.status.as_str() {
            "ok" => "✅",
            "warn" => "⚠️",
            _ => "❌",
        };
        println!("{icon} [{}] {}", c.id, c.message);
        if let Some(d) = &c.detail {
            println!("    {d}");
        }
    }
    println!();
    if report.ok {
        println!("Summary: environment ready; you can run oclive init / cargo build.");
    } else {
        println!("Summary: fix failed checks before initializing a kernel project.");
    }
}

fn run_capture(cmd: &mut Command) -> Option<String> {
    let o = cmd.output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_rust_toolchain() -> DoctorCheck {
    if let Some(ver) = run_capture(Command::new("rustc").arg("--version")) {
        if let Some((maj, min)) = parse_rustc_version(&ver) {
            if maj < 1 || (maj == 1 && min < 70) {
                return DoctorCheck {
                    id: "rust_toolchain".into(),
                    status: "warn".into(),
                    message: format!("{ver} — Rust 1.70+ recommended"),
                    detail: Some("Run: rustup update stable".into()),
                    fix_command: Some(vec!["rustup".into(), "update".into(), "stable".into()]),
                };
            }
        }
        return DoctorCheck::ok("rust_toolchain", ver);
    }
    if let Some(show) = run_capture(Command::new("rustup").arg("show").arg("active-toolchain")) {
        let line = show.lines().next().unwrap_or(&show).trim();
        return DoctorCheck::ok("rust_toolchain", format!("rustup: {line}"));
    }
    DoctorCheck {
        id: "rust_toolchain".into(),
        status: "fail".into(),
        message: "Rust toolchain not found (rustc / rustup)".into(),
        detail: Some("Install from https://rustup.rs/".into()),
        fix_command: None,
    }
}

fn parse_rustc_version(ver: &str) -> Option<(u32, u32)> {
    let part = ver.split_whitespace().nth(1)?;
    let mut it = part.split('.');
    let maj: u32 = it.next()?.parse().ok()?;
    let min: u32 = it.next()?.parse().ok()?;
    Some((maj, min))
}

fn check_cargo() -> DoctorCheck {
    if let Some(ver) = run_capture(Command::new("cargo").arg("--version")) {
        DoctorCheck::ok("cargo", ver)
    } else {
        DoctorCheck {
            id: "cargo".into(),
            status: "fail".into(),
            message: "cargo not found".into(),
            detail: if cfg!(windows) {
                Some("Install Rust: https://rustup.rs/ or Visual Studio Build Tools".into())
            } else {
                Some(
                    "Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                        .into(),
                )
            },
            fix_command: None,
        }
    }
}

fn check_cpp_toolchain() -> DoctorCheck {
    if cfg!(windows) {
        if run_capture(&mut Command::new("cl")).is_some() {
            return DoctorCheck::ok("cpp_toolchain", "MSVC cl available");
        }
        return DoctorCheck {
            id: "cpp_toolchain".into(),
            status: "warn".into(),
            message: "MSVC cl not found (some native deps may need it)".into(),
            detail: Some(
                "Install Visual Studio Build Tools (C++ workload) or: rustup default stable-msvc"
                    .into(),
            ),
            fix_command: None,
        };
    }
    if run_capture(Command::new("cc").arg("--version")).is_some()
        || run_capture(Command::new("g++").arg("--version")).is_some()
    {
        return DoctorCheck::ok("cpp_toolchain", "C/C++ compiler available");
    }
    DoctorCheck {
        id: "cpp_toolchain".into(),
        status: "warn".into(),
        message: "cc/g++ not found".into(),
        detail: Some(
            "Linux: sudo apt install build-essential · macOS: xcode-select --install".into(),
        ),
        fix_command: None,
    }
}

fn check_system_memory() -> DoctorCheck {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory() / (1024 * 1024);
    let avail = sys.available_memory() / (1024 * 1024);
    let msg = format!("{total} MiB total, {avail} MiB available");
    if total < 4096 {
        DoctorCheck::warn(
            "system_memory",
            format!("{msg} — below 4 GiB recommended"),
            Some("Dual Monolith release builds can use a lot of RAM".into()),
        )
    } else {
        DoctorCheck::ok("system_memory", msg)
    }
}

fn check_disk_space(path: &Path) -> DoctorCheck {
    match fs2::available_space(path) {
        Ok(bytes) => {
            let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let msg = format!("{gb:.1} GiB free ({})", path.display());
            if bytes < 1024 * 1024 * 1024 {
                DoctorCheck {
                    id: "disk_space".into(),
                    status: "warn".into(),
                    message: format!("{msg} — below 1 GiB"),
                    detail: Some(
                        "Suggestion: clean target/, bench_results/, or free disk space (cannot auto-delete)".into(),
                    ),
                    fix_command: Some(vec!["echo".into(), "manual_cleanup".into()]),
                }
            } else {
                DoctorCheck::ok("disk_space", msg)
            }
        }
        Err(e) => DoctorCheck::warn(
            "disk_space",
            "Could not read available disk space",
            Some(e.to_string()),
        ),
    }
}

fn check_ollama() -> DoctorCheck {
    let agent = crate::http_client::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .build();
    match agent.get("http://127.0.0.1:11434/api/tags").call() {
        Ok(resp) => {
            if resp.status() != 200 {
                return DoctorCheck::fail(
                    "ollama",
                    format!("Ollama returned HTTP {}", resp.status()),
                    None,
                );
            }
            let body = resp.into_string().unwrap_or_default();
            let n = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v.get("models").and_then(|m| m.as_array()).map(|a| a.len()))
                .unwrap_or(0);
            DoctorCheck::ok("ollama", format!("running, {n} model(s)"))
        }
        Err(e) => DoctorCheck {
            id: "ollama".into(),
            status: "fail".into(),
            message: "Ollama not running or unreachable (127.0.0.1:11434)".into(),
            detail: Some(format!("{e}; safe to ignore if using remote LLM only")),
            fix_command: if cfg!(windows) {
                Some(vec![
                    "powershell".into(),
                    "-Command".into(),
                    "Start-Process ollama -ArgumentList serve".into(),
                ])
            } else {
                Some(vec!["sh".into(), "-c".into(), "ollama serve &".into()])
            },
        },
    }
}

fn check_network_github() -> DoctorCheck {
    let agent = crate::http_client::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();
    match agent.get("https://github.com").call() {
        Ok(resp) if resp.status() == 200 => {
            DoctorCheck::ok("network", "https://github.com reachable")
        }
        Ok(resp) => DoctorCheck::warn(
            "network",
            format!("GitHub returned HTTP {}", resp.status()),
            None,
        ),
        Err(e) => DoctorCheck::fail(
            "network",
            "Cannot reach https://github.com",
            Some(e.to_string()),
        ),
    }
}

fn run_watch(root: &Path, json: bool) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::{stdout, Write};
    use std::time::{Duration, Instant};

    if json {
        anyhow::bail!("--watch does not support --json; omit --json");
    }
    enable_raw_mode().context("enable_raw_mode")?;
    let mut last_ollama_ok = check_ollama().status == "ok";
    let mut _last_disk_gb = disk_free_gb(root);
    let mut _last_mem_mb = sys_available_mb();

    eprintln!("oclive doctor --watch (every 60s, press q to quit)\n");
    loop {
        let checks = vec![
            check_disk_space(root),
            check_system_memory(),
            check_ollama(),
        ];
        let ollama_ok = checks[2].status == "ok";
        let disk_gb = disk_free_gb(root);
        let mem_mb = sys_available_mb();

        let mut alerts = Vec::new();
        if let Some(gb) = disk_gb {
            if gb < 1.0 {
                alerts.push(format!("WARN disk free {gb:.2} GiB (< 1 GiB)"));
            }
        }
        if let Some(mb) = mem_mb {
            if mb < 500 {
                alerts.push(format!("WARN available memory {mb} MiB (< 500 MiB)"));
            }
        }
        if last_ollama_ok && !ollama_ok {
            alerts.push("WARN Ollama stopped or unreachable".into());
        }

        print!("\x1b[2J\x1b[H");
        let _ = stdout().flush();
        println!("oclive doctor --watch — {}", root.display());
        println!("Updated: {}\n", chrono_lite_now());
        for c in &checks {
            let icon = match c.status.as_str() {
                "ok" => "OK",
                "warn" => "WARN",
                _ => "FAIL",
            };
            println!("  [{icon}] {} — {}", c.id, c.message);
        }
        for a in &alerts {
            println!("\n  ⚠ {a}");
        }
        println!("\nPress q to quit.");

        last_ollama_ok = ollama_ok;
        _last_disk_gb = disk_gb;
        _last_mem_mb = mem_mb;

        let until = Instant::now() + Duration::from_secs(60);
        while Instant::now() < until {
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(k) = event::read()? {
                    if k.kind == KeyEventKind::Press && k.code == KeyCode::Char('q') {
                        disable_raw_mode().ok();
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn disk_free_gb(path: &Path) -> Option<f64> {
    fs2::available_space(path)
        .ok()
        .map(|b| b as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn sys_available_mb() -> Option<u64> {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    Some(sys.available_memory() / (1024 * 1024))
}

fn check_workspace_writable(path: &Path) -> DoctorCheck {
    let probe_dir = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    fs::create_dir_all(&probe_dir).ok();
    let probe = probe_dir.join(".oclive_doctor_probe");
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            DoctorCheck::ok("workspace_writable", format!("{} is writable", probe_dir.display()))
        }
        Err(e) => DoctorCheck {
            id: "workspace_writable".into(),
            status: "fail".into(),
            message: format!("{} is not writable", probe_dir.display()),
            detail: Some(format!(
                "{e}; check permissions or use a writable path (Windows: run as admin or adjust ACL)"
            )),
            fix_command: Some(vec!["echo".into(), "fix_permissions".into()]),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_report_serializes() {
        let r = DoctorReport {
            schema_version: 1,
            ok: true,
            checks: vec![DoctorCheck::ok("cargo", "cargo 1.85.0")],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert!(v["checks"].is_array());
    }
}
