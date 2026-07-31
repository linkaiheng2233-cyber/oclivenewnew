//! Runtime `/health` distro fields — process env snapshot (P2a+).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const ENV_DISTRO_ID: &str = "OCLIVE_DISTRO_ID";
pub const ENV_DISTRO_PROFILE: &str = "OCLIVE_DISTRO_PROFILE";

/// Distro identity exposed on `GET /health` (JSON).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistroHealthSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distro_profile_hash: Option<String>,
}

/// Read `OCLIVE_DISTRO_ID` and SHA-256 hex of `OCLIVE_DISTRO_PROFILE` file when set.
#[must_use]
pub fn distro_health_snapshot() -> DistroHealthSnapshot {
    let distro_id = std::env::var(ENV_DISTRO_ID)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let distro_profile_hash = std::env::var(ENV_DISTRO_PROFILE)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .and_then(|p| hash_file_sha256_hex(Path::new(&p)));
    DistroHealthSnapshot {
        distro_id,
        distro_profile_hash,
    }
}

fn hash_file_sha256_hex(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Some(format!("{:x}", hasher.finalize()))
}

/// SHA-256 hex of a distro profile file (caller-side hash for policy).
#[must_use]
pub fn profile_file_sha256_hex(path: &Path) -> Option<String> {
    hash_file_sha256_hex(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::Mutex;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct EnvRestore {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvRestore {
        fn capture(key: &'static str) -> Self {
            Self {
                key,
                previous: std::env::var_os(key),
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn profile_hash_stable_for_same_file() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        let _profile_restore = EnvRestore::capture(ENV_DISTRO_PROFILE);
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join("distro.oclive.toml");
        std::fs::write(&profile, b"distro_id = \"vscode\"\n").unwrap();
        std::env::set_var(ENV_DISTRO_PROFILE, profile.display().to_string());
        let a = distro_health_snapshot();
        let b = distro_health_snapshot();
        assert_eq!(a.distro_profile_hash, b.distro_profile_hash);
        assert!(a.distro_profile_hash.is_some());
    }

    #[test]
    fn empty_distro_id_omitted() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        let _id_restore = EnvRestore::capture(ENV_DISTRO_ID);
        let _profile_restore = EnvRestore::capture(ENV_DISTRO_PROFILE);
        std::env::remove_var(ENV_DISTRO_ID);
        std::env::remove_var(ENV_DISTRO_PROFILE);
        let snap = distro_health_snapshot();
        assert!(snap.distro_id.is_none());
        assert!(snap.distro_profile_hash.is_none());
    }
}
