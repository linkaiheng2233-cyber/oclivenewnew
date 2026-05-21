//! `test --coverage` via cargo llvm-cov.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn run_coverage(root: &Path, open: bool) -> Result<()> {
    if Command::new("cargo-llvm-cov")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        // ok
    } else {
        bail!("cargo-llvm-cov not installed. Install: cargo install cargo-llvm-cov");
    }

    let st = Command::new("cargo")
        .args([
            "llvm-cov",
            "--workspace",
            "--html",
            "--output-dir",
            "target/llvm-cov/html",
        ])
        .current_dir(root)
        .status()?;
    if !st.success() {
        bail!("cargo llvm-cov failed (exit {:?})", st.code());
    }

    let summary = Command::new("cargo")
        .args(["llvm-cov", "--workspace", "--summary-only"])
        .current_dir(root)
        .output()?;
    let text = String::from_utf8_lossy(&summary.stdout);
    println!("oclive test --coverage — {}", root.display());
    println!("Report: {}/target/llvm-cov/html/index.html", root.display());
    for line in text.lines() {
        if line.contains('%') || line.contains("TOTAL") {
            println!("  {line}");
        }
    }

    if open {
        let index = root.join("target/llvm-cov/html/index.html");
        if index.is_file() {
            open_in_browser(&index)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn open_in_browser(path: &Path) -> Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", &path.display().to_string()])
        .status()?;
    Ok(())
}

#[cfg(not(windows))]
fn open_in_browser(path: &Path) -> Result<()> {
    Command::new("xdg-open").arg(path).status()?;
    Ok(())
}
