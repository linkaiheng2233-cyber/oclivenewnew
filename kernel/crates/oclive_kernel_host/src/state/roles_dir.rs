//! Resolve the on-disk `roles/` directory for dev and packaged runs.

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
        let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("roles");
        match from_manifest.canonicalize() {
            Ok(canon) if canon.is_dir() => {
                tracing::info!(
                    target: "oclive_roles",
                    "find_roles_dir: manifest-relative -> {}",
                    canon.display()
                );
                return Some(canon);
            }
            _ => {
                if from_manifest.is_dir() {
                    tracing::info!(
                        target: "oclive_roles",
                        "find_roles_dir: manifest-relative (non-canon) -> {}",
                        from_manifest.display()
                    );
                    return Some(from_manifest);
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

/// Resolve `roles/` for dev, packaged, and headless runs.
///
/// Priority: `OCLIVE_ROLES_DIR` → (debug) repo dev paths → `resource_dir/roles` when
/// `resource_dir` is set → exe/cwd heuristics → relative `roles/`.
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
        let bundled = res.join("roles");
        if bundled.is_dir() {
            tracing::info!(
                target: "oclive_roles",
                "find_roles_dir: bundled -> {}",
                bundled.display()
            );
            return bundled;
        }
        tracing::warn!(
            target: "oclive_roles",
            "resource_dir/roles missing or not a directory: {}",
            bundled.display()
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
