//! `test --miri` via cargo-miri.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn run_miri(root: &Path, only_crate: Option<&str>) -> Result<()> {
    if !Command::new("cargo-miri")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        bail!("cargo-miri not installed. Install: rustup component add miri-preview -p nightly && cargo install cargo-miri");
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("miri").arg("test");
    if let Some(k) = only_crate {
        cmd.args(["-p", k]);
    } else {
        cmd.arg("--workspace");
    }
    cmd.current_dir(root);
    println!("oclive test --miri — {}", root.display());
    let st = cmd.status()?;
    if st.success() {
        println!("Miri: PASS");
        Ok(())
    } else {
        bail!("Miri tests failed (exit {:?})", st.code());
    }
}
