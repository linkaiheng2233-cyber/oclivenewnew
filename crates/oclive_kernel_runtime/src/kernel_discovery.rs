//! Kernel binary discovery SSOT — shared by desktop spawn, VS Code extension, and docs.
//!
//! Tier scores and promotion threshold must stay aligned with `oclive-vscode/src/discovery.ts`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Minimum candidate score to promote a dev binary into `%LOCALAPPDATA%/OCLive/runtime/`.
pub const PROMOTE_SCORE_THRESHOLD: u8 = 88;

pub const SCORE_ENV: u8 = 100;
pub const SCORE_DEV_FULL_DEBUG: u8 = 95;
pub const SCORE_DEV_FULL_RELEASE: u8 = 94;
pub const SCORE_DEV_HEADLESS_DEBUG: u8 = 90;
pub const SCORE_DEV_HEADLESS_RELEASE: u8 = 89;
pub const SCORE_SHARED: u8 = 88;
pub const SCORE_SETTINGS: u8 = 85;
pub const SCORE_BUNDLED: u8 = 50;

/// `%LOCALAPPDATA%/OCLive/runtime` (Windows) or platform equivalent parent + `runtime`.
#[must_use]
pub fn shared_runtime_dir() -> PathBuf {
    shared_runtime_parent().join("runtime")
}

fn shared_runtime_parent() -> PathBuf {
    crate::paths::canonical_brand_app_data_dir()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            {
                if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
                    return local.join("OCLive");
                }
            }
            crate::paths::canonical_brand_app_data_dir()
        })
}

/// Shared promoted kernel binary path.
#[must_use]
pub fn shared_kernel_binary_path() -> PathBuf {
    let name = if cfg!(windows) {
        "oclive-kernel-server.exe"
    } else {
        "oclive-kernel-server"
    };
    shared_runtime_dir().join(name)
}

/// Discovery tier label (mirrors VS Code `KernelTier`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KernelTier {
    Shared,
    DevFull,
    DevHeadless,
    Bundled,
    Settings,
    Env,
}

/// A candidate kernel binary sorted by [`KernelCandidate::score`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelCandidate {
    pub binary: PathBuf,
    pub tier: KernelTier,
    pub score: u8,
    /// e.g. `["--api"]` when the binary is the full Tauri host.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn kernel_exe(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn walk_parents(start: &Path, max: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut cur = start.to_path_buf();
    for _ in 0..max {
        out.push(cur.clone());
        let Some(parent) = cur.parent() else {
            break;
        };
        if parent == cur {
            break;
        }
        cur = parent.to_path_buf();
    }
    out
}

/// Find monorepo root containing `src-tauri/Cargo.toml` and `roles/`.
#[must_use]
pub fn find_monorepo_root(anchors: &[PathBuf]) -> Option<PathBuf> {
    for anchor in anchors {
        for dir in walk_parents(anchor, 8) {
            let marker = dir.join("src-tauri").join("Cargo.toml");
            let roles = dir.join("roles");
            if marker.is_file() && roles.is_dir() {
                return Some(dir);
            }
        }
    }
    None
}

fn dev_kernel_candidates(repo_root: &Path) -> Vec<KernelCandidate> {
    let mut out = Vec::new();
    let target_roots = [
        repo_root
            .join("..")
            .join("oclive-dev-artifacts")
            .join("oclivenewnew-cargo-target"),
        repo_root.join("target"),
        repo_root.join("..").join("target"),
    ];
    for root in target_roots {
        for (profile, tauri_score, headless_score) in [
            ("debug", SCORE_DEV_FULL_DEBUG, SCORE_DEV_HEADLESS_DEBUG),
            ("release", SCORE_DEV_FULL_RELEASE, SCORE_DEV_HEADLESS_RELEASE),
        ] {
            let tauri = root.join(profile).join(kernel_exe("oclivenewnew-tauri"));
            let headless = root.join(profile).join(kernel_exe("oclive-kernel-server"));
            if is_executable(&tauri) {
                out.push(KernelCandidate {
                    binary: tauri,
                    tier: KernelTier::DevFull,
                    score: tauri_score,
                    extra_args: vec!["--api".into()],
                });
            }
            if is_executable(&headless) {
                out.push(KernelCandidate {
                    binary: headless,
                    tier: KernelTier::DevHeadless,
                    score: headless_score,
                    extra_args: vec![],
                });
            }
        }
    }
    out
}

/// Collect kernel binary candidates (deduped by path, highest score wins).
#[must_use]
pub fn discover_kernel_candidates(
    anchors: &[PathBuf],
    settings_binary: Option<&Path>,
    bundled_binary: Option<&Path>,
) -> Vec<KernelCandidate> {
    let mut candidates = Vec::new();

    if let Ok(from_env) = std::env::var("OCLIVE_KERNEL_BINARY") {
        let p = PathBuf::from(from_env.trim());
        if is_executable(&p) {
            candidates.push(KernelCandidate {
                binary: p,
                tier: KernelTier::Env,
                score: SCORE_ENV,
                extra_args: vec![],
            });
        }
    }

    if let Some(p) = settings_binary.filter(|p| is_executable(p)) {
        candidates.push(KernelCandidate {
            binary: p.to_path_buf(),
            tier: KernelTier::Settings,
            score: SCORE_SETTINGS,
            extra_args: vec![],
        });
    }

    let shared = shared_kernel_binary_path();
    if is_executable(&shared) {
        candidates.push(KernelCandidate {
            binary: shared,
            tier: KernelTier::Shared,
            score: SCORE_SHARED,
            extra_args: vec![],
        });
    }

    if let Some(p) = bundled_binary.filter(|p| is_executable(p)) {
        candidates.push(KernelCandidate {
            binary: p.to_path_buf(),
            tier: KernelTier::Bundled,
            score: SCORE_BUNDLED,
            extra_args: vec![],
        });
    }

    if let Some(repo) = find_monorepo_root(anchors) {
        candidates.extend(dev_kernel_candidates(&repo));
    }

    let mut by_path: std::collections::HashMap<PathBuf, KernelCandidate> =
        std::collections::HashMap::new();
    for c in candidates {
        by_path
            .entry(c.binary.clone())
            .and_modify(|prev| {
                if c.score > prev.score {
                    *prev = c.clone();
                }
            })
            .or_insert(c);
    }
    let mut out: Vec<_> = by_path.into_values().collect();
    out.sort_by(|a, b| b.score.cmp(&a.score));
    out
}

#[must_use]
pub fn pick_best_kernel(candidates: &[KernelCandidate]) -> Option<&KernelCandidate> {
    candidates.first()
}

/// Copy `binary` into shared runtime dir; returns destination path on success.
///
/// # Errors
///
/// Returns I/O error message when copy fails.
pub fn promote_to_shared_runtime(binary: &Path) -> Result<PathBuf, String> {
    let dest = shared_kernel_binary_path();
    std::fs::create_dir_all(shared_runtime_dir()).map_err(|e| e.to_string())?;
    if binary.canonicalize().ok() == dest.canonicalize().ok() {
        return Ok(dest);
    }
    std::fs::copy(binary, &dest).map_err(|e| format!("promote kernel: {e}"))?;
    Ok(dest)
}

/// Whether this candidate should be promoted to shared runtime before spawn.
#[must_use]
pub fn should_promote(candidate: &KernelCandidate) -> bool {
    candidate.score >= PROMOTE_SCORE_THRESHOLD
        && !matches!(candidate.tier, KernelTier::Shared | KernelTier::Bundled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promote_threshold_is_88() {
        assert_eq!(PROMOTE_SCORE_THRESHOLD, 88);
    }

    #[test]
    fn shared_kernel_name() {
        let p = shared_kernel_binary_path();
        assert!(p.to_string_lossy().contains("runtime"));
    }
}
