//! Black-box tests for `kernel ensure --plan-only --json` schema stability.

use oclive_kernel_runtime::{
    build_resolve_plan, default_requirements_for_distro_id, PolicyContext,
};
use oclive_kernel_types::{AttachReason, ProfileCompat, ReplaceReason};

#[test]
fn ensure_report_schema_fields_roundtrip() {
    let ctx = PolicyContext {
        health_ok: true,
        running_manifest: None,
        running_distro_id: Some("desktop".into()),
        running_profile: None,
        running_profile_hash: Some("abc".into()),
    };
    let resolution = build_resolve_plan(&ctx, &[], "vscode", None, false, true, true);
    let json = serde_json::json!({
        "schema_version": 2,
        "plan": resolution.plan,
        "profile_compat": resolution.profile_compat,
        "caller_requirements": resolution.caller_requirements,
        "running_profile_summary": resolution.running_profile_summary,
        "executed": false,
        "health_ok": true,
        "running_distro_id": "desktop",
    });
    assert_eq!(json["schema_version"], 2);
    assert!(json["profile_compat"].is_string());
    assert!(json["caller_requirements"]["distroId"].is_string());
}

#[test]
fn attach_reason_serializes_snake_case() {
    let v = serde_json::to_value(AttachReason::KernelPinnedProfileMismatch).unwrap();
    assert_eq!(v, "kernel_pinned_profile_mismatch");
}

#[test]
fn replace_reason_serializes_snake_case() {
    let v = serde_json::to_value(ReplaceReason::ProfileMismatch).unwrap();
    assert_eq!(v, "profile_mismatch");
}

#[test]
fn profile_compat_unknown_when_hash_mismatch_no_summary() {
    use oclive_kernel_runtime::evaluate_profile_compat;
    let caller = default_requirements_for_distro_id("desktop");
    let compat = evaluate_profile_compat(&caller, None, Some("desktop"), Some("aaa"), Some("bbb"));
    assert_eq!(compat, ProfileCompat::Unknown);
}
