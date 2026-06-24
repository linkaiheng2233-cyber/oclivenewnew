//! End-to-end: non-interactively generate a project and `cargo build` it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
use common::*;

use serde_json::Value;
use std::fs;
use std::process::Command;

#[test]
fn e2e_non_interactive_monolith_full_release_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_rel");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success(), "oclive-cli init --monolith");
    let mt_path = out.join("monolith.toml");
    assert!(mt_path.is_file());
    let mt = fs::read_to_string(&mt_path).unwrap();
    assert!(mt.contains("enabled = true"));
    assert!(mt.contains("weld_modules = ["));
    assert!(mt.contains("\"memory\""));
    assert!(mt.contains("exclude = []"));
    assert!(out.join("src/process_message_monolith.rs").is_file());
    let mono_rs = fs::read_to_string(out.join("src/process_message_monolith.rs")).unwrap();
    assert!(
        mono_rs.contains("oclive_monolith_builtin::memory::invoke"),
        "expected static builtin call in monolith source"
    );
    assert!(
        !mono_rs.contains("super::dynamic_plugin_host::trait_dispatch_memory"),
        "full weld should not dispatch memory via trait stub"
    );
    let cargo_toml = fs::read_to_string(out.join("Cargo.toml")).unwrap();
    assert!(
        cargo_toml.contains("src/main_monolith.rs"),
        "Monolith bin should use dedicated main_monolith.rs"
    );
    assert!(out.join("src/main_monolith.rs").is_file());
    assert!(
        cargo_build_release(&out).success(),
        "standard release build"
    );
    assert!(
        cargo_build_release_monolith(&out).success(),
        "monolith release build"
    );
    let slug = "my-oclive-kernel";
    let std_bin = release_binary_path(&out, slug);
    let mono_bin = release_binary_path(&out, &format!("{slug}-monolith"));
    assert!(std_bin.is_file(), "expected {:?}", std_bin);
    assert!(mono_bin.is_file(), "expected {:?}", mono_bin);
}

#[test]
fn e2e_monolith_ignored_for_library() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_lib");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "--project-type",
        "library",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(
        !out.join("monolith.toml").exists(),
        "library project must not emit monolith.toml"
    );
}

#[test]
fn e2e_build_custom_weld_modules_then_compiles() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_custom");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let mt = r#"[monolith]
enabled = true
weld_modules = ["memory", "emotion"]
exclude = []
"#;
    fs::write(out.join("monolith.toml"), mt).unwrap();
    assert!(
        run_cli(&["build", "-o", out.to_str().unwrap(), "--no-cargo"]).success(),
        "oclive build --no-cargo"
    );
    let rs = fs::read_to_string(out.join("src/process_message_monolith.rs")).unwrap();
    assert!(rs.contains("oclive_monolith_builtin::memory::invoke"));
    assert!(rs.contains("trait_dispatch_llm"));
    assert!(
        cargo_build_release(&out).success() && cargo_build_release_monolith(&out).success(),
        "custom weld project compiles"
    );
}

#[test]
fn e2e_build_rejects_weld_and_exclude_together() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_bad");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let mt = r#"[monolith]
enabled = true
weld_modules = ["memory"]
exclude = ["agent"]
"#;
    fs::write(out.join("monolith.toml"), mt).unwrap();
    assert!(
        !run_cli(&["build", "-o", out.to_str().unwrap(), "--no-cargo"]).success(),
        "conflicting weld_modules + exclude must fail"
    );
}

#[test]
fn e2e_build_when_monolith_disabled_skips_second_cargo_build() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_off");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let mt = r#"[monolith]
enabled = false
weld_modules = []
exclude = []
"#;
    fs::write(out.join("monolith.toml"), mt).unwrap();
    let o = run_cli_output(&["build", "-o", out.to_str().unwrap()]);
    assert!(o.status.success(), "build should succeed");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stderr),
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(
        combined.contains("skipping Monolith build"),
        "expected skip message: {combined}"
    );
}

#[test]
fn e2e_bench_smoke_json() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_bench");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let report_path = tmp.path().join("report.json");
    let st = Command::new("cargo")
        .current_dir(repo_root())
        .args([
            "run",
            "-p",
            "oclive-cli",
            "--quiet",
            "--",
            "--experimental",
            "bench",
            "--release",
            "-o",
            out.to_str().unwrap(),
            "--runs",
            "2",
            "--inner-iters",
            "30",
            "--output",
            report_path.to_str().unwrap(),
        ])
        .status()
        .expect("bench");
    assert!(st.success(), "oclive bench smoke");
    let raw = fs::read_to_string(&report_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert!(v["standard_ms"]["p50"].is_number());
    assert!(v["monolith_ms"]["p50"].is_number());
    assert!(v["binary_size"]["standard"].is_number());
    assert!(v["peak_memory"]["monolith"].is_number());
}

#[test]
fn e2e_bench_json_schema_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_schema");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let o = run_cli_output(&[
        "bench",
        "--release",
        "-o",
        out.to_str().unwrap(),
        "--runs",
        "2",
        "--inner-iters",
        "20",
        "--json",
    ]);
    assert!(o.status.success(), "bench --json");
    let stdout = String::from_utf8_lossy(&o.stdout);
    let start = stdout.find('{').expect("stdout should contain JSON object");
    let end = stdout.rfind('}').map(|i| i + 1).expect("JSON end");
    let json_slice = &stdout[start..end];
    let v: Value = serde_json::from_str(json_slice).expect("parse JSON slice");
    assert_bench_report_matches_schema(&v);
}

#[test]
fn e2e_monolith_preset_latency_welds_all_slots() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "--monolith",
        "--monolith-preset",
        "latency",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    let mt = fs::read_to_string(out.join("monolith.toml")).unwrap();
    for slot in [
        "memory",
        "emotion",
        "event",
        "prompt",
        "llm",
        "agent",
        "complex_emotion",
    ] {
        assert!(mt.contains(slot), "monolith.toml should weld {slot}");
    }
}
