//! 高风险能力授权（持久化）：权限标识与 PLUGIN_V1 / `oclive_validation::plugin_permissions` 一致。
//!
//! - 生产默认强制校验；集成测 / 内存库构造见 [`HighRiskGrantStore::load`] 的 `enforce` 参数。
//! - 自动化或 CI 可设 `OCLIVE_SKIP_HIGH_RISK_GRANTS=1` 跳过（与 `OCLIVE_SKIP_STARTUP_HEALTH` 同类排障开关）。

use crate::env_flags;
use oclive_kernel_runtime::AppError;
use oclive_validation::{MCP_HTTP, MCP_STDIO, NETWORK_WILDCARD, PROCESS_SPAWN};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const FILE_NAME: &str = "high_risk_grants.json";

/// 磁盘 JSON 形状：`high_risk_grants.json` 顶层键为权限标识（见 PLUGIN_V1 §权限规范）。
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
    fn merge_legacy_key(set: &mut HashSet<String>, legacy: &HashSet<String>) {
        for id in legacy {
            set.insert(id.clone());
        }
    }

    fn from_json_value(v: Value) -> Self {
        let legacy =
            serde_json::from_value::<LegacyHighRiskGrantsFile>(v.clone()).unwrap_or_default();
        let mut raw = v;
        if let Value::Object(ref mut m) = raw {
            migrate_legacy_bucket(m, "mcp_http", MCP_HTTP);
            migrate_legacy_bucket(m, "mcp_stdio", MCP_STDIO);
            migrate_legacy_bucket(m, "directory_plugin_process_spawn", PROCESS_SPAWN);
        }
        let mut file: Self = serde_json::from_value(raw).unwrap_or_default();
        Self::merge_legacy_key(&mut file.mcp_http, &legacy.mcp_http);
        Self::merge_legacy_key(&mut file.mcp_stdio, &legacy.mcp_stdio);
        Self::merge_legacy_key(
            &mut file.process_spawn,
            &legacy.directory_plugin_process_spawn,
        );
        file
    }
}

fn migrate_legacy_bucket(
    map: &mut serde_json::Map<String, Value>,
    legacy_key: &str,
    spec_key: &str,
) {
    if map.contains_key(spec_key) {
        return;
    }
    if let Some(v) = map.remove(legacy_key) {
        map.insert(spec_key.to_string(), v);
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LegacyHighRiskGrantsFile {
    #[serde(default)]
    mcp_http: HashSet<String>,
    #[serde(default)]
    mcp_stdio: HashSet<String>,
    #[serde(default)]
    directory_plugin_process_spawn: HashSet<String>,
}

pub struct HighRiskGrantStore {
    app_data: PathBuf,
    inner: RwLock<HighRiskGrantsFile>,
    /// `false` 时不强制（`AppState::new_in_memory*` 等测试夹具）。
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
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                return HighRiskGrantsFile::from_json_value(v);
            }
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

    /// Remote / 出站 HTTP 前调用；未授权返回 [`AppError::HighRiskCapabilityNotGranted`]。
    ///
    /// # Errors
    ///
    /// 未授予 `network:*` 时返回 [`AppError::HighRiskCapabilityNotGranted`]。
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
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn grant_mcp_http(&self, server_id: &str) -> Result<(), String> {
        self.grant_bucket(MCP_HTTP, server_id, |f| &mut f.mcp_http)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn revoke_mcp_http(&self, server_id: &str) -> Result<(), String> {
        self.revoke_bucket(server_id, |f| &mut f.mcp_http)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn grant_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        self.grant_bucket(MCP_STDIO, server_id, |f| &mut f.mcp_stdio)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn revoke_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        self.revoke_bucket(server_id, |f| &mut f.mcp_stdio)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn grant_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.grant_process_spawn(plugin_id)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn revoke_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.revoke_process_spawn(plugin_id)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn grant_process_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.grant_bucket(PROCESS_SPAWN, plugin_id, |f| &mut f.process_spawn)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn revoke_process_spawn(&self, plugin_id: &str) -> Result<(), String> {
        self.revoke_bucket(plugin_id, |f| &mut f.process_spawn)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
    pub fn grant_network(&self, grant_id: &str) -> Result<(), String> {
        self.grant_bucket(NETWORK_WILDCARD, grant_id, |f| &mut f.network)
    }

    /// # Errors
    ///
    /// `id` 为空或 grants 文件持久化失败时返回 `Err(String)`。
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

/// 解析 Tauri `grant_*` / `revoke_*` 的 `kind`（规范标识 + 旧版别名）。
#[must_use]
pub fn normalize_grant_kind(kind: &str) -> Option<GrantKind> {
    match kind.trim() {
        MCP_HTTP | "mcp_http" => Some(GrantKind::McpHttp),
        MCP_STDIO | "mcp_stdio" => Some(GrantKind::McpStdio),
        PROCESS_SPAWN | "directory_plugin_process_spawn" => Some(GrantKind::ProcessSpawn),
        NETWORK_WILDCARD | "network" | "network_wildcard" => Some(GrantKind::Network),
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
    fn reads_legacy_grant_file() {
        let dir = tempdir().unwrap();
        let p = dir.path().join(FILE_NAME);
        fs::write(
            &p,
            r#"{
  "mcp_http": ["s1"],
  "mcp_stdio": ["s2"],
  "directory_plugin_process_spawn": ["p1"]
}"#,
        )
        .unwrap();
        let store = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
        assert!(store.is_mcp_http_granted("s1"));
        assert!(store.is_mcp_stdio_granted("s2"));
        assert!(store.is_process_spawn_granted("p1"));
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
    fn normalize_grant_kind_accepts_spec_and_legacy() {
        assert_eq!(normalize_grant_kind("mcp:http"), Some(GrantKind::McpHttp));
        assert_eq!(normalize_grant_kind("mcp_http"), Some(GrantKind::McpHttp));
        assert_eq!(
            normalize_grant_kind("process:spawn"),
            Some(GrantKind::ProcessSpawn)
        );
        assert_eq!(normalize_grant_kind("network:*"), Some(GrantKind::Network));
    }
}
