//! 目录插件 manifest `dependencies` 与已安装版本比对。

use super::manifest::OclivePluginManifest;
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// 返回 `(dependency_status, dependency_issues)`，`status` 为 `ok` / `missing` / `mismatch`。
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
                    "依赖 {} 的版本范围无效: {} ({})",
                    dep, req_s, e
                ));
                continue;
            }
        };
        match version_by_id.get(dep) {
            None => {
                any_missing = true;
                issues.push(format!("依赖缺失: {}（需要 {}）", dep, req_s));
            }
            Some(ver) => {
                if !req.matches(ver) {
                    any_mismatch = true;
                    issues.push(format!(
                        "依赖版本不匹配: {} 需要 {}，实际 {}",
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
