//! `test --loom` — run Loom model tests in the workspace.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn run_loom(root: &Path) -> Result<()> {
    println!("oclive test --loom — {}", root.display());
    let st = Command::new("cargo")
        .args([
            "test",
            "--locked",
            "--release",
            "-p",
            "oclivenewnew-tauri",
            "--test",
            "loom_concurrency",
            "--features",
            "loom-tests",
            "--",
            "--test-threads=1",
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
