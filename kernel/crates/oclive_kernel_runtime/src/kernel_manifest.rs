//! Kernel binary manifest — version and capability identity for promote/compare (P2a).

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Build profile label (logical seed vs shared full build).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelBuildProfile {
    Full,
    Bundled,
}

impl KernelBuildProfile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Bundled => "bundled",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("bundled") {
            Self::Bundled
        } else {
            Self::Full
        }
    }
}

/// Structured identity of a kernel binary (CLI, `/health`, sidecar JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KernelBinaryManifest {
    /// Crate / product version (e.g. `0.2.0`).
    pub version: String,
    pub build_profile: String,
    #[serde(default)]
    pub feature_set: Vec<String>,
    pub built_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub runtime_api_version: String,
}

impl KernelBinaryManifest {
    /// Manifest for the currently running binary when compile-time env vars are set.
    #[must_use]
    pub fn from_compile_time_env() -> Self {
        let build_profile = option_env!("OCLIVE_KERNEL_BUILD_PROFILE").unwrap_or("full");
        let feature_set = default_feature_set(build_profile);
        Self {
            version: option_env!("OCLIVE_KERNEL_PKG_VERSION")
                .unwrap_or("0.2.0")
                .to_string(),
            build_profile: build_profile.to_string(),
            feature_set,
            built_at: option_env!("OCLIVE_KERNEL_BUILT_AT")
                .unwrap_or("")
                .to_string(),
            git_commit: option_env!("OCLIVE_KERNEL_GIT_COMMIT").map(str::to_string),
            runtime_api_version: crate::RUNTIME_API_VERSION.to_string(),
        }
    }

    /// Sidecar file name next to `oclive-kernel-server(.exe)`.
    #[must_use]
    pub fn sidecar_filename() -> &'static str {
        "oclive-kernel-server.oclive-manifest.json"
    }

    /// Path to manifest sidecar for a kernel binary path.
    #[must_use]
    pub fn sidecar_path_for_binary(binary: &Path) -> std::path::PathBuf {
        let parent = binary.parent().unwrap_or_else(|| Path::new("."));
        parent.join(Self::sidecar_filename())
    }

    /// Read sidecar if present.
    #[must_use]
    pub fn read_sidecar(binary: &Path) -> Option<Self> {
        let p = Self::sidecar_path_for_binary(binary);
        let raw = std::fs::read_to_string(p).ok()?;
        serde_json::from_str(&raw).ok()
    }

    /// Write sidecar next to binary (used after promote).
    ///
    /// # Errors
    ///
    /// Returns I/O or JSON error message.
    pub fn write_sidecar(&self, binary: &Path) -> Result<(), String> {
        let p = Self::sidecar_path_for_binary(binary);
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(p, json).map_err(|e| e.to_string())
    }

    /// Synthetic manifest when sidecar / compile-time env is unavailable.
    #[must_use]
    pub fn synthetic(build_profile: &str, version: &str) -> Self {
        Self {
            version: version.to_string(),
            build_profile: build_profile.to_string(),
            feature_set: default_feature_set(build_profile),
            built_at: String::new(),
            git_commit: None,
            runtime_api_version: crate::RUNTIME_API_VERSION.to_string(),
        }
    }

    /// Compare capability: fuller `feature_set` → semver → `built_at`.
    #[must_use]
    pub fn cmp_for_capability(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let self_set: std::collections::HashSet<_> = self.feature_set.iter().collect();
        let other_set: std::collections::HashSet<_> = other.feature_set.iter().collect();
        let self_superset = other_set.iter().all(|f| self_set.contains(*f));
        let other_superset = self_set.iter().all(|f| other_set.contains(*f));

        if self_set.len() > other_set.len() && self_superset {
            return Ordering::Greater;
        }
        if other_set.len() > self_set.len() && other_superset {
            return Ordering::Less;
        }
        if self_set.len() != other_set.len() {
            return self_set.len().cmp(&other_set.len());
        }
        match semver_cmp(&self.version, &other.version) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.built_at.cmp(&other.built_at),
        }
    }

    /// Compare for promote decisions: capability first, then semver, then `built_at`.
    #[must_use]
    pub fn cmp_for_promote(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_for_capability(other)
    }
}

#[must_use]
fn default_feature_set(build_profile: &str) -> Vec<String> {
    if build_profile == "bundled" {
        vec![
            "chat".into(),
            "role_load".into(),
            "memory".into(),
            "ollama_llm".into(),
        ]
    } else {
        vec![
            "chat".into(),
            "role_load".into(),
            "memory".into(),
            "emotion".into(),
            "event".into(),
            "prompt".into(),
            "llm".into(),
            "agent".into(),
            "complex_emotion".into(),
            "http_api".into(),
        ]
    }
}

fn semver_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u64> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let pb: Vec<u64> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    for i in 0..pa.len().max(pb.len()) {
        let va = pa.get(i).copied().unwrap_or(0);
        let vb = pb.get(i).copied().unwrap_or(0);
        match va.cmp(&vb) {
            std::cmp::Ordering::Equal => {}
            o => return o,
        }
    }
    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_json() {
        let m = KernelBinaryManifest {
            version: "0.2.0".into(),
            build_profile: "full".into(),
            feature_set: vec!["chat".into()],
            built_at: "2026-01-01T00:00:00Z".into(),
            git_commit: Some("abc1234".into()),
            runtime_api_version: "0.2.0".into(),
        };
        let j = serde_json::to_string(&m).unwrap();
        let back: KernelBinaryManifest = serde_json::from_str(&j).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn feature_set_orders_full_over_bundled() {
        let full = KernelBinaryManifest::synthetic("full", "0.3.0");
        let bundled = KernelBinaryManifest::synthetic("bundled", "0.3.0");
        assert_eq!(full.cmp_for_promote(&bundled), std::cmp::Ordering::Greater);
    }

    #[test]
    fn semver_cmp_orders() {
        let a = KernelBinaryManifest {
            version: "0.2.1".into(),
            build_profile: "full".into(),
            feature_set: vec![],
            built_at: "".into(),
            git_commit: None,
            runtime_api_version: "0.2.0".into(),
        };
        let b = KernelBinaryManifest {
            version: "0.2.0".into(),
            ..a.clone()
        };
        assert_eq!(a.cmp_for_promote(&b), std::cmp::Ordering::Greater);
    }
}
