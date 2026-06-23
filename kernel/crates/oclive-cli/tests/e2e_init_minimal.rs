//! End-to-end: non-interactively generate a project and `cargo build` it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
use common::*;

use serde_json::Value;
use std::fs;

#[test]
fn e2e_preset_minimal_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k1");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success(), "oclive-cli init");
    assert!(out.join("CONFIG_REFERENCE.md").is_file());
    let cref = fs::read_to_string(out.join("CONFIG_REFERENCE.md")).expect("CONFIG_REFERENCE.md");
    assert!(
        cref.contains("开发者编译选项"),
        "CONFIG_REFERENCE should mention planned developer compile options"
    );
    assert!(
        cref.contains("RFC_OCLIVE_MONOLITH_MODE"),
        "CONFIG_REFERENCE should link Monolith RFC"
    );
    assert!(
        !out.join("monolith.toml").exists(),
        "minimal preset without --monolith must not emit monolith.toml"
    );
    let settings_path = out.join("roles/default/settings.json");
    let raw = fs::read_to_string(&settings_path).expect("settings.json");
    let v: Value = serde_json::from_str(&raw).expect("parse settings");
    let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
    assert!(
        !pb.contains_key("agent"),
        "minimal preset omits agent in JSON"
    );
    assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "ollama");
    for k in [
        "_comment_memory",
        "_comment_llm",
        "_comment_plugin_backends",
    ] {
        assert!(v.get(k).is_some(), "missing root key {k}");
    }
    for k in [
        "memory",
        "emotion",
        "event",
        "prompt",
        "llm",
        "complex_emotion",
    ] {
        assert!(pb.get(k).is_some(), "missing plugin_backends.{k}");
    }
    let st2 = cargo_build(&out);
    assert!(st2.success(), "generated project cargo build");
}

#[test]
fn e2e_preset_full_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k2");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success());
    assert!(out.join("CONFIG_REFERENCE.md").is_file());
    let raw = fs::read_to_string(out.join("roles/default/settings.json")).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
    assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "remote");
    assert_eq!(pb.get("agent").unwrap().as_str().unwrap(), "builtin");
    assert!(cargo_build(&out).success());
}

#[test]
fn e2e_preset_mixed_library_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k3");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "mixed",
        "--project-type",
        "library",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success());
    assert!(cargo_build(&out).success());
}

#[test]
fn e2e_non_interactive_minimal_no_extra_input() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("test-project");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(cargo_build(&out).success());
}

#[test]
fn e2e_build_without_monolith_toml_no_cargo_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("no_mt");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("monolith.toml").exists());
    let o = run_cli_output(&["build", "-o", out.to_str().unwrap(), "--no-cargo"]);
    assert!(
        o.status.success(),
        "build without monolith should succeed with --no-cargo"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stderr),
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(
        combined.contains("monolith.toml"),
        "should mention missing monolith.toml: {combined}"
    );
}

#[test]
fn e2e_with_example_plugin_copies_llamacpp() {
    let tmp = tempfile::tempdir().unwrap();
    let out_on = tmp.path().join("with");
    let out_off = tmp.path().join("without");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "--with-example-plugin",
        "-o",
        out_on.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out_off.to_str().unwrap(),
    ])
    .success());
    let plug = out_on.join("plugins/com.oclive.example.llamacpp_llm/manifest.json");
    assert!(plug.is_file());
    assert!(!out_off
        .join("plugins/com.oclive.example.llamacpp_llm")
        .exists());
}

#[test]
fn e2e_init_skip_role_pack_no_roles_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("nr");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "--skip-role-pack",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("roles").exists());
}
