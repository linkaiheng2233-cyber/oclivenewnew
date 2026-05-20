//! `oclive test` — 内核工程回归检查。

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
pub struct TestArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub json: bool,
    /// Skip OOCP protocol black-box tests (slow)
    #[arg(long)]
    pub skip_oocp: bool,

    /// Run jobs aligned with `.github/workflows/ci.yml` locally
    #[arg(long)]
    pub ci_parity: bool,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    ok: bool,
    detail: String,
}

pub fn run(args: TestArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path.clone());
    if args.ci_parity {
        return crate::test_ci_parity::run_ci_parity(&root, args.skip_oocp, args.json);
    }
    let mut checks = Vec::new();

    checks.push(run_cargo_check(&root));
    checks.push(run_clippy(&root));
    checks.push(run_pack_validate_all(&root));

    if !args.skip_oocp {
        checks.push(run_oocp_hint(&root));
    } else {
        checks.push(CheckResult {
            name: "oocp".into(),
            ok: true,
            detail: "skipped (--skip-oocp)".into(),
        });
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        println!("oclive test — {}", root.display());
        let mut ok_all = true;
        for c in &checks {
            let mark = if c.ok { "✅" } else { "❌" };
            println!("  {mark} {} — {}", c.name, c.detail);
            ok_all &= c.ok;
        }
        println!(
            "\n{}",
            if ok_all {
                "All checks passed"
            } else {
                "Some checks failed"
            }
        );
        if !ok_all {
            bail!("test did not pass all checks");
        }
    }
    Ok(())
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
        if entry.join("manifest.json").is_file() {
            n += 1;
            let st = Command::new(std::env::current_exe().unwrap_or_default())
                .args([
                    "pack",
                    "validate",
                    entry.to_str().unwrap_or("."),
                ])
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
    }
}

fn fail(name: &str, detail: impl ToString) -> CheckResult {
    CheckResult {
        name: name.into(),
        ok: false,
        detail: detail.to_string(),
    }
}
