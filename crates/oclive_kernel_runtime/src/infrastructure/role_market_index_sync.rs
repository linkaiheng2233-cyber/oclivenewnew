//! 角色市场索引 HTTP 拉取、校验与磁盘缓存。
//!
//! HTTP 使用 [`sync_role_market_index_from_url`] 的原生 `async`（不再经 `blocking_http::block_on`）。
//! 磁盘读写仍为同步 `std::fs`；宿主侧宜在 `spawn_blocking` 中调用本模块，或仅在网络层用 `async`。

use crate::error::{AppError, Result};
use crate::models::role_market_index::{RoleIndexDownload, RoleIndexEntry, RoleIndexFile};
use crate::utils::digest::sha256_hex;
use oclive_validation::{
    validate_role_market_index_v1, RoleMarketIndexEntryDisk, RoleMarketIndexFileDisk,
};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_ROLES_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-roles/main/roles.json";

#[must_use]
pub fn role_market_index_cache_path(app_data_dir: &Path, url: &str) -> PathBuf {
    let hex = sha256_hex(url.as_bytes());
    app_data_dir
        .join("cache")
        .join("role_market")
        .join(format!("roles-{}.json", &hex[..16]))
}

fn map_role_entry(d: RoleMarketIndexEntryDisk) -> RoleIndexEntry {
    RoleIndexEntry {
        entry_type: d.entry_type,
        id: d.id,
        name: d.name,
        description: d.description,
        author: d.author,
        version: d.version,
        min_runtime_version: d.min_runtime_version,
        tags: d.tags,
        downloads: d
            .downloads
            .into_iter()
            .map(|x| RoleIndexDownload {
                label: x.label,
                kind: x.kind,
                url: x.url,
                sha256: x.sha256,
                note: x.note,
                trust: x.trust,
            })
            .collect(),
    }
}

pub fn load_role_market_index_cache(cache_path: &Path) -> Result<RoleIndexFile> {
    if !cache_path.exists() {
        return Ok(RoleIndexFile {
            generated_at: None,
            roles: Vec::new(),
            warning: None,
        });
    }
    let raw = fs::read_to_string(cache_path).map_err(AppError::IoError)?;
    serde_json::from_str(&raw).map_err(AppError::from)
}

pub async fn sync_role_market_index_from_url(
    url: &str,
    cache_path: &Path,
) -> Result<RoleIndexFile> {
    let url = url.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            AppError::InvalidParameter(format!("[ROLE_INDEX_HTTP] client build: {}", e))
        })?;
    let resp = cli
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_INDEX_HTTP] get: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::InvalidParameter(format!(
            "[ROLE_INDEX_HTTP] status={} url={}",
            resp.status(),
            url
        )));
    }
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_INDEX_HTTP] read body: {}", e)))?;
    validate_role_market_index_v1(&text)
        .map_err(|e| AppError::InvalidParameter(format!("[ROLE_INDEX_VALIDATE] {}", e)))?;
    let parsed_disk: RoleMarketIndexFileDisk =
        serde_json::from_str(&text).map_err(AppError::from)?;
    let mut roles = parsed_disk
        .roles
        .into_iter()
        .map(map_role_entry)
        .collect::<Vec<_>>();
    roles.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(cache_path, &text).map_err(AppError::IoError)?;
    Ok(RoleIndexFile {
        generated_at: parsed_disk.generated_at,
        roles,
        warning: None,
    })
}
