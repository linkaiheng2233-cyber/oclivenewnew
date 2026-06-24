//! Resolve the on-disk `models/` directory (GGUF/BIN for local import), mirroring [`find_roles_dir`].

use oclive_kernel_runtime::{canonical_brand_app_data_dir, tauri_legacy_app_data_dir};
#[cfg(target_os = "windows")]
use oclive_kernel_runtime::TAURI_APP_IDENTIFIER;
use std::fs;
use std::path::{Path, PathBuf};

/// Override local models folder (`OCLIVE_MODELS_DIR`).
pub const ENV_MODELS_DIR: &str = "OCLIVE_MODELS_DIR";

/// When `roles/` lives at `{root}/roles`, bundled resources use `{root}` for `models/`.
#[must_use]
pub fn resource_dir_from_roles(roles_dir: &Path) -> Option<PathBuf> {
    if roles_dir.file_name().is_some_and(|n| n == "roles") {
        return roles_dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn brand_parent_for_app_data(app_data: &Path) -> PathBuf {
    app_data
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(canonical_brand_parent)
}

fn canonical_brand_parent() -> PathBuf {
    canonical_brand_app_data_dir()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(canonical_brand_app_data_dir)
}

#[cfg(debug_assertions)]
fn try_dev_models_dir() -> Option<PathBuf> {
    let from_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("models");
    if let Ok(canon) = from_manifest.canonicalize() {
        if canon.is_dir() {
            tracing::info!(
                target: "oclive_models",
                "find_models_dir: manifest-relative -> {}",
                canon.display()
            );
            return Some(canon);
        }
    }
    tracing::info!(
        target: "oclive_models",
        "find_models_dir: manifest-relative (ensure) -> {}",
        from_manifest.display()
    );
    Some(from_manifest)
}

#[cfg(not(debug_assertions))]
fn try_dev_models_dir() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..12 {
            let Some(dir) = cur else {
                break;
            };
            let candidate = dir.join("models");
            if candidate.is_dir() {
                tracing::info!(
                    target: "oclive_models",
                    "find_models_dir: near_exe -> {}",
                    candidate.display()
                );
                return Some(candidate);
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let a = cwd.join("models");
        if a.is_dir() {
            tracing::info!(
                target: "oclive_models",
                "find_models_dir: cwd/models -> {}",
                a.display()
            );
            return Some(a);
        }
        let b = cwd.join("..").join("models");
        if let Ok(canon) = b.canonicalize() {
            if canon.is_dir() {
                tracing::info!(
                    target: "oclive_models",
                    "find_models_dir: ../models -> {}",
                    canon.display()
                );
                return Some(canon);
            }
        }
    }
    None
}

/// Resolve `models/` for dev, packaged, and headless runs (same heuristics as `roles/`).
///
/// Priority: `OCLIVE_MODELS_DIR` -> (debug) repo dev paths -> `resource_dir/models` when
/// `resource_dir` is set -> exe/cwd heuristics -> relative `models/`.
#[must_use]
pub fn find_models_dir(resource_dir: Option<&Path>) -> PathBuf {
    if let Ok(custom) = std::env::var(ENV_MODELS_DIR) {
        let p = PathBuf::from(&custom);
        if p.is_dir() {
            tracing::info!(
                target: "oclive_models",
                "find_models_dir: OCLIVE_MODELS_DIR -> {}",
                p.display()
            );
            return p;
        }
        tracing::warn!(
            target: "oclive_models",
            "OCLIVE_MODELS_DIR is set but not a directory ({}); ignoring",
            custom
        );
    }

    #[cfg(debug_assertions)]
    if let Some(dev) = try_dev_models_dir() {
        return dev;
    }

    if let Some(res) = resource_dir {
        let bundled = res.join("models");
        if bundled.is_dir() {
            tracing::info!(
                target: "oclive_models",
                "find_models_dir: bundled -> {}",
                bundled.display()
            );
            return bundled;
        }
    }

    if let Some(dev) = try_dev_models_dir() {
        return dev;
    }

    let fallback = PathBuf::from("models");
    tracing::info!(
        target: "oclive_models",
        "find_models_dir: relative fallback -> {}",
        fallback.display()
    );
    fallback
}

/// Like [`find_models_dir`], creating the directory when missing.
pub fn ensure_models_dir(resource_dir: Option<&Path>) -> PathBuf {
    let dir = find_models_dir(resource_dir);
    if let Err(e) = fs::create_dir_all(&dir) {
        tracing::warn!(
            target: "oclive_models",
            path = %dir.display(),
            error = %e,
            "failed to create models directory"
        );
    }
    dir
}

/// Canonical models folder aligned with the given `roles/` root.
#[must_use]
pub fn ensure_models_dir_for_roles(roles_dir: &Path) -> PathBuf {
    ensure_models_dir(resource_dir_from_roles(roles_dir).as_deref())
}

#[must_use]
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => {
            a == b
                || a.to_string_lossy()
                    .eq_ignore_ascii_case(&b.to_string_lossy())
        }
    }
}

fn is_model_weight_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        let e = e.to_ascii_lowercase();
        e == "gguf" || e == "bin"
    })
}

/// Move/copy `*.gguf` / `*.bin` from `from` into `to` (skip when destination already exists).
#[must_use]
pub fn migrate_gguf_files(from: &Path, to: &Path) -> usize {
    if !from.is_dir() || paths_equal(from, to) {
        return 0;
    }
    if fs::create_dir_all(to).is_err() {
        return 0;
    }
    let Ok(rd) = fs::read_dir(from) else {
        return 0;
    };
    let mut moved = 0usize;
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() || !is_model_weight_file(&path) {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let dest = to.join(name);
        if dest.is_file() {
            continue;
        }
        if fs::rename(&path, &dest).is_ok() {
            moved += 1;
            continue;
        }
        if fs::copy(&path, &dest).is_ok() {
            let _ = fs::remove_file(&path);
            moved += 1;
        }
    }
    if moved > 0 {
        tracing::info!(
            target: "oclive_models",
            from = %from.display(),
            to = %to.display(),
            moved,
            "migrated local model files"
        );
    }
    moved
}

pub fn try_remove_empty_dir(dir: &Path) {
    if !dir.is_dir() {
        return;
    }
    match fs::remove_dir(dir) {
        Ok(()) => tracing::info!(
            target: "oclive_models",
            path = %dir.display(),
            "removed empty legacy models directory"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => tracing::debug!(
            target: "oclive_models",
            path = %dir.display(),
            error = %e,
            "legacy models directory not removed (not empty or in use)"
        ),
    }
}

/// Known pre-repo-root locations (app data / brand tree / legacy Tauri id).
#[must_use]
pub fn legacy_models_dir_candidates(app_data: &Path) -> Vec<PathBuf> {
    let brand = brand_parent_for_app_data(app_data);
    let mut out = vec![
        brand.join("models"),
        app_data.join("models"),
        tauri_legacy_app_data_dir().join("models"),
        canonical_brand_parent().join("models"),
    ];
    #[cfg(target_os = "windows")]
    if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        for name in [
            TAURI_APP_IDENTIFIER,
            "com.keven.oclive",
            "com.oclivenewnew.app",
        ] {
            out.push(local.join(name).join("models"));
        }
    }
    out.sort();
    out.dedup_by(|a, b| {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    });
    out
}

/// Whether a stored DB path should be replaced by the canonical repo-root `models/`.
#[must_use]
pub fn is_managed_legacy_models_path(stored: &Path, canonical: &Path, app_data: &Path) -> bool {
    if paths_equal(stored, canonical) {
        return false;
    }
    if !stored.exists() {
        return true;
    }
    for legacy in legacy_models_dir_candidates(app_data) {
        if paths_equal(stored, &legacy) {
            return true;
        }
    }
    let brand = brand_parent_for_app_data(app_data);
    let legacy_root = tauri_legacy_app_data_dir();
    if (stored.starts_with(&brand) || stored.starts_with(&legacy_root))
        && stored
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case("models"))
    {
        return true;
    }
    false
}

/// Move weights from a legacy folder into `canonical`, then remove the source dir when empty.
pub fn migrate_and_cleanup_models(from: &Path, canonical: &Path) {
    let _ = migrate_gguf_files(from, canonical);
    try_remove_empty_dir(from);
}

/// One-shot startup: migrate any known legacy folders into canonical `models/`.
pub fn reconcile_legacy_models_layout(canonical: &Path, app_data: &Path) {
    for legacy in legacy_models_dir_candidates(app_data) {
        if !paths_equal(&legacy, canonical) {
            migrate_and_cleanup_models(&legacy, canonical);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resource_dir_from_roles_parent() {
        let roles = PathBuf::from("/repo/roles");
        assert_eq!(
            resource_dir_from_roles(&roles),
            Some(PathBuf::from("/repo"))
        );
    }

    #[test]
    fn migrate_gguf_moves_files() {
        let tmp = tempdir().expect("tempdir");
        let from = tmp.path().join("old");
        let to = tmp.path().join("new");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("a.gguf"), b"gguf").unwrap();
        fs::write(from.join("readme.txt"), b"skip").unwrap();
        let n = migrate_gguf_files(&from, &to);
        assert_eq!(n, 1);
        assert!(to.join("a.gguf").is_file());
        assert!(from.join("readme.txt").is_file());
    }

    #[test]
    fn legacy_path_under_brand_models() {
        let tmp = tempdir().expect("tempdir");
        let canonical = tmp.path().join("repo").join("models");
        let app_data = tmp.path().join("OCLive").join("data");
        let legacy = tmp.path().join("OCLive").join("models");
        fs::create_dir_all(&legacy).unwrap();
        assert!(is_managed_legacy_models_path(
            &legacy, &canonical, &app_data
        ));
    }
}
