//! Embed build metadata for `KernelBinaryManifest` / `--version-json`.

use std::process::Command;

/// Reproducible build timestamp: `SOURCE_DATE_EPOCH` when set (seconds since UNIX epoch), else 0.
fn source_date_epoch_secs() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn main() {
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.2.0".into());
    println!("cargo:rustc-env=OCLIVE_KERNEL_PKG_VERSION={pkg_version}");
    println!("cargo:rustc-env=OCLIVE_KERNEL_BUILD_PROFILE=full");

    let built_at = source_date_epoch_secs();
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

    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let git_head = manifest_dir.join("../../.git/HEAD");
    if git_head.is_file() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
        let refs_heads = manifest_dir.join("../../.git/refs/heads");
        if refs_heads.is_dir() {
            println!("cargo:rerun-if-changed=../../.git/refs/heads");
        }
    }
}
