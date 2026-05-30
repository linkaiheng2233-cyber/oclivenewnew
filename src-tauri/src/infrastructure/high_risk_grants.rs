//! High-risk capability permission grants (persisted): permission ids align with PLUGIN_V1 / `oclive_validation::plugin_permissions`.
//!
//! - Production enforces grants by default; integration tests / in-memory DB fixtures use [`HighRiskGrantStore::load`] `enforce` parameter.
//! - Automation or CI may set `OCLIVE_SKIP_HIGH_RISK_GRANTS=1` to skip (same class of troubleshooting switch as `OCLIVE_SKIP_STARTUP_HEALTH`).

use crate::env_flags;
use oclive_kernel_runtime::AppError;
use oclive_validation::{MCP_HTTP, MCP_STDIO, NETWORK_WILDCARD, PROCESS_SPAWN};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const FILE_NAME: &str = "high_risk_grants.json";

/// On-disk JSON shape: `high_risk_grants.json` top-level keys are permission ids (see PLUGIN_V1 § permission spec).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighRiskGrantsFile {
    #[serde(default, rename = "mcp:http")]
    pub mcp_http: HashSet<String>,
    #[serde(default, rename = "mcp:stdio")]
    pub mcp_stdio: HashSet<String>,
    #[serde(default, rename = "process:spawn")]
    pub process_spawn: HashSet<String>,
    #[serde(default, rename = "network:*")]
    pub network: HashSet<String>,
}

impl HighRiskGrantsFile {
    fn from_json_str(raw: &str) -> Self {
        serde_json::from_str(raw).unwrap_or_default()
    }
}

pub struct HighRiskGrantStore {
    app_data: PathBuf,
    inner: RwLock<HighRiskGrantsFile>,
    /// When `false`, enforcement is disabled (`AppState::new_in_memory*` and similar test fixtures).
    enforce: bool,
}

impl HighRiskGrantStore {
    #[must_use]
    pub fn load(app_data: PathBuf, enforce: bool) -> Arc<Self> {
        let data = Self::read_disk(&app_data);
        Arc::new(Self {
            app_data,
            inner: RwLock::new(data),
            enforce,
        })
    }

    fn file_path(app_data: &Path) -> PathBuf {
        app_data.join(FILE_NAME)
    }

    fn read_disk(app_data: &Path) -> HighRiskGrantsFile {
        let p = Self::file_path(app_data);
        if let Ok(raw) = fs::read_to_string(&p) {
            return HighRiskGrantsFile::from_json_str(&raw);
        }
        HighRiskGrantsFile::default()
    }

    fn persist(&self, data: &HighRiskGrantsFile) -> Result<(), String> {
        let p = Self::file_path(&self.app_data);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create_dir_all {}: {}", parent.display(), e))?;
        }
        let raw = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(&p, raw).map_err(|e| format!("write {}: {}", p.display(), e))
    }

    fn enforcement_active(&self) -> bool {
        self.enforce && !env_flags::env_flag_enabled("OCLIVE_SKIP_HIGH_RISK_GRANTS")
    }

    #[must_use]
    pub fn snapshot(&self) -> HighRiskGrantsFile {
        self.inner.read().clone()
    }

    fn granted(set: &HashSet<String>, id: &str) -> bool {
        set.contains(id.trim())
    }

    #[must_use]
    pub fn is_mcp_http_granted(&self, server_id: &str) -> bool {
        !self.enforcement_active() || Self::granted(&self.inner.read().mcp_http, server_id)
    }

    #[must_use]
    pub fn is_mcp_stdio_granted(&self, server_id: &str) -> bool {
        !self.enforcement_active() || Self::granted(&self.inner.read().mcp_stdio, server_id)
    }

    #[must_use]
    pub fn is_directory_plugin_spawn_granted(&self, plugin_id: &str) -> bool {
        self.is_process_spawn_granted(plugin_id)
    }

    #[must_use]
    pub fn is_process_spawn_granted(&self, plugin_id: &str) -> bool {
        !self.enforcement_active() || Self::granted(&self.inner.read().process_spawn, plugin_id)
    }

    #[must_use]
    pub fn is_network_granted(&self, grant_id: &str) -> bool {
        !self.enforcement_active() || Self::granted(&self.inner.read().network, grant_id)
    }

    /// Call before Remote / outbound HTTP; returns [`AppError::HighRiskCapabilityNotGranted`] when not granted.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::HighRiskCapabilityNotGranted`] when `network:*` is not granted.
    pub fn require_network(&self, grant_id: &str) -> Result<(), AppError> {
        if self.is_network_granted(grant_id) {
            return Ok(());
        }
        Err(AppError::HighRiskCapabilityNotGranted {
            capability: NETWORK_WILDCARD.into(),
            id: grant_id.trim().to_string(),
        })
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn grant_mcp_http(&self, server_id: &str) -> Result<(), String> {
        self.grant_bucket(MCP_HTTP, server_id, |f| &mut f.mcp_http)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn revoke_mcp_http(&self, server_id: &str) -> Result<(), String> {
        self.revoke_bucket(server_id, |f| &mut f.mcp_http)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn grant_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        self.grant_bucket(MCP_STDIO, server_id, |f| &mut f.mcp_stdio)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn revoke_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        self.revoke_bucket(server_id, |f| &mut f.mcp_stdio)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn grant_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.grant_process_spawn(plugin_id)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn revoke_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.revoke_process_spawn(plugin_id)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn grant_process_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.grant_bucket(PROCESS_SPAWN, plugin_id, |f| &mut f.process_spawn)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn revoke_process_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.revoke_bucket(plugin_id, |f| &mut f.process_spawn)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn grant_network(&self, grant_id: &str) -> Result<(), String> {
        self.grant_bucket(NETWORK_WILDCARD, grant_id, |f| &mut f.network)
    }

    /// # Errors
    ///
    /// Returns `Err(String)` when `id` is empty or grants file persistence fails.
    pub fn revoke_network(&self, grant_id: &str) -> Result<(), String> {
        self.revoke_bucket(grant_id, |f| &mut f.network)
    }

    fn grant_bucket(
        &self,
        _capability: &str,
        id: &str,
        pick: impl FnOnce(&mut HighRiskGrantsFile) -> &mut HashSet<String>,
    ) -> Result<(), String> {
        let key = id.trim().to_string();
        if key.is_empty() {
            return Err("id required".to_string());
        }
        let mut w = self.inner.write();
        pick(&mut w).insert(key);
        self.persist(&w)
    }

    fn revoke_bucket(
        &self,
        id: &str,
        pick: impl FnOnce(&mut HighRiskGrantsFile) -> &mut HashSet<String>,
    ) -> Result<(), String> {
        let key = id.trim().to_string();
        let mut w = self.inner.write();
        pick(&mut w).remove(&key);
        self.persist(&w)
    }
}

/// Parses Tauri `grant_*` / `revoke_*` `kind` values (canonical permission ids).
#[must_use]
pub fn normalize_grant_kind(kind: &str) -> Option<GrantKind> {
    match kind.trim() {
        MCP_HTTP => Some(GrantKind::McpHttp),
        MCP_STDIO => Some(GrantKind::McpStdio),
        PROCESS_SPAWN => Some(GrantKind::ProcessSpawn),
        NETWORK_WILDCARD => Some(GrantKind::Network),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantKind {
    McpHttp,
    McpStdio,
    ProcessSpawn,
    Network,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn serde_uses_permission_spec_keys() {
        let mut f = HighRiskGrantsFile::default();
        f.process_spawn.insert("plug-a".into());
        let raw = serde_json::to_string(&f).unwrap();
        assert!(raw.contains("\"process:spawn\""));
        assert!(!raw.contains("directory_plugin_process_spawn"));
    }

    #[test]
    fn network_grant_enforced() {
        let dir = tempdir().unwrap();
        let store = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
        assert!(store.require_network("remote:plugin").is_err());
        store.grant_network("remote:plugin").unwrap();
        assert!(store.require_network("remote:plugin").is_ok());
    }

    #[test]
    fn normalize_grant_kind_accepts_spec_keys_only() {
        assert_eq!(normalize_grant_kind("mcp:http"), Some(GrantKind::McpHttp));
        assert_eq!(normalize_grant_kind("mcp_http"), None);
        assert_eq!(
            normalize_grant_kind("process:spawn"),
            Some(GrantKind::ProcessSpawn)
        );
        assert_eq!(normalize_grant_kind("network:*"), Some(GrantKind::Network));
    }
}
