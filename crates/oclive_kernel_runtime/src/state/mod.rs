// Stage 1 migration: `resolve_roles_dir` lives in this crate.
//
// Next step: migrate `AppState` and its dependencies here, then remove the dependency
// on `oclivenewnew-tauri` entirely.

use std::fs;
use std::path::{Path, PathBuf};

pub use oclivenewnew_tauri::state::AppState;

/// 自动发现时要求至少有一个「子目录 + manifest.json」，避免误用盘符根上空的 `D:\roles` 等。
fn roles_dir_has_any_role_pack(roles_root: &Path) -> bool {
    let Ok(rd) = fs::read_dir(roles_root) else {
        return false;
    };
    rd.flatten().any(|e| {
        let p = e.path();
        p.is_dir() && p.join("manifest.json").is_file()
    })
}

/// 开发时进程 cwd 可能是 `src-tauri/`，优先定位到项目根的 `roles/`。
/// 日志 target：`oclive_roles`（与桌面发行版保持一致，便于过滤）。
pub fn resolve_roles_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("OCLIVE_ROLES_DIR") {
        let p = PathBuf::from(&custom);
        if p.is_dir() {
            log::info!(
                target: "oclive_roles",
                "resolve_roles_dir: OCLIVE_ROLES_DIR -> {}",
                p.display()
            );
            return p;
        }
        log::warn!(
            target: "oclive_roles",
            "OCLIVE_ROLES_DIR is set but not a directory ({}); ignoring",
            custom
        );
    }
    #[cfg(debug_assertions)]
    {
        let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("roles");
        match from_manifest.canonicalize() {
            Ok(canon) if canon.is_dir() => {
                log::info!(
                    target: "oclive_roles",
                    "resolve_roles_dir: manifest-relative -> {}",
                    canon.display()
                );
                return canon;
            }
            _ => {
                if from_manifest.is_dir() {
                    log::info!(
                        target: "oclive_roles",
                        "resolve_roles_dir: manifest-relative (non-canon) -> {}",
                        from_manifest.display()
                    );
                    return from_manifest;
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
                log::info!(
                    target: "oclive_roles",
                    "resolve_roles_dir: near_exe -> {}",
                    candidate.display()
                );
                return candidate;
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let a = cwd.join("roles");
        if a.is_dir() && roles_dir_has_any_role_pack(&a) {
            log::info!(
                target: "oclive_roles",
                "resolve_roles_dir: cwd/roles -> {}",
                a.display()
            );
            return a;
        }
        let b = cwd.join("..").join("roles");
        if let Ok(canon) = b.canonicalize() {
            if canon.is_dir() && roles_dir_has_any_role_pack(&canon) {
                log::info!(
                    target: "oclive_roles",
                    "resolve_roles_dir: ../roles -> {}",
                    canon.display()
                );
                return canon;
            }
        }
    }
    let fallback = PathBuf::from("roles");
    log::info!(
        target: "oclive_roles",
        "resolve_roles_dir: relative fallback -> {}",
        fallback.display()
    );
    fallback
}
