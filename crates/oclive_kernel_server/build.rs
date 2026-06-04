//! Embed build metadata for `KernelBinaryManifest` / `--version-json`.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.2.0".into());
    println!("cargo:rustc-env=OCLIVE_KERNEL_PKG_VERSION={pkg_version}");
    println!("cargo:rustc-env=OCLIVE_KERNEL_BUILD_PROFILE=full");

    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=OCLIVE_KERNEL_BUILT_AT={built_at}");

    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--short=HEAD"])
        .output()
    {
        if out.status.success() {
            let commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !commit.is_empty() {
                println!("cargo:rustc-env=OCLIVE_KERNEL_GIT_COMMIT={commit}");
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
}
