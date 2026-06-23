//! Distro profile requirements parsing & matching (scheduling subset).
//!
//! DTOs live in [`oclive_kernel_types`]; TOML SSOT in [`crate::distro_oclive_file`];
//! full runtime merge stays in `oclive_kernel_host::host_profile`.

use crate::distro_oclive_file::{
    parse_distro_oclive_file, parse_distro_oclive_toml, requirements_from_flags,
};
use oclive_kernel_types::{ActiveProfileSummary, DistroProfileRequirements, ProfileCompat};
use std::path::Path;

pub use oclive_kernel_types::{AttachReason, ReplaceReason};

/// Parse scheduling requirements from `distro.oclive.toml`.
///
/// # Errors
///
/// Returns I/O or TOML parse errors as strings.
pub fn parse_distro_requirements_file(path: &Path) -> Result<DistroProfileRequirements, String> {
    let file = parse_distro_oclive_file(path)?;
    Ok(file.into_requirements(path))
}

/// Parse scheduling requirements from TOML text (tests / in-memory).
///
/// # Errors
///
/// Returns TOML parse errors as strings.
pub fn parse_distro_requirements_toml(
    raw: &str,
    path_hint: &Path,
) -> Result<DistroProfileRequirements, String> {
    let file = parse_distro_oclive_toml(raw)?;
    Ok(file.into_requirements(path_hint))
}

#[must_use]
pub fn default_requirements_for_distro_id(distro_id: &str) -> DistroProfileRequirements {
    match distro_id {
        "vscode" => requirements_from_flags(
            "vscode",
            true,
            true,
            Some("concise".into()),
            Some("minimal".into()),
        ),
        "desktop" => requirements_from_flags(
            "desktop",
            false,
            false,
            Some("full".into()),
            Some("standard".into()),
        ),
        _ => requirements_from_flags(distro_id, false, false, None, None),
    }
}

#[must_use]
pub fn active_summary_from_requirements(req: &DistroProfileRequirements) -> ActiveProfileSummary {
    let base = ["memory", "emotion", "event", "prompt", "llm"];
    let mut enabled_modules: Vec<String> = base.iter().map(|s| (*s).to_string()).collect();
    let disabled_modules = req.forbidden_modules.clone();
    for m in &req.required_modules {
        if !enabled_modules.contains(m) && !disabled_modules.contains(m) {
            enabled_modules.push(m.clone());
        }
    }
    for f in &req.forbidden_modules {
        enabled_modules.retain(|e| e != f);
    }
    ActiveProfileSummary {
        distro_id: Some(req.distro_id.clone()),
        enabled_modules,
        disabled_modules,
        post_process_profile: req.post_process_profile.clone(),
        prompt_profile: req.prompt_profile.clone(),
        ..Default::default()
    }
}

/// Resolve caller requirements: profile file → defaults for `distro_id` (cross-distro capability policy).
#[must_use]
pub fn resolve_caller_requirements(
    distro_id: &str,
    profile_path: Option<&Path>,
) -> DistroProfileRequirements {
    if let Some(path) = profile_path.filter(|p| p.is_file()) {
        if let Ok(req) = parse_distro_requirements_file(path) {
            return req;
        }
    }
    default_requirements_for_distro_id(distro_id)
}

#[must_use]
pub fn profile_satisfies_caller(
    active: &ActiveProfileSummary,
    caller: &DistroProfileRequirements,
) -> bool {
    for m in &caller.forbidden_modules {
        if active.enabled_modules.iter().any(|e| e == m) {
            return false;
        }
    }
    for m in &caller.required_modules {
        if !active.enabled_modules.iter().any(|e| e == m) {
            return false;
        }
    }
    if let Some(ref want) = caller.post_process_profile {
        if active.post_process_profile.as_deref() != Some(want.as_str()) {
            return false;
        }
    }
    if let Some(ref want) = caller.prompt_profile {
        if active.prompt_profile.as_deref() != Some(want.as_str()) {
            return false;
        }
    }
    true
}

#[must_use]
pub fn profiles_compatible_by_hash(running_hash: Option<&str>, caller_hash: Option<&str>) -> bool {
    matches!(
        (running_hash, caller_hash),
        (Some(a), Some(b)) if !a.is_empty() && a == b
    )
}

/// Profile compatibility for policy (tightened: no distro_id-only match without summary).
#[must_use]
pub fn evaluate_profile_compat(
    caller: &DistroProfileRequirements,
    running_profile: Option<&ActiveProfileSummary>,
    running_distro_id: Option<&str>,
    running_profile_hash: Option<&str>,
    caller_profile_hash: Option<&str>,
) -> ProfileCompat {
    if profiles_compatible_by_hash(running_profile_hash, caller_profile_hash) {
        return ProfileCompat::Compatible;
    }

    if let Some(active) = running_profile {
        return if profile_satisfies_caller(active, caller) {
            ProfileCompat::Compatible
        } else {
            ProfileCompat::Incompatible
        };
    }

    if let Some(rid) = running_distro_id {
        if rid != caller.distro_id {
            return ProfileCompat::Incompatible;
        }
        if profiles_compatible_by_hash(running_profile_hash, caller_profile_hash) {
            return ProfileCompat::Compatible;
        }
        return ProfileCompat::Unknown;
    }

    ProfileCompat::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vscode_forbids_agent_and_complex_emotion() {
        let req = default_requirements_for_distro_id("vscode");
        assert!(req.forbidden_modules.contains(&"agent".to_string()));
        let active =
            active_summary_from_requirements(&default_requirements_for_distro_id("desktop"));
        assert!(!profile_satisfies_caller(&active, &req));
    }

    #[test]
    fn same_distro_id_different_hash_is_unknown_without_summary() {
        let caller = default_requirements_for_distro_id("desktop");
        let compat =
            evaluate_profile_compat(&caller, None, Some("desktop"), Some("aaa"), Some("bbb"));
        assert_eq!(compat, ProfileCompat::Unknown);
    }

    #[test]
    fn parse_example_vscode_toml() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/distro-profiles");
        let req = parse_distro_requirements_file(&root.join("vscode.oclive.toml")).unwrap();
        assert_eq!(req.distro_id, "vscode");
        assert!(req.forbidden_modules.contains(&"agent".to_string()));
    }
}
