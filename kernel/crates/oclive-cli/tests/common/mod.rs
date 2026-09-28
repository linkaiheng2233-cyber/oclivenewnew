//! Shared helpers for `oclive-cli` integration tests.

#![allow(dead_code)]

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/oclive-cli")
        .parent()
        .expect("kernel/crates")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

pub fn cli_command() -> Command {
    // Use the binary Cargo selected for this test invocation, including its active features.
    // Real builds of generated projects stay in the cargo_build helpers below.
    let mut command = Command::new(env!("CARGO_BIN_EXE_oclive-cli"));
    command.current_dir(repo_root());
    command
}

pub fn run_cli_output(args: &[&str]) -> std::process::Output {
    cli_command()
        .args(["--experimental"])
        .args(args)
        .output()
        .expect("run test-built oclive-cli")
}

pub fn run_cli(args: &[&str]) -> std::process::ExitStatus {
    cli_command()
        .args(["--experimental"])
        .args(args)
        .status()
        .expect("run test-built oclive-cli")
}

pub fn assert_bench_report_matches_schema(v: &Value) {
    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(2));
    assert!(v.get("package_name").and_then(|x| x.as_str()).is_some());
    assert!(v.get("runs").and_then(|x| x.as_u64()).is_some());
    assert!(v.get("inner_iters").and_then(|x| x.as_u64()).is_some());
    assert_eq!(v.get("release").and_then(|x| x.as_bool()), Some(true));
    let check_stats = |key: &str| {
        let o = v.get(key).and_then(|x| x.as_object()).expect(key);
        for k in ["min", "max", "p50", "p95", "mean"] {
            assert!(
                o.get(k).and_then(|x| x.as_f64()).is_some(),
                "{key}.{k} must be number"
            );
        }
        let samples = o
            .get("samples")
            .and_then(|x| x.as_array())
            .expect("samples");
        assert!(!samples.is_empty(), "{key}.samples");
        for s in samples {
            assert!(s.as_f64().is_some(), "sample must be number");
        }
    };
    check_stats("standard_ms");
    check_stats("monolith_ms");
    for key in ["binary_size", "peak_memory", "build_time"] {
        let o = v.get(key).and_then(|x| x.as_object()).expect(key);
        assert!(o.get("standard").is_some());
        assert!(o.get("monolith").is_some());
    }
}

pub fn cargo_build(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build")
}

pub fn cargo_build_release(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release")
}

pub fn cargo_build_release_monolith(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release", "--features", "monolith"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release --features monolith")
}

pub fn release_binary_path(project_dir: &std::path::Path, bin_base: &str) -> std::path::PathBuf {
    let p = project_dir.join("target/release").join(bin_base);
    if cfg!(windows) {
        p.with_extension("exe")
    } else {
        p
    }
}
