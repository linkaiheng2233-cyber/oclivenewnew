//! `test --loom` — run Loom model tests in the workspace.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn run_loom(root: &Path) -> Result<()> {
    if !Command::new("cargo")
        .args(["loom", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        bail!(
            "cargo-loom not installed. Install: cargo install cargo-loom --locked"
        );
    }
    println!("oclive test --loom — {}", root.display());
    let st = Command::new("cargo")
        .args([
            "loom",
            "test",
            "-p",
            "oclivenewnew-tauri",
            "--",
            "loom_concurrency",
        ])
        .current_dir(root)
        .status()?;
    if st.success() {
        println!("Loom: PASS");
        Ok(())
    } else {
        bail!("loom tests failed (exit {:?})", st.code());
    }
}
