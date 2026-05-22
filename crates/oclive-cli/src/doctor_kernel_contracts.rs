//! `oclive doctor` checks for [`oclive_kernel_contracts`] trait implementations in kernel trees.

use crate::doctor_cmd::DoctorCheck;
use std::path::{Path, PathBuf};

const TRAIT_CHECKS: &[(&str, &str)] = &[
    ("plugin_host_port_impl", "PluginHostPort"),
    ("llm_client_impl", "LlmClient"),
    ("slot_registry_resolver_impl", "SlotRegistryResolver"),
    ("event_estimator_impl", "EventEstimator"),
    ("agent_provider_impl", "AgentProvider"),
];

/// When `Cargo.toml` references oclive kernel crates, verify `src/` (or `src-tauri/src/`) contains trait impls.
pub(crate) fn kernel_contract_impl_checks(root: &Path) -> Vec<DoctorCheck> {
    let cargo = root.join("Cargo.toml");
    let workspace_cargo = root.join("src-tauri/Cargo.toml");
    let kernel_toml = if cargo.is_file() {
        cargo
    } else if workspace_cargo.is_file() {
        workspace_cargo
    } else {
        return vec![DoctorCheck::ok(
            "kernel_contracts",
            "no kernel Cargo.toml at probe root (trait impl audit skipped)",
        )];
    };
    let raw = match std::fs::read_to_string(&kernel_toml) {
        Ok(s) => s,
        Err(e) => {
            return vec![DoctorCheck::warn(
                "kernel_contracts",
                format!("cannot read {}: {e}", kernel_toml.display()),
                None,
            )];
        }
    };
    if !raw.contains("oclive_kernel") {
        return vec![DoctorCheck::ok(
            "kernel_contracts",
            "project does not depend on oclive_kernel_* (trait impl audit skipped)",
        )];
    }

    let src_roots = rust_src_roots(root);
    if src_roots.is_empty() {
        return vec![DoctorCheck::warn(
            "kernel_contracts",
            "kernel dependency present but no src/ tree found",
            Some("expected src/ or src-tauri/src/ with trait impl blocks".into()),
        )];
    }

    TRAIT_CHECKS
        .iter()
        .map(|(id, trait_name)| {
            if source_contains_trait(&src_roots, trait_name) {
                DoctorCheck::ok(id, format!("found impl for {trait_name}"))
            } else {
                DoctorCheck::fail(
                    id,
                    format!("no `impl {trait_name}` found under {}", root.display()),
                    Some(format!(
                        "implement {trait_name} for your host adapter (see oclive_kernel_contracts)"
                    )),
                )
            }
        })
        .collect()
}

fn rust_src_roots(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let direct = root.join("src");
    if direct.is_dir() {
        out.push(direct);
    }
    let tauri = root.join("src-tauri/src");
    if tauri.is_dir() {
        out.push(tauri);
    }
    out
}

fn source_contains_trait(src_roots: &[PathBuf], trait_name: &str) -> bool {
    let needle = format!("impl {trait_name}");
    let needle_for = format!("{trait_name} for");
    for root in src_roots {
        for path in walk_rust_files(root) {
            let Ok(raw) = std::fs::read_to_string(&path) else {
                continue;
            };
            if raw.contains(&needle) || raw.contains(&needle_for) {
                return true;
            }
        }
    }
    false
}

fn walk_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}
