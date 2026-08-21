//! Resolve the on-disk `roles/` directory for dev and packaged runs.

#[cfg(debug_assertions)]
use oclive_kernel_runtime::chat_pro_roles_dir;
use std::fs;
use std::path::{Path, PathBuf};

fn roles_dir_has_any_role_pack(roles_root: &Path) -> bool {
    let Ok(rd) = fs::read_dir(roles_root) else {
        return false;
    };
    rd.flatten().any(|e| {
        let p = e.path();
        p.is_dir()
            && (p.join("manifest.json").is_file()
                || p.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME)
                    .is_file())
    })
}

fn try_dev_roles_dir() -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if let Some(roles) = chat_pro_roles_dir(std::slice::from_ref(&manifest)) {
            if roles.is_dir() {
                tracing::info!(
                    target: "oclive_roles",
                    "find_roles_dir: monorepo chat-pro -> {}",
                    roles.display()
                );
                return Some(roles);
            }
        }
        let legacy = manifest.join("..").join("roles");
        match legacy.canonicalize() {
            Ok(canon) if canon.is_dir() => {
                tracing::info!(
                    target: "oclive_roles",
                    "find_roles_dir: manifest-relative legacy -> {}",
                    canon.display()
                );
                return Some(canon);
            }
            _ => {
                if legacy.is_dir() {
                    tracing::info!(
                        target: "oclive_roles",
                        "find_roles_dir: manifest-relative legacy (non-canon) -> {}",
                        legacy.display()
                    );
                    return Some(legacy);
                }
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..12 {
            let Some(dir) = cur else {
                break;
            };
            let candidate = dir.join("roles");
            if candidate.is_dir() && roles_dir_has_any_role_pack(&candidate) {
                tracing::info!(
                    target: "oclive_roles",
                    "find_roles_dir: near_exe -> {}",
                    candidate.display()
                );
                return Some(candidate);
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let a = cwd.join("roles");
        if a.is_dir() && roles_dir_has_any_role_pack(&a) {
            tracing::info!(
                target: "oclive_roles",
                "find_roles_dir: cwd/roles -> {}",
                a.display()
            );
            return Some(a);
        }
        let b = cwd.join("..").join("roles");
        if let Ok(canon) = b.canonicalize() {
            if canon.is_dir() && roles_dir_has_any_role_pack(&canon) {
                tracing::info!(
                    target: "oclive_roles",
                    "find_roles_dir: ../roles -> {}",
                    canon.display()
                );
                return Some(canon);
            }
        }
    }
    None
}

fn find_bundled_roles_dir(resource_dir: &Path) -> Option<PathBuf> {
    [
        resource_dir.join("resources").join("roles"),
        resource_dir.join("roles"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir() && roles_dir_has_any_role_pack(candidate))
}

/// Resolve `roles/` for dev, packaged, and headless runs.
///
/// Priority: `OCLIVE_ROLES_DIR` → (debug) repo dev paths → bundled roles under
/// `resource_dir` (flat or NSIS nested `resources/roles`) → exe/cwd heuristics →
/// relative `roles/`.
pub fn find_roles_dir(resource_dir: Option<&Path>) -> PathBuf {
    if let Ok(custom) = std::env::var("OCLIVE_ROLES_DIR") {
        let p = PathBuf::from(&custom);
        if p.is_dir() {
            tracing::info!(
                target: "oclive_roles",
                "find_roles_dir: OCLIVE_ROLES_DIR -> {}",
                p.display()
            );
            return p;
        }
        tracing::warn!(
            target: "oclive_roles",
            "OCLIVE_ROLES_DIR is set but not a directory ({}); ignoring",
            custom
        );
    }

    #[cfg(debug_assertions)]
    if let Some(dev) = try_dev_roles_dir() {
        return dev;
    }

    if let Some(res) = resource_dir {
        tracing::info!(
            target: "oclive_roles",
            "find_roles_dir: tauri resource_dir -> {}",
            res.display()
        );
        if let Some(bundled) = find_bundled_roles_dir(res) {
            tracing::info!(
                target: "oclive_roles",
                "find_roles_dir: bundled -> {}",
                bundled.display()
            );
            return bundled;
        }
        tracing::warn!(
            target: "oclive_roles",
            "bundled roles missing under resource_dir: {}",
            res.display()
        );
    }

    if let Some(dev) = try_dev_roles_dir() {
        return dev;
    }

    let fallback = PathBuf::from("roles");
    tracing::info!(
        target: "oclive_roles",
        "find_roles_dir: relative fallback -> {}",
        fallback.display()
    );
    fallback
}

#[cfg(test)]
mod tests {
    use super::find_bundled_roles_dir;
    use std::fs;

    fn write_minimal_role(roles: &std::path::Path) {
        let role = roles.join("mumu");
        fs::create_dir_all(&role).expect("create role fixture");
        fs::write(role.join("pipeline.ocblueprint"), "{}").expect("write role fixture");
    }

    #[test]
    fn bundled_roles_support_nsis_nested_resources_layout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("resources").join("roles");
        write_minimal_role(&nested);
        assert_eq!(find_bundled_roles_dir(temp.path()), Some(nested));
    }

    #[test]
    fn bundled_roles_prefer_nested_release_layout_over_legacy_hotfix() {
        let temp = tempfile::tempdir().expect("tempdir");
        let flat = temp.path().join("roles");
        let nested = temp.path().join("resources").join("roles");
        write_minimal_role(&flat);
        write_minimal_role(&nested);
        assert_eq!(find_bundled_roles_dir(temp.path()), Some(nested));
    }
}
