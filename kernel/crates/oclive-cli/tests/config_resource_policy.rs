//! Explicit, offline drafts preserve source files and use the original Host policy.
use std::process::Command;

#[cfg(not(feature = "diagnostics-host"))]
#[test]
fn default_build_rejects_before_reading_or_writing_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
        .current_dir(dir.path())
        .args([
            "config",
            "resource-policy",
            "--distro-profile",
            "missing.toml",
            "--policy-file",
            "missing-policy.toml",
            "--diagnostics-file",
            "missing.json",
            "--output",
            "new.toml",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("diagnostics-host"));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[cfg(feature = "diagnostics-host")]
#[path = "support/resource_capture.rs"]
mod resource_capture;

#[cfg(not(feature = "diagnostics-host"))]
#[test]
fn default_build_rejects_interactive_before_reading_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
        .current_dir(dir.path())
        .args([
            "config",
            "resource-policy",
            "--interactive",
            "--distro-profile",
            "missing.toml",
            "--diagnostics-file",
            "missing.json",
            "--output",
            "draft.toml",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("diagnostics-host"));
    assert!(output.stdout.is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[cfg(feature = "diagnostics-host")]
mod with_host {
    use super::*;
    use oclive_kernel_host::domain::host_profile::load_host_profile_file;
    use oclive_kernel_types::{ResourceCandidatePlan, ResourceSchedulingStrategy};
    use serde_json::{json, Value};
    use std::{fs, path::PathBuf, process::Output};
    use tempfile::{tempdir, TempDir};

    struct Fixture {
        dir: TempDir,
        source: String,
        capture: Value,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                dir: tempdir().unwrap(),
                source: "# Keep original source bytes\ndistro_id = 'legacy'\n[host_flags]\nskip_agent = true\n[future_extension]\nlabel = '保留'\nlevels = [1, 2]\n".into(),
                capture: resource_capture::captured_resources(),
            }
        }

        fn output_path(&self) -> PathBuf {
            self.dir.path().join("draft.toml")
        }

        fn run(&self, patch: &str, extra: &[&str]) -> Output {
            let source = self.dir.path().join("source.toml");
            let policy = self.dir.path().join("policy.toml");
            let capture = self.dir.path().join("capture.json");
            let bytes = serde_json::to_vec(&self.capture).unwrap();
            fs::write(&source, &self.source).unwrap();
            fs::write(&policy, patch).unwrap();
            fs::write(&capture, &bytes).unwrap();
            let mut command = Command::new(env!("CARGO_BIN_EXE_oclive-cli"));
            command
                .current_dir(self.dir.path())
                .args(["config", "resource-policy", "--distro-profile"])
                .arg(&source)
                .arg("--policy-file")
                .arg(&policy);
            if !extra.contains(&"--diagnostics-file") {
                command.arg("--diagnostics-file").arg(&capture);
            }
            if !extra.contains(&"--output") {
                command.arg("--output").arg(self.output_path());
            }
            let output = command
                .args(extra)
                .env("OCLIVE_GPU_SAFETY_RESERVE_MIB", "999999")
                .output()
                .unwrap();
            assert_eq!(fs::read(source).unwrap(), self.source.as_bytes());
            assert_eq!(fs::read(policy).unwrap(), patch.as_bytes());
            assert_eq!(fs::read(capture).unwrap(), bytes);
            assert_eq!(
                fs::read_dir(self.dir.path()).unwrap().count(),
                if self.output_path().exists() { 4 } else { 3 },
                "no temporary draft left behind"
            );
            output
        }

        fn succeeds(&self, patch: &str) -> Output {
            let result = self.run(patch, &[]);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(String::from_utf8_lossy(&result.stderr).contains("offline"));
            result
        }

        fn rejects(&self, patch: &str, reason: &str) {
            let output = self.run(patch, &[]);
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(reason),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!self.output_path().exists());
        }

        fn interactive(&self, answers: &str, extra: &[&str]) -> Output {
            use std::{io::Write, process::Stdio};
            let source = self.dir.path().join("source.toml");
            let capture = self.dir.path().join("capture.json");
            let bytes = serde_json::to_vec(&self.capture).unwrap();
            fs::write(&source, &self.source).unwrap();
            fs::write(&capture, &bytes).unwrap();
            let mut child = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
                .current_dir(self.dir.path())
                .args([
                    "config",
                    "resource-policy",
                    "--interactive",
                    "--distro-profile",
                ])
                .arg(&source)
                .arg("--diagnostics-file")
                .arg(&capture)
                .arg("--output")
                .arg(self.output_path())
                .args(extra)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            // Early CLI/capture rejection may close stdin before reading the supplied answers.
            let _ = child.stdin.take().unwrap().write_all(answers.as_bytes());
            let output = child.wait_with_output().unwrap();
            assert_eq!(fs::read(source).unwrap(), self.source.as_bytes());
            assert_eq!(fs::read(capture).unwrap(), bytes);
            assert_eq!(
                fs::read_dir(self.dir.path()).unwrap().count(),
                if self.output_path().exists() { 3 } else { 2 },
                "no wizard temporary file left behind"
            );
            output
        }
    }

    #[test]
    fn interactive_keeps_values_until_explicit_confirmation() {
        let mut fixture = Fixture::new();
        fixture.source.push_str("[resource_coordination]\nstrategy='latency_first'\ngpu_safety_reserve_mib=777\nfuture_hint='kept'\n");
        let output = fixture.interactive("\n\n\n\n\n\nyes\n", &[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("builtin.test"));
        assert!(text.contains("gpu"));
        assert!(text.contains("not live admission"));
        let profile = load_host_profile_file(&fixture.output_path()).unwrap();
        assert_eq!(profile.resource_coordination.gpu_safety_reserve_mib, 777);
        assert_eq!(
            profile.resource_coordination.scheduling.strategy,
            ResourceSchedulingStrategy::LatencyFirst
        );
        let mut document: toml::Value =
            toml::from_str(&fs::read_to_string(fixture.output_path()).unwrap()).unwrap();
        assert_eq!(
            document["resource_coordination"]["future_hint"].as_str(),
            Some("kept")
        );
        document
            .as_table_mut()
            .unwrap()
            .remove("resource_coordination");
        let mut original: toml::Value = toml::from_str(&fixture.source).unwrap();
        original
            .as_table_mut()
            .unwrap()
            .remove("resource_coordination");
        assert_eq!(document, original);
    }

    #[test]
    fn interactive_serializes_all_six_canonical_commands() {
        let mut fixture = Fixture::new();
        for id in ["builtin.second", "builtin.third"] {
            let mut second = fixture.capture["adapters"][0].clone();
            second["descriptor"]["adapter_id"] = json!(id);
            fixture.capture["adapters"]
                .as_array_mut()
                .unwrap()
                .push(second);
        }
        let answers = [
            "4",
            "",
            "+",
            "1",
            "builtin.test",
            "2",
            "builtin.test",
            "4",
            "builtin.test,builtin.second",
            "5",
            "builtin.test,builtin.third",
            "6",
            "builtin.test",
            "builtin.second",
            "7",
            "builtin.test",
            "gpu",
            "0",
            "999999",
            "1000",
            "1",
            "yes",
            "",
        ]
        .join("\n");
        let output = fixture.interactive(&answers, &[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let profile = load_host_profile_file(&fixture.output_path()).unwrap();
        assert_eq!(
            profile.resource_coordination.scheduling.strategy,
            ResourceSchedulingStrategy::Custom
        );
        assert_eq!(profile.resource_coordination.scheduling.commands.len(), 6);
        assert_eq!(profile.resource_coordination.gpu_safety_reserve_mib, 65536);
        assert_eq!(
            profile
                .resource_coordination
                .system_memory_safety_reserve_mib,
            1000
        );
        assert_eq!(profile.resource_coordination.cpu_safety_reserve_threads, 1);
        let document: toml::Value =
            toml::from_str(&fs::read_to_string(fixture.output_path()).unwrap()).unwrap();
        let commands = document["resource_coordination"]["commands"]
            .as_array()
            .unwrap();
        assert_eq!(
            commands
                .iter()
                .map(|c| c["kind"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "require",
                "residency",
                "coexist",
                "exclusive",
                "yield_then_run",
                "fallback"
            ]
        );
        let preview = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
            .args(["doctor", "resource-plan"])
            .arg(fixture.dir.path().join("capture.json"))
            .arg("--distro-profile")
            .arg(fixture.output_path())
            .arg("--json")
            .output()
            .unwrap();
        assert!(preview.status.success());
        let plan: ResourceCandidatePlan = serde_json::from_slice(&preview.stdout).unwrap();
        assert_eq!(plan.compiled_from_revision, 42);
        assert!(!plan.executable);
    }

    #[test]
    fn interactive_clears_explicitly_and_supports_on_demand() {
        for (answers, command_count) in [
            ("1\n-\n-\n\n\n\nyes\n", 0),
            ("3\n-\n+\n3\nbuiltin.test\n0\n\n\n\nyes\n", 1),
        ] {
            let mut fixture = Fixture::new();
            fixture.source.push_str("[resource_coordination]\nstrategy='custom'\nprimary_adapter_id='builtin.test'\ncommands=[{kind='residency',adapter_id='builtin.test',mode='resident'}]\n");
            let output = fixture.interactive(answers, &[]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let profile = load_host_profile_file(&fixture.output_path()).unwrap();
            assert_eq!(
                profile.resource_coordination.scheduling.primary_adapter_id,
                None
            );
            assert_eq!(
                profile.resource_coordination.scheduling.commands.len(),
                command_count
            );
            if command_count == 1 {
                assert!(fs::read_to_string(fixture.output_path())
                    .unwrap()
                    .contains("on_demand"));
            }
        }
    }

    #[test]
    fn interactive_selects_a_canonical_primary_strategy() {
        let fixture = Fixture::new();
        let answers = ["2", "builtin.test", "", "", "", "", "yes", ""].join("\n");
        let output = fixture.interactive(&answers, &[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let profile = load_host_profile_file(&fixture.output_path()).unwrap();
        assert_eq!(
            profile.resource_coordination.scheduling.strategy,
            ResourceSchedulingStrategy::PrimaryFirst
        );
        assert_eq!(
            profile
                .resource_coordination
                .scheduling
                .primary_adapter_id
                .as_deref(),
            Some("builtin.test")
        );
    }

    #[test]
    fn interactive_cancel_eof_and_bad_choices_never_publish() {
        for answers in ["q\n", "", "9\n", "\n\n\n\n\n\nno\n", "\n\n\nbad-number\n"] {
            let fixture = Fixture::new();
            let output = fixture.interactive(answers, &[]);
            assert!(!output.status.success());
            assert!(!fixture.output_path().exists());
        }
        let mut fixture = Fixture::new();
        fixture.capture["schema_version"] = json!(0);
        let output = fixture.interactive("", &[]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("schema_version"));
        assert!(output.stdout.is_empty());
    }

    #[test]
    fn interactive_uses_original_validator_before_confirmation() {
        let fixture = Fixture::new();
        let output = fixture.interactive("4\n\n+\n1\nbuiltin.unknown\n0\n\n\n\nyes\n", &[]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("resource_scheduling_adapter_unregistered"));
        assert!(!fixture.output_path().exists());
        let mut fixture = Fixture::new();
        let mut second = fixture.capture["adapters"][0].clone();
        second["descriptor"]["adapter_id"] = json!("builtin.second");
        fixture.capture["adapters"]
            .as_array_mut()
            .unwrap()
            .push(second);
        let output = fixture.interactive("4\n\n+\n4\nbuiltin.test,builtin.second\n5\nbuiltin.test,builtin.second\n0\n\n\n\nyes\n", &[]);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("resource_scheduling_group_conflict")
        );
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("At least two adapters are required to both coexist and be exclusive"));
        assert!(!fixture.output_path().exists());
    }

    #[test]
    fn interactive_rejects_patch_file_and_existing_output_before_prompting() {
        let fixture = Fixture::new();
        let output = fixture.interactive("", &["--policy-file", "missing.toml"]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used"));
        assert!(!fixture.output_path().exists());
        let fixture = Fixture::new();
        fs::write(fixture.output_path(), b"existing sentinel").unwrap();
        let output = fixture.interactive("", &[]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            fs::read(fixture.output_path()).unwrap(),
            b"existing sentinel"
        );
    }

    #[test]
    fn legacy_profile_draft_round_trips_through_original_preview() {
        let fixture = Fixture::new();
        fixture.succeeds("[resource_coordination]\nstrategy = 'custom'\n[[resource_coordination.commands]]\nkind = 'residency'\nadapter_id = 'builtin.test'\nmode = 'resident'\n");
        let original: toml::Value = toml::from_str(&fixture.source).unwrap();
        let mut generated: toml::Value =
            toml::from_str(&fs::read_to_string(fixture.output_path()).unwrap()).unwrap();
        generated
            .as_table_mut()
            .unwrap()
            .remove("resource_coordination");
        assert_eq!(generated, original);
        let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
            .args(["doctor", "resource-plan"])
            .arg(fixture.dir.path().join("capture.json"))
            .arg("--distro-profile")
            .arg(fixture.output_path())
            .arg("--json")
            .output()
            .unwrap();
        assert!(output.status.success());
        let plan: ResourceCandidatePlan = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(plan.compiled_from_revision, 42);
        assert_eq!(plan.transitions.len(), 1);
        assert!(!plan.executable);
        assert!(plan
            .reason_codes
            .iter()
            .any(|code| code == "resource_plan_controller_unavailable"));
    }

    #[test]
    fn partial_patch_preserves_existing_keys_and_original_loader_clamps() {
        let mut fixture = Fixture::new();
        fixture.source.push_str("[resource_coordination]\nstrategy = 'latency_first'\ncpu_safety_reserve_threads = 3\nfuture_hint = 'kept'\n");
        fixture.succeeds("[resource_coordination]\ngpu_safety_reserve_mib = 999999\n");
        let profile = load_host_profile_file(&fixture.output_path()).unwrap();
        assert_eq!(profile.resource_coordination.gpu_safety_reserve_mib, 65536);
        assert_eq!(profile.resource_coordination.cpu_safety_reserve_threads, 3);
        assert_eq!(
            profile.resource_coordination.scheduling.strategy,
            ResourceSchedulingStrategy::LatencyFirst
        );
        let generated: toml::Value =
            toml::from_str(&fs::read_to_string(fixture.output_path()).unwrap()).unwrap();
        assert_eq!(
            generated["resource_coordination"]["future_hint"].as_str(),
            Some("kept")
        );
    }

    #[test]
    fn commands_replace_instead_of_appending_and_can_be_cleared() {
        for patch in [
            "[resource_coordination]\ncommands = [{kind='require', adapter_id='builtin.test'}]\n",
            "[resource_coordination]\nstrategy = 'compatibility_first'\ncommands = []\n",
        ] {
            let mut fixture = Fixture::new();
            fixture.source.push_str("[resource_coordination]\nstrategy = 'custom'\ncommands = [{kind='residency', adapter_id='builtin.test', mode='resident'}]\n");
            fixture.succeeds(patch);
            let profile = load_host_profile_file(&fixture.output_path()).unwrap();
            assert_eq!(
                profile.resource_coordination.scheduling.commands.len(),
                if patch.contains("commands = []") {
                    0
                } else {
                    1
                }
            );
            assert!(!fs::read_to_string(fixture.output_path())
                .unwrap()
                .contains("mode = \"resident\""));
        }
    }

    #[test]
    fn malformed_or_cross_section_patches_never_publish() {
        for (patch, reason) in [
            ("[host_flags]\nskip_agent = false\n", "only [resource_coordination]"),
            ("[resource_coordination]\nstrategy = 'custom'\n[host_flags]\nskip_agent = false\n", "only [resource_coordination]"),
            ("resource_coordination = false\n", "table"),
            ("[resource_coordination]\nstrategy = 'flight'\n", "unknown variant"),
            ("[resource_coordination]\nautomatic_preemption = 'false'\n", "invalid type"),
            ("[resource_coordination]\ncommands = [{kind='residency', adapter_id='builtin.test', mode='flight'}]\n", "unknown variant"),
        ] { Fixture::new().rejects(patch, reason); }
    }

    #[test]
    fn original_registry_rejects_unknown_empty_and_conflicting_intents() {
        for (patch, reason) in [
            ("[resource_coordination]\nstrategy = 'primary_first'\n", "resource_primary_adapter_required"),
            ("[resource_coordination]\nstrategy = 'custom'\n", "resource_custom_schedule_empty"),
            ("[resource_coordination]\ncommands = [{kind='require', adapter_id='builtin.unknown'}]\n", "resource_scheduling_adapter_unregistered"),
        ] { Fixture::new().rejects(patch, reason); }
        let mut fixture = Fixture::new();
        let mut second = fixture.capture["adapters"][0].clone();
        second["descriptor"]["adapter_id"] = json!("builtin.second");
        fixture.capture["adapters"]
            .as_array_mut()
            .unwrap()
            .push(second);
        fixture.rejects("[resource_coordination]\ncommands = [{kind='coexist', adapter_ids=['builtin.test','builtin.second']}, {kind='exclusive', adapter_ids=['builtin.test','builtin.second']}]\n", "resource_scheduling_group_conflict");
    }

    #[test]
    fn existing_output_and_source_as_output_are_never_overwritten() {
        let fixture = Fixture::new();
        fs::write(fixture.output_path(), b"existing output sentinel").unwrap();
        let output = fixture.run("[resource_coordination]\n", &[]);
        assert!(!output.status.success());
        assert_eq!(
            fs::read(fixture.output_path()).unwrap(),
            b"existing output sentinel"
        );
        let fixture = Fixture::new();
        let output = fixture.run("[resource_coordination]\n", &["--output", "source.toml"]);
        assert!(!output.status.success());
        assert!(!fixture.output_path().exists());
    }

    #[test]
    fn valid_degraded_profiles_remain_drafts_without_controller_authority() {
        let mut fixture = Fixture::new();
        fixture.capture["adapters"][0]["descriptor"]["control_mode"] = json!("observe_only");
        fixture.capture["adapters"][0]["descriptor"]["lifecycle_operations"] = json!(["observe"]);
        fixture.capture["adapters"][0]["descriptor"]["profiles"][0]["coordinator_selectable"] =
            json!(false);
        fixture.capture["snapshot"]["available"] = json!(false);
        fixture.capture["snapshot"]["gpu_devices"] = json!([]);
        let output = fixture.succeeds("[resource_coordination]\ncommands = [{kind='residency', adapter_id='builtin.test', mode='resident'}]\n");
        let text = String::from_utf8_lossy(&output.stderr);
        assert!(text.contains("resource_"));
        assert!(text.contains("no live controllers"));
    }

    #[test]
    fn draft_and_confirmation_explain_the_same_missing_controller() {
        let patch = "[resource_coordination]\ncommands = [{kind='residency', adapter_id='builtin.test', mode='resident'}]\n";
        let fixture = Fixture::new();
        let output = fixture.succeeds(patch);
        let expected = "resource_plan_controller_unavailable — No live controller is available for a proposed transition";
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));

        let mut fixture = Fixture::new();
        fixture.source.push_str(patch);
        let output = fixture.interactive("\n\n\n\n\n\nno\n", &[]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
        assert!(!fixture.output_path().exists());
    }

    #[test]
    fn invalid_capture_is_not_a_reason_to_skip_validation() {
        let mut fixture = Fixture::new();
        fixture.capture["schema_version"] = json!(0);
        fixture.rejects("[resource_coordination]\n", "schema_version");
        let fixture = Fixture::new();
        let output = fixture.run(
            "[resource_coordination]\n",
            &["--diagnostics-file", "missing.json"],
        );
        assert!(!output.status.success());
        assert!(!fixture.output_path().exists());
    }
}
