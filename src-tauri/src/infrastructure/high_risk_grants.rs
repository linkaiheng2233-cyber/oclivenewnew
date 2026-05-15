//! 高风险能力授权（持久化）：MCP `http` / `stdio` 出站与子进程、目录插件 `process` 子进程。
//!
//! - 生产默认强制校验；集成测 / 内存库构造见 [`HighRiskGrantStore::load`] 的 `enforce` 参数。
//! - 自动化或 CI 可设 `OCLIVE_SKIP_HIGH_RISK_GRANTS=1` 跳过（与 `OCLIVE_SKIP_STARTUP_HEALTH` 同类排障开关）。

use crate::env_flags;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const FILE_NAME: &str = "high_risk_grants.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighRiskGrantsFile {
    #[serde(default)]
    pub mcp_http: HashSet<String>,
    #[serde(default)]
    pub mcp_stdio: HashSet<String>,
    /// 目录插件 manifest 含 `process`、需宿主 `spawn` 握手者。
    #[serde(default)]
    pub directory_plugin_process_spawn: HashSet<String>,
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
            return serde_json::from_str(&raw).unwrap_or_default();
        }
        HighRiskGrantsFile::default()
    }

    fn persist(&self, data: &HighRiskGrantsFile) -> Result<(), String> {
        let p = Self::file_path(&self.app_data);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create_dir_all {}: {}", parent.display(), e))?;
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

    #[must_use]
    pub fn is_mcp_http_granted(&self, server_id: &str) -> bool {
        !self.enforcement_active()
            || self
                .inner
                .read()
                .mcp_http
                .contains(server_id.trim())
    }

    #[must_use]
    pub fn is_mcp_stdio_granted(&self, server_id: &str) -> bool {
        !self.enforcement_active()
            || self
                .inner
                .read()
                .mcp_stdio
                .contains(server_id.trim())
    }

    #[must_use]
    pub fn is_directory_plugin_spawn_granted(&self, plugin_id: &str) -> bool {
        !self.enforcement_active()
            || self
                .inner
                .read()
                .directory_plugin_process_spawn
                .contains(plugin_id.trim())
    }

    pub fn grant_mcp_http(&self, server_id: &str) -> Result<(), String> {
        let sid = server_id.trim().to_string();
        if sid.is_empty() {
            return Err("server_id required".to_string());
        }
        let mut w = self.inner.write();
        w.mcp_http.insert(sid);
        self.persist(&w)
    }

    pub fn revoke_mcp_http(&self, server_id: &str) -> Result<(), String> {
        let sid = server_id.trim().to_string();
        let mut w = self.inner.write();
        w.mcp_http.remove(&sid);
        self.persist(&w)
    }

    pub fn grant_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        let sid = server_id.trim().to_string();
        if sid.is_empty() {
            return Err("server_id required".to_string());
        }
        let mut w = self.inner.write();
        w.mcp_stdio.insert(sid);
        self.persist(&w)
    }

    pub fn revoke_mcp_stdio(&self, server_id: &str) -> Result<(), String> {
        let sid = server_id.trim().to_string();
        let mut w = self.inner.write();
        w.mcp_stdio.remove(&sid);
        self.persist(&w)
    }

    pub fn grant_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        let id = plugin_id.trim().to_string();
        if id.is_empty() {
            return Err("plugin_id required".to_string());
        }
        let mut w = self.inner.write();
        w.directory_plugin_process_spawn.insert(id);
        self.persist(&w)
    }

    pub fn revoke_directory_plugin_spawn(&self, plugin_id: &str) -> Result<(), String> {
        let id = plugin_id.trim().to_string();
        let mut w = self.inner.write();
        w.directory_plugin_process_spawn.remove(&id);
        self.persist(&w)
    }
}
