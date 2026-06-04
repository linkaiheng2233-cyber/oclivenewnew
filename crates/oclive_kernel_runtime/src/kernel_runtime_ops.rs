//! Shared runtime promote / backup / rollback (P3a).

use crate::kernel_discovery::{
    promote_to_shared_runtime, shared_kernel_binary_path, shared_runtime_dir, should_promote,
    KernelCandidate, KernelTier, SCORE_SHARED,
};
use crate::kernel_manifest::KernelBinaryManifest;
use std::path::{Path, PathBuf};

const MAX_BACKUPS: usize = 3;

/// After discovery, optionally promote into shared runtime (backup + manifest sidecar).
pub fn apply_promote_to_candidate(candidate: &mut KernelCandidate) {
    if !should_promote(candidate) {
        return;
    }
    let manifest = KernelBinaryManifest::read_sidecar(&candidate.binary);
    match promote_with_backup(&candidate.binary, manifest.as_ref()) {
        Ok(report) => {
            candidate.binary = report.dest;
            candidate.tier = KernelTier::Shared;
            candidate.score = SCORE_SHARED;
            candidate.extra_args.clear();
        }
        Err(e) if e.contains("same or newer") => {
            candidate.binary = shared_kernel_binary_path();
            candidate.tier = KernelTier::Shared;
            candidate.score = SCORE_SHARED;
            candidate.extra_args.clear();
        }
        Err(e) => {
            tracing::warn!(
                target: "oclive_desktop",
                error = %e,
                "promote_with_backup failed; spawning from original candidate"
            );
        }
    }
}

/// Result of a promote operation.
#[derive(Debug, Clone)]
pub struct PromoteReport {
    pub dest: PathBuf,
    pub backup_dir: Option<PathBuf>,
}

fn backups_dir() -> PathBuf {
    shared_runtime_dir().join("backups")
}

/// List backup directories (newest first by name / timestamp folder).
#[must_use]
pub fn list_runtime_backups() -> Vec<PathBuf> {
    let root = backups_dir();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    dirs
}

fn backup_current_shared() -> Result<Option<PathBuf>, String> {
    let shared = shared_kernel_binary_path();
    if !shared.is_file() {
        return Ok(None);
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dir = backups_dir().join(format!("{stamp}"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = shared
        .file_name()
        .map(|n| n.to_owned())
        .unwrap_or_else(|| std::ffi::OsString::from("oclive-kernel-server"));
    let dest_bin = dir.join(name);
    std::fs::copy(&shared, &dest_bin).map_err(|e| format!("backup copy: {e}"))?;
    if let Some(m) = KernelBinaryManifest::read_sidecar(&shared) {
        let _ = m.write_sidecar(&dest_bin);
    }
    prune_old_backups();
    Ok(Some(dir))
}

fn prune_old_backups() {
    let mut backups = list_runtime_backups();
    while backups.len() > MAX_BACKUPS {
        if let Some(old) = backups.pop() {
            let _ = std::fs::remove_dir_all(old);
        }
    }
}

/// Whether `candidate` should replace the shared runtime binary (manifest first, always promote if no shared).
#[must_use]
pub fn should_promote_binary(_candidate: &Path, candidate_manifest: Option<&KernelBinaryManifest>) -> bool {
    let shared = shared_kernel_binary_path();
    if !shared.is_file() {
        return true;
    }
    let Some(cand_m) = candidate_manifest else {
        return true;
    };
    let Some(shared_m) = KernelBinaryManifest::read_sidecar(&shared) else {
        return true;
    };
    matches!(cand_m.cmp_for_promote(&shared_m), std::cmp::Ordering::Greater)
}

/// Copy `binary` into shared runtime after backing up the previous file.
///
/// # Errors
///
/// Returns I/O or promote error message.
pub fn promote_with_backup(
    binary: &Path,
    manifest: Option<&KernelBinaryManifest>,
) -> Result<PromoteReport, String> {
    if !should_promote_binary(binary, manifest) {
        return Err("shared runtime already has same or newer kernel".into());
    }
    let backup_dir = backup_current_shared()?;
    let dest = promote_to_shared_runtime(binary)?;
    let m = manifest
        .cloned()
        .unwrap_or_else(KernelBinaryManifest::from_compile_time_env);
    m.write_sidecar(&dest)?;
    tracing::info!(
        target: "oclive_desktop",
        dest = %dest.display(),
        backup = ?backup_dir,
        "kernel promoted to shared runtime"
    );
    Ok(PromoteReport { dest, backup_dir })
}

/// Restore the newest backup into shared runtime.
///
/// # Errors
///
/// Returns error when no backup or copy fails.
pub fn rollback_shared_kernel() -> Result<PathBuf, String> {
    let backups = list_runtime_backups();
    let Some(latest) = backups.first() else {
        return Err("no kernel backups in runtime/backups/".into());
    };
    let entries: Vec<PathBuf> = std::fs::read_dir(latest)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    let binary = entries
        .into_iter()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("oclive-kernel-server") || n.contains("kernel-server"))
        })
        .ok_or_else(|| "backup folder has no kernel binary".to_string())?;
    let _ = backup_current_shared();
    let dest = promote_to_shared_runtime(&binary)?;
    if let Some(m) = KernelBinaryManifest::read_sidecar(&binary) {
        m.write_sidecar(&dest)?;
    }
    tracing::info!(
        target: "oclive_desktop",
        from = %latest.display(),
        dest = %dest.display(),
        "kernel rolled back from backup"
    );
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backups_dir_under_runtime() {
        let b = backups_dir();
        assert!(b.to_string_lossy().contains("runtime"));
        assert!(b.ends_with("backups"));
    }
}
