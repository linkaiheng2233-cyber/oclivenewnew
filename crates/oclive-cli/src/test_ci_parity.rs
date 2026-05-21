//! `test --ci-parity` — run local checks aligned with generated CI workflow.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Serialize)]
struct JobResult {
    name: String,
    ok: bool,
    elapsed_ms: u64,
    detail: String,
}

pub fn run_ci_parity(root: &Path, skip_oocp: bool, json: bool) -> Result<()> {
    let workflow = root.join(".github/workflows/ci.yml");
    let jobs = if workflow.is_file() {
        parse_ci_jobs(&workflow)?
    } else {
        default_jobs(skip_oocp)
    };

    let mut results = Vec::new();
    let mut ok_all = true;
    for job in jobs {
        let t0 = Instant::now();
        let (ok, detail) = run_job(root, &job)?;
        let elapsed = t0.elapsed().as_millis() as u64;
        ok_all &= ok;
        results.push(JobResult {
            name: job.clone(),
            ok,
            elapsed_ms: elapsed,
            detail,
        });
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("oclive test --ci-parity — {}", root.display());
        for r in &results {
            let mark = if r.ok { "PASS" } else { "FAIL" };
            println!("  [{mark}] {} ({:.2}s) — {}", r.name, r.elapsed_ms as f64 / 1000.0, r.detail);
        }
        println!(
            "\n{}",
            if ok_all {
                "All CI parity jobs passed."
            } else {
                "One or more jobs failed."
            }
        );
    }
    if !ok_all {
        bail!("CI parity check failed");
    }
    Ok(())
}

fn default_jobs(skip_oocp: bool) -> Vec<String> {
    let mut v = vec![
        "cargo_fmt".into(),
        "cargo_clippy".into(),
        "cargo_build".into(),
        "cargo_test".into(),
        "oclive_test".into(),
        "bench_regression".into(),
    ];
    if !skip_oocp {
        v.push("oocp_hint".into());
    }
    v
}

fn parse_ci_jobs(path: &Path) -> Result<Vec<String>> {
    let raw = std::fs::read_to_string(path)?;
    let mut jobs = Vec::new();
    if raw.contains("cargo fmt") {
        jobs.push("cargo_fmt".into());
    }
    if raw.contains("cargo clippy") {
        jobs.push("cargo_clippy".into());
    }
    if raw.contains("cargo build") {
        jobs.push("cargo_build".into());
    }
    if raw.contains("cargo test") {
        jobs.push("cargo_test".into());
    }
    if raw.contains("oclive test") || raw.contains("oclive-cli -- test") {
        jobs.push("oclive_test".into());
    }
    if raw.contains("bench --release") && raw.contains("regression") {
        jobs.push("bench_regression".into());
    }
    if raw.contains("oocp") || raw.contains("OOCP") {
        jobs.push("oocp_hint".into());
    }
    if jobs.is_empty() {
        return Ok(default_jobs(false));
    }
    Ok(jobs)
}

fn run_job(root: &Path, job: &str) -> Result<(bool, String)> {
    let manifest = root.join("Cargo.toml");
    let _m = manifest.to_str().unwrap_or("Cargo.toml");
    match job {
        "cargo_fmt" => {
            let st = Command::new("cargo")
                .args(["fmt", "--all", "--", "--check"])
                .current_dir(root)
                .status()?;
            Ok((st.success(), format!("exit {:?}", st.code())))
        }
        "cargo_clippy" => {
            let st = Command::new("cargo")
                .args(["clippy", "--all-targets", "--", "-D", "warnings"])
                .current_dir(root)
                .status()?;
            Ok((st.success(), format!("exit {:?}", st.code())))
        }
        "cargo_build" => {
            let st = Command::new("cargo")
                .args(["build", "--release"])
                .current_dir(root)
                .status()?;
            Ok((st.success(), format!("exit {:?}", st.code())))
        }
        "cargo_test" => {
            let st = Command::new("cargo").args(["test"]).current_dir(root).status()?;
            Ok((st.success(), format!("exit {:?}", st.code())))
        }
        "oclive_test" => {
            let exe = std::env::current_exe().context("current_exe")?;
            let st = Command::new(&exe)
                .args(["test", "-o", root.to_str().unwrap_or(".")])
                .arg("--skip-oocp")
                .status()?;
            Ok((st.success(), format!("exit {:?}", st.code())))
        }
        "bench_regression" => {
            let exe = std::env::current_exe().context("current_exe")?;
            let _ = Command::new(&exe)
                .args([
                    "bench",
                    "--release",
                    "-o",
                    root.to_str().unwrap_or("."),
                    "--runs",
                    "3",
                    "--save",
                ])
                .status();
            let st = Command::new(&exe)
                .args([
                    "bench",
                    "--release",
                    "-o",
                    root.to_str().unwrap_or("."),
                    "--regression",
                    "--runs",
                    "5",
                ])
                .status()?;
            Ok((
                st.success(),
                "bench save + regression (matches CI continue-on-error semantics)".into(),
            ))
        }
        "oocp_hint" => Ok((
            true,
            "OOCP requires running kernel HTTP — use --skip-oocp to skip this hint job".into(),
        )),
        other => Ok((true, format!("unknown job `{other}`, skipped"))),
    }
}
