//! `doctor --sbom` — CycloneDX / SPDX bill of materials via cargo-cyclonedx.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn run_sbom(root: &Path, format: &str) -> Result<()> {
    if !root.join("Cargo.toml").is_file() {
        bail!("missing Cargo.toml at {}", root.display());
    }
    let fmt = format.to_ascii_lowercase();
    if fmt != "cyclonedx" && fmt != "spdx" {
        bail!("--sbom-format must be cyclonedx or spdx (got {format})");
    }

    if !Command::new("cargo")
        .args(["cyclonedx", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        bail!(
            "cargo-cyclonedx not installed. Install: cargo install cargo-cyclonedx"
        );
    }

    let out_file = if fmt == "spdx" {
        "sbom.spdx.json"
    } else {
        "sbom.json"
    };
    let mut cmd = Command::new("cargo");
    cmd.arg("cyclonedx")
        .arg(if fmt == "spdx" { "spdx" } else { "cyclonedx" })
        .args(["--output-format", "json", "--output-file", out_file])
        .current_dir(root);
    let st = cmd.status()?;
    if !st.success() {
        bail!("cargo cyclonedx failed (exit {:?})", st.code());
    }

    let output = root.join(out_file);
    let dep_count = count_deps_metadata(root);
    let licenses = license_histogram(root);

    println!("oclive doctor --sbom — {}", root.display());
    println!("  format: {fmt}");
    println!("  output: {}", output.display());
    if let Some(n) = dep_count {
        println!("  dependencies (cargo metadata packages): {n}");
    }
    if !licenses.is_empty() {
        println!("  license distribution (top entries):");
        for (lic, c) in licenses.iter().take(8) {
            println!("    {lic}: {c}");
        }
    }
    println!("  vulnerability scan: run `oclive lint --deps` or `cargo audit`");
    if !output.is_file() {
        println!("  note: expected file not found at {}; check cargo-cyclonedx CLI output path", output.display());
    }
    Ok(())
}

fn count_deps_metadata(root: &Path) -> Option<u32> {
    let o = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .ok()?;
    if !o.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).ok()?;
    v.get("packages")
        .and_then(|p| p.as_array())
        .map(|a| a.len() as u32)
}

fn license_histogram(root: &Path) -> Vec<(String, u32)> {
    let o = match Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(root)
        .output()
    {
        Ok(x) if x.status.success() => x,
        _ => return Vec::new(),
    };
    let v: serde_json::Value = match serde_json::from_slice(&o.stdout) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut buckets: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    if let Some(pkgs) = v.get("packages").and_then(|p| p.as_array()) {
        for pkg in pkgs {
            let lic = pkg
                .get("license")
                .and_then(|l| l.as_str())
                .unwrap_or("unknown")
                .to_string();
            *buckets.entry(lic).or_insert(0) += 1;
        }
    }
    let mut v: Vec<_> = buckets.into_iter().collect();
    v.sort_by_key(|b| std::cmp::Reverse(b.1));
    v
}
