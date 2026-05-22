//! `oclive test` — 内核工程回归检查。

use anyhow::{bail, Result};
use clap::Parser;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Parser, Debug)]
pub struct TestArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
    /// Skip OOCP protocol black-box tests (slow)
    #[arg(long)]
    pub skip_oocp: bool,

    /// Start kernel (`cargo run --release -- --api`), run OOCP S0–S11, then stop (oclivenewnew root only)
    #[arg(long)]
    pub oocp: bool,

    /// Run jobs aligned with `.github/workflows/ci.yml` locally
    #[arg(long)]
    pub ci_parity: bool,

    /// Generate HTML coverage via cargo llvm-cov
    #[arg(long)]
    pub coverage: bool,

    /// Open coverage report in browser after generation
    #[arg(long)]
    pub open: bool,

    /// Run Miri undefined-behavior tests
    #[arg(long)]
    pub miri: bool,

    /// Miri: test only this crate (`-p`)
    #[arg(long = "miri-only")]
    pub miri_only: Option<String>,

    /// Loom concurrency model tests (`cargo-loom` required)
    #[arg(long)]
    pub loom: bool,

    /// With `test`: run Monolith equivalence via `bench --equivalence` when `monolith.toml` exists
    #[arg(long)]
    pub equivalence_check: bool,
}

#[derive(Serialize)]
pub(crate) struct CheckResult {
    pub(crate) name: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<u64>,
}

pub fn run(args: TestArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path.clone());
    if args.coverage {
        return crate::test_coverage::run_coverage(&root, args.open);
    }
    if args.loom {
        return crate::test_loom::run_loom(&root);
    }
    if args.miri {
        return crate::test_miri::run_miri(&root, args.miri_only.as_deref());
    }
    if args.equivalence_check {
        return crate::test_equivalence_check::run(&root);
    }
    if args.ci_parity {
        return crate::test_ci_parity::run_ci_parity(&root, args.skip_oocp, args.json);
    }
    if args.oocp {
        return crate::test_oocp::run_oocp_integration(&root);
    }
    let t0 = Instant::now();
    let mut checks = Vec::new();

    checks.push(timed(|| run_cargo_check(&root)));
    checks.push(timed(|| run_clippy(&root)));
    checks.push(timed(|| run_pack_validate_all(&root)));

    if !args.skip_oocp {
        checks.push(timed(|| run_oocp_hint(&root)));
    } else {
        checks.push(CheckResult {
            name: "oocp".into(),
            ok: true,
            detail: "skipped (--skip-oocp)".into(),
            duration_ms: Some(0),
        });
    }

    let elapsed = t0.elapsed();
    if args.json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        crate::test_report::print_human_report(&root, &checks, elapsed);
        if checks.iter().any(|c| !c.ok) {
            bail!("test did not pass all checks");
        }
    }
    Ok(())
}

fn timed(f: impl FnOnce() -> CheckResult) -> CheckResult {
    let t = Instant::now();
    let mut c = f();
    c.duration_ms = Some(t.elapsed().as_millis() as u64);
    c
}

fn run_cargo_check(root: &Path) -> CheckResult {
    let st = Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .status();
    match st {
        Ok(s) if s.success() => ok("cargo check", "passed"),
        Ok(s) => fail("cargo check", format!("exit code {:?}", s.code())),
        Err(e) => fail("cargo check", e.to_string()),
    }
}

fn run_clippy(root: &Path) -> CheckResult {
    let st = Command::new("cargo")
        .args([
            "clippy",
            "--manifest-path",
            root.join("Cargo.toml").to_str().unwrap_or("Cargo.toml"),
            "--",
            "-D",
            "warnings",
        ])
        .status();
    match st {
        Ok(s) if s.success() => ok("clippy", "passed"),
        Ok(s) => fail("clippy", format!("exit code {:?}", s.code())),
        Err(e) => fail("clippy", format!("cannot start cargo: {e}")),
    }
}

fn run_pack_validate_all(root: &Path) -> CheckResult {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return ok("pack validate", "no roles/ directory; skipped");
    }
    let mut n = 0u32;
    let mut fail_n = 0u32;
    for entry in walk_dirs(&roles) {
        if entry.join("manifest.json").is_file()
            || entry
                .join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME)
                .is_file()
        {
            n += 1;
            let mut args = vec![
                "pack".to_string(),
                "validate".to_string(),
                entry.to_str().unwrap_or(".").to_string(),
            ];
            if entry.join("manifest.json").is_file() {
                args.push("--profile".into());
                args.push("legacy".into());
            }
            let st = Command::new(std::env::current_exe().unwrap_or_default())
                .args(&args)
                .status();
            if !matches!(st, Ok(s) if s.success()) {
                fail_n += 1;
            }
        }
    }
    if fail_n > 0 {
        fail("pack validate", format!("{fail_n}/{n} role packs failed"))
    } else {
        ok("pack validate", format!("{n} role packs"))
    }
}

fn run_oocp_hint(root: &Path) -> CheckResult {
    let script = find_oocp_runner();
    let Some(script) = script else {
        return ok(
            "oocp",
            "examples/oocp-test-suite/run.mjs not found (use oclivenewnew root or set OCLIVE_ROOT)",
        );
    };
    ok(
        "oocp",
        format!(
            "Start kernel HTTP locally, then run: node {} (project: {})",
            script.display(),
            root.display()
        ),
    )
}

fn find_oocp_runner() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OCLIVE_ROOT") {
        let s = PathBuf::from(p).join("examples/oocp-test-suite/run.mjs");
        if s.is_file() {
            return Some(s);
        }
    }
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..6 {
        let cand = dir.join("examples/oocp-test-suite/run.mjs");
        if cand.is_file() {
            return Some(cand);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn walk_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.push(p.clone());
                    stack.push(p);
                }
            }
        }
    }
    out
}

fn ok(name: &str, detail: impl ToString) -> CheckResult {
    CheckResult {
        name: name.into(),
        ok: true,
        detail: detail.to_string(),
        duration_ms: None,
    }
}

fn fail(name: &str, detail: impl ToString) -> CheckResult {
    CheckResult {
        name: name.into(),
        ok: false,
        detail: detail.to_string(),
        duration_ms: None,
    }
}
