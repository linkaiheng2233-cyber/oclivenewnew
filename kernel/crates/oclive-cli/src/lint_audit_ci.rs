//! `lint --audit-ci` — verify cargo-audit job in GitHub Actions CI.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub fn run_audit_ci(start: &Path) -> Result<()> {
    let ci = find_ci_yml(start)?;
    let text = std::fs::read_to_string(&ci)?;
    println!("oclive lint --audit-ci");
    println!("  workflow: {}", ci.display());

    let has_job = text.contains("cargo-audit:") || text.contains("cargo-audit ");
    let has_continue =
        text.contains("continue-on-error: true") || text.contains("continue-on-error:true");

    if !has_job {
        println!("  [FAIL] No cargo-audit job found in ci.yml");
        println!("  Suggestion: add a cargo-audit job (see oclivenewnew .github/workflows/ci.yml)");
        bail!("cargo-audit job missing from CI workflow");
    }
    println!("  [PASS] cargo-audit job present");

    if has_continue {
        println!("  [WARN] cargo-audit uses continue-on-error: true");
        println!(
            "  Suggestion: set continue-on-error: false after dependency upgrades so high-severity advisories block merges"
        );
    } else {
        println!("  [PASS] cargo-audit is not continue-on-error (failures block CI)");
    }
    Ok(())
}

fn find_ci_yml(start: &Path) -> Result<PathBuf> {
    let mut dir = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    for _ in 0..12 {
        let cand = dir.join(".github/workflows/ci.yml");
        if cand.is_file() {
            return Ok(cand);
        }
        if !dir.pop() {
            break;
        }
    }
    bail!(
        "could not find .github/workflows/ci.yml from {}",
        start.display()
    );
}
