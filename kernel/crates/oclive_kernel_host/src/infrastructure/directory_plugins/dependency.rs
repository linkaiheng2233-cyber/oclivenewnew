//! Directory plugin manifest `dependencies` vs installed versions.

use super::manifest::OclivePluginManifest;
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Returns `(dependency_status, dependency_issues)` where `status` is `ok` / `missing` / `mismatch`.
#[must_use]
pub fn dependency_report(
    manifest: &OclivePluginManifest,
    version_by_id: &HashMap<String, Version>,
) -> (String, Vec<String>) {
    let Some(deps) = manifest.dependencies.as_ref() else {
        return ("ok".to_string(), vec![]);
    };
    if deps.is_empty() {
        return ("ok".to_string(), vec![]);
    }

    let self_id = manifest.id.trim();
    let mut issues: Vec<String> = Vec::new();
    let mut any_missing = false;
    let mut any_mismatch = false;

    for (dep_id, range_str) in deps {
        let dep = dep_id.trim();
        if dep.is_empty() || dep == self_id {
            continue;
        }
        let req_s = range_str.trim();
        let req = match VersionReq::parse(req_s) {
            Ok(r) => r,
            Err(e) => {
                any_mismatch = true;
                issues.push(format!(
                    "Invalid semver range for dependency {}: {} ({})",
                    dep, req_s, e
                ));
                continue;
            }
        };
        match version_by_id.get(dep) {
            None => {
                any_missing = true;
                issues.push(format!("Missing dependency: {} (requires {})", dep, req_s));
            }
            Some(ver) => {
                if !req.matches(ver) {
                    any_mismatch = true;
                    issues.push(format!(
                        "Dependency version mismatch: {} requires {}, found {}",
                        dep, req_s, ver
                    ));
                }
            }
        }
    }

    let status = if any_missing {
        "missing"
    } else if any_mismatch {
        "mismatch"
    } else {
        "ok"
    };
    (status.to_string(), issues)
}
