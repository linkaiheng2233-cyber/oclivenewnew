//! Embed the kernel manifest metadata at the crate that owns the manifest.

use std::path::{Path, PathBuf};
use std::process::Command;

fn git_output(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn build_timestamp(repo_root: &Path) -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .filter(|value| value.parse::<u64>().is_ok())
        .or_else(|| git_output(repo_root, &["log", "-1", "--format=%ct"]))
        .unwrap_or_else(|| "0".to_string())
}

fn watch_git_head(repo_root: &Path) {
    let Some(git_dir) =
        git_output(repo_root, &["rev-parse", "--absolute-git-dir"]).map(PathBuf::from)
    else {
        return;
    };
    let head = git_dir.join("HEAD");
    if head.is_file() {
        println!("cargo:rerun-if-changed={}", head.display());
    }
    if let Some(head_log) =
        git_output(repo_root, &["rev-parse", "--git-path", "logs/HEAD"]).map(PathBuf::from)
    {
        let head_log = if head_log.is_absolute() {
            head_log
        } else {
            repo_root.join(head_log)
        };
        if head_log.is_file() {
            println!("cargo:rerun-if-changed={}", head_log.display());
        }
    }
    if let Some(reference) = git_output(repo_root, &["symbolic-ref", "-q", "HEAD"])
        .and_then(|name| git_output(repo_root, &["rev-parse", "--git-path", &name]))
        .map(PathBuf::from)
    {
        let reference = if reference.is_absolute() {
            reference
        } else {
            repo_root.join(reference)
        };
        if reference.is_file() {
            println!("cargo:rerun-if-changed={}", reference.display());
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let repo_root = manifest_dir.join("../../..");
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.2.0".into());

    println!("cargo:rustc-env=OCLIVE_KERNEL_PKG_VERSION={pkg_version}");
    println!("cargo:rustc-env=OCLIVE_KERNEL_BUILD_PROFILE=full");
    println!(
        "cargo:rustc-env=OCLIVE_KERNEL_BUILT_AT={}",
        build_timestamp(&repo_root)
    );
    if let Some(commit) = git_output(&repo_root, &["rev-parse", "HEAD"]) {
        println!("cargo:rustc-env=OCLIVE_KERNEL_GIT_COMMIT={commit}");
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    watch_git_head(&repo_root);
}
