//! Offline CLI previews use the real Host compiler without acquiring controllers.

use std::process::Command;

#[cfg(feature = "diagnostics-host")]
#[path = "support/resource_capture.rs"]
mod resource_capture;

#[cfg(not(feature = "diagnostics-host"))]
#[test]
fn default_build_explains_the_opt_in_without_reading_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
        .args(["doctor", "resource-plan", "missing.json", "--json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("diagnostics-host"));
    assert!(output.stdout.is_empty());
}

#[cfg(feature = "diagnostics-host")]
mod with_host {
    use super::*;
    use oclive_kernel_types::{ResourceCandidatePlan, ResourceCandidatePlanState};
    use serde_json::{json, Value};
    use std::fs;
    use tempfile::{tempdir, TempDir};

    struct Fixture {
        dir: TempDir,
        capture: Value,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                dir: tempdir().unwrap(),
                capture: resource_capture::captured_resources(),
            }
        }

        fn run(&self, extra: &[&str]) -> std::process::Output {
            let path = self.dir.path().join("diagnostics.json");
            let bytes = serde_json::to_vec_pretty(&self.capture).unwrap();
            fs::write(&path, &bytes).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
                .current_dir(self.dir.path())
                .args(["doctor", "resource-plan"])
                .arg(&path)
                .args(extra)
                // The preview is explicit input, independent of ambient host overrides.
                .env("OCLIVE_GPU_SAFETY_RESERVE_MIB", "65536")
                .env("OCLIVE_RESOURCE_ALLOW_UNVERIFIED", "false")
                .env("OCLIVE_GPU_DEVICE_INDEX", "999")
                .output()
                .unwrap();
            assert_eq!(
                fs::read(path).unwrap(),
                bytes,
                "capture must remain unchanged"
            );
            output
        }

        fn plan(&self, extra: &[&str]) -> ResourceCandidatePlan {
            let output = self.run(extra);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stderr).contains("offline"));
            serde_json::from_slice(&output.stdout)
                .expect("stdout is one canonical plan JSON document")
        }
    }

    #[test]
    fn captured_managed_facts_never_forge_controller_authority() {
        let fixture = Fixture::new();
        let plan = fixture.plan(&["--json"]);
        assert_ne!(plan.plan_id, "stale");
        assert_eq!(plan.compiled_from_revision, 42);
        assert_eq!(plan.selections[0].profile_id, "gpu");
        assert_eq!(plan.transitions.len(), 1);
        assert!(!plan.executable);
        assert_eq!(plan.state, ResourceCandidatePlanState::Degraded);
        assert!(plan
            .reason_codes
            .iter()
            .any(|code| code == "resource_plan_controller_unavailable"));
    }

    #[test]
    fn human_output_browses_profiles_and_preserves_stable_reasons() {
        let output = Fixture::new().run(&[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("builtin.test"));
        assert!(text.contains("gpu"));
        assert!(text.contains("2048"));
        assert!(text.contains("resource_plan_controller_unavailable"));
        assert!(text.contains("42"));
    }

    #[test]
    fn legacy_and_finite_profiles_round_trip_through_the_host_loader() {
        let fixture = Fixture::new();
        let profile = fixture.dir.path().join("distro.oclive.toml");
        let legacy = b"distro_id = 'legacy'\n";
        fs::write(&profile, legacy).unwrap();
        let plan = fixture.plan(&["--json", "--distro-profile", profile.to_str().unwrap()]);
        assert!(
            plan.transitions.is_empty(),
            "legacy profile has no residency request"
        );
        assert_eq!(fs::read(&profile).unwrap(), legacy);
        let finite = b"distro_id = 'finite'\n[resource_coordination]\nstrategy = 'custom'\n[[resource_coordination.commands]]\nkind = 'residency'\nadapter_id = 'builtin.test'\nmode = 'resident'\n";
        fs::write(&profile, finite).unwrap();
        let plan = fixture.plan(&["--json", "--distro-profile", profile.to_str().unwrap()]);
        assert_eq!(plan.transitions.len(), 1);
        assert!(!plan.executable);
        assert_eq!(fs::read(&profile).unwrap(), finite);
    }

    #[test]
    fn absent_or_explicitly_missing_gpu_is_not_invented() {
        let fixture = Fixture::new();
        let plan = fixture.plan(&["--json", "--gpu-device-index", "9"]);
        assert!(!plan.executable);
        assert_ne!(plan.state, ResourceCandidatePlanState::Ready);
        let mut fixture = fixture;
        fixture.capture["snapshot"]["available"] = json!(false);
        fixture.capture["snapshot"]["gpu_devices"] = json!([]);
        let plan = fixture.plan(&["--json"]);
        assert!(!plan.executable);
        assert!(plan
            .reason_codes
            .iter()
            .any(|code| code.starts_with("resource_plan_gpu_")));
    }

    #[test]
    fn observe_only_and_conflicting_intent_remain_diagnostics() {
        let mut fixture = Fixture::new();
        fixture.capture["adapters"][0]["descriptor"]["control_mode"] = json!("observe_only");
        fixture.capture["adapters"][0]["descriptor"]["lifecycle_operations"] = json!(["observe"]);
        fixture.capture["adapters"][0]["descriptor"]["profiles"][0]["coordinator_selectable"] =
            json!(false);
        let plan = fixture.plan(&["--json"]);
        assert!(!plan.executable);
        assert_ne!(plan.state, ResourceCandidatePlanState::Ready);

        let mut fixture = Fixture::new();
        let mut other = fixture.capture["adapters"][0].clone();
        other["descriptor"]["adapter_id"] = json!("builtin.other");
        fixture.capture["adapters"]
            .as_array_mut()
            .unwrap()
            .push(other);
        fixture.capture["policy"]["scheduling"]["commands"] = json!([
            {"kind": "coexist", "adapter_ids": ["builtin.test", "builtin.other"]},
            {"kind": "exclusive", "adapter_ids": ["builtin.test", "builtin.other"]}
        ]);
        let plan = fixture.plan(&["--json"]);
        assert_eq!(plan.state, ResourceCandidatePlanState::Blocked);
        assert!(plan
            .reason_codes
            .iter()
            .any(|code| code == "resource_scheduling_group_conflict"));
    }

    #[test]
    fn wrong_schema_duplicate_adapter_and_invalid_owner_fail_before_a_plan() {
        for mutation in 0..3 {
            let mut fixture = Fixture::new();
            match mutation {
                0 => fixture.capture["schema_version"] = json!(999),
                1 => {
                    let duplicate = fixture.capture["adapters"][0].clone();
                    fixture.capture["adapters"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                2 => {
                    fixture.capture["adapters"][0]["registration_source"] =
                        json!("directory_plugin");
                    fixture.capture["adapters"][0]["registration_source_id"] = json!("com.other");
                }
                _ => unreachable!(),
            }
            let output = fixture.run(&["--json"]);
            assert!(!output.status.success());
            assert!(
                output.stdout.is_empty(),
                "invalid input must not emit a candidate"
            );
        }
    }

    #[test]
    fn typed_input_and_profile_parse_errors_never_emit_a_candidate() {
        let mut fixture = Fixture::new();
        fixture.capture["snapshot"]["available"] = json!("false");
        let output = fixture.run(&["--json"]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let fixture = Fixture::new();
        let profile = fixture.dir.path().join("invalid.oclive.toml");
        fs::write(&profile, "distro_id = 'bad'\n[resource_coordination]\n[[resource_coordination.commands]]\nkind = 'residency'\nadapter_id = 'builtin.test'\nmode = 'flight'\n").unwrap();
        let output = fixture.run(&["--json", "--distro-profile", profile.to_str().unwrap()]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("load distro profile"));
        let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
            .current_dir(fixture.dir.path())
            .args(["doctor", "resource-plan", "missing.json", "--json"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("read resource diagnostics"));
    }
}
