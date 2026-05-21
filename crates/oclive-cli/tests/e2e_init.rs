//! 端到端：非交互生成项目并 `cargo build`。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/oclive-cli")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn run_cli_output(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .current_dir(repo_root())
        .args(["run", "-p", "oclive-cli", "--"])
        .args(args)
        .output()
        .expect("cargo run -p oclive-cli")
}

fn run_cli(args: &[&str]) -> std::process::ExitStatus {
    Command::new("cargo")
        .current_dir(repo_root())
        .args(["run", "-p", "oclive-cli", "--quiet", "--"])
        .args(args)
        .status()
        .expect("cargo run -p oclive-cli")
}

fn assert_bench_report_matches_schema(v: &Value) {
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

fn cargo_build(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build")
}

fn cargo_build_release(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release")
}

fn cargo_build_release_monolith(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release", "--features", "monolith"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release --features monolith")
}

fn release_binary_path(project_dir: &std::path::Path, bin_base: &str) -> std::path::PathBuf {
    let p = project_dir.join("target/release").join(bin_base);
    if cfg!(windows) {
        p.with_extension("exe")
    } else {
        p
    }
}

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
fn e2e_pack_create_validate_publish() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("com.example.demo");
    assert!(run_cli(&[
        "pack",
        "create",
        "-o",
        root.to_str().unwrap(),
        "--flat",
        "--id",
        "com.example.demo",
        "--name",
        "Demo",
    ])
    .success());
    assert!(root.join("manifest.json").exists());
    assert!(run_cli(&[
        "pack",
        "validate",
        root.to_str().unwrap(),
        "--host-version",
        "999.0.0",
    ])
    .success());
    let zip_path = tmp.path().join("out.oclivepack");
    assert!(run_cli(&[
        "pack",
        "publish",
        root.to_str().unwrap(),
        "-o",
        zip_path.to_str().unwrap(),
    ])
    .success());
    assert!(zip_path.is_file());
}

#[test]
fn e2e_pack_validate_robot_soul_example() {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("robot-soul-minimal")
        .join("roles")
        .join("default");
    let example = example
        .canonicalize()
        .expect("robot-soul-minimal example path");
    assert!(
        example.join("manifest.json").is_file(),
        "missing {}",
        example.display()
    );
    let o = run_cli_output(&[
        "pack",
        "validate",
        example.to_str().unwrap(),
        "--host-version",
        "0.2.0",
        "--profile",
        "robot-soul",
    ]);
    assert!(
        o.status.success(),
        "robot-soul validate failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn e2e_template_robot_soul_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-soul",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "--monolith",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(out_tpl.join("monolith.toml").is_file());
    assert!(out_manual.join("monolith.toml").is_file());
    let s_tpl = fs::read_to_string(out_tpl.join("roles/default/settings.json")).unwrap();
    let s_man = fs::read_to_string(out_manual.join("roles/default/settings.json")).unwrap();
    let v_tpl: Value = serde_json::from_str(&s_tpl).unwrap();
    let v_man: Value = serde_json::from_str(&s_man).unwrap();
    assert_eq!(v_tpl.get("plugin_backends"), v_man.get("plugin_backends"));
    assert!(out_tpl.join("roles/default/prompts/system.md").is_file());
    assert!(out_tpl.join("plugins/README.md").is_file());
    let m: Value = serde_json::from_str(
        &fs::read_to_string(out_tpl.join("roles/default/manifest.json")).unwrap(),
    )
    .unwrap();
    assert!(m.get("default_personality").is_some());
}

#[test]
fn e2e_template_headless_api_no_roles_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("api");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "headless-api",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("roles").exists());
    assert!(!out.join("monolith.toml").exists());
    assert!(out.join("plugins/README.md").is_file());
}

#[test]
fn e2e_template_library_embed() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("lib");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "library-embed",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("monolith.toml").exists());
    let cargo = fs::read_to_string(out.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("[lib]") || cargo.contains("crate-type"));
    assert!(!out.join("roles").exists());
}

#[test]
fn e2e_template_robot_gateway_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-gateway",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "mixed",
        "--monolith",
        "--skip-role-pack",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(out_tpl.join("monolith.toml").is_file());
    assert!(out_manual.join("monolith.toml").is_file());
    assert!(out_tpl.join("roles/gateway/settings.json").is_file());
    assert!(out_tpl.join("mcp_servers/README.md").is_file());
    assert!(!out_manual.join("roles").exists());
    let mt_tpl = fs::read_to_string(out_tpl.join("monolith.toml")).unwrap();
    let mt_man = fs::read_to_string(out_manual.join("monolith.toml")).unwrap();
    assert_eq!(mt_tpl, mt_man);
}

#[test]
fn e2e_template_dialogue_only_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "dialogue-only",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(!out_tpl.join("monolith.toml").exists());
    assert!(out_tpl.join("roles/default/settings.json").is_file());
    let s_tpl = fs::read_to_string(out_tpl.join("roles/default/settings.json")).unwrap();
    let s_man = fs::read_to_string(out_manual.join("roles/default/settings.json")).unwrap();
    let v_tpl: Value = serde_json::from_str(&s_tpl).unwrap();
    let v_man: Value = serde_json::from_str(&s_man).unwrap();
    assert_eq!(v_tpl.get("plugin_backends"), v_man.get("plugin_backends"));
    assert!(out_tpl.join("docs/ORCHESTRATION_REFERENCE.md").is_file());
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
fn e2e_doctor_json_smoke() {
    let output = run_cli_output(&["doctor", "--json"]);
    let v: Value = serde_json::from_slice(&output.stdout).expect("doctor json");
    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(1));
    assert!(
        v.get("checks")
            .and_then(|x| x.as_array())
            .map(|a| !a.is_empty())
            == Some(true)
    );
}

#[test]
fn e2e_init_quick_non_interactive() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("quick");
    assert!(run_cli(&[
        "init",
        "--quick",
        "--non-interactive",
        "--quiet",
        "-o",
        out.to_str().unwrap(),
        "--project-name",
        "quick-chat",
    ])
    .success());
    assert!(out.join("Cargo.toml").is_file());
    assert!(!out.join("monolith.toml").exists());
    assert!(!out.join("roles").exists());
    let settings = out.join("CONFIG_REFERENCE.md");
    assert!(settings.is_file());
}

#[test]
fn e2e_list_templates_prints_matrix() {
    let output = run_cli_output(&["init", "--list-templates"]);
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("robot-gateway"));
    assert!(out.contains("dialogue-only"));
    assert!(out.contains("preset"));
}

#[test]
fn e2e_template_robot_gateway_has_mcp_scaffold() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("gw");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-gateway",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(out.join("mcp_servers/README.md").is_file());
    assert!(out.join("mcp_servers/smart_home.example.json").is_file());
    let settings = fs::read_to_string(out.join("roles/gateway/settings.json")).unwrap();
    assert!(settings.contains("\"agent\""));
    assert!(settings.contains("agent_mcp"));
    let v: Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(
        v.get("plugin_backends")
            .and_then(|p| p.get("agent"))
            .and_then(|a| a.as_str()),
        Some("builtin")
    );
    assert!(out.join("docs/WELD_BENCH_REPORT.md").is_file());
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
