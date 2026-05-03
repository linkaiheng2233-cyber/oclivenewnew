//! 插件市场索引 HTTP 拉取、校验与磁盘缓存（宿主无关路径参数）。

use crate::error::{AppError, Result};
use crate::infrastructure::blocking_http::block_on;
use crate::models::plugin_market_index::PluginIndexFile;
use oclive_validation::validate_plugin_market_index_v1;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_PLUGIN_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json";

#[must_use]
pub fn resolve_plugin_index_url(index_url: Option<&str>) -> String {
    let env_url = std::env::var("OCLIVE_PLUGIN_INDEX_URL").ok();
    index_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            env_url
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| DEFAULT_PLUGIN_INDEX_URL.to_string())
}

#[must_use]
pub fn plugin_index_default_cache_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("plugin_index_cache.json")
}

#[must_use]
pub fn plugin_index_cache_path_for_source(app_data_dir: &Path, source_url: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(source_url.trim().as_bytes());
    let digest = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    app_data_dir.join(format!("plugin_index_cache_{}.json", digest))
}

pub fn load_plugin_index_cache(cache_path: &Path) -> Result<PluginIndexFile> {
    if !cache_path.exists() {
        return Ok(PluginIndexFile {
            generated_at: None,
            plugins: Vec::new(),
        });
    }
    let raw = fs::read_to_string(cache_path).map_err(AppError::IoError)?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse plugin index cache failed: {}", e)))
}

pub fn sync_plugin_index_from_url(url: &str, cache_path: &Path) -> Result<PluginIndexFile> {
    let url = url.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("index http client failed: {}", e)))?;
    let text = block_on(async move {
        let resp = cli
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Unknown(format!("sync plugin index failed: {}", e)))?;
        if !resp.status().is_success() {
            return Err(AppError::Unknown(format!(
                "sync plugin index status={} url={}",
                resp.status(),
                url
            )));
        }
        resp.text()
            .await
            .map_err(|e| AppError::Unknown(format!("read plugin index response failed: {}", e)))
    })?;
    validate_plugin_market_index_v1(&text)
        .map_err(|e| AppError::Unknown(format!("plugins.json validate failed: {}", e)))?;
    let mut parsed: PluginIndexFile = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse plugins.json failed: {}", e)))?;
    parsed.plugins.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        cache_path,
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| AppError::Unknown(format!("encode index cache failed: {}", e)))?,
    )
    .map_err(AppError::IoError)?;
    Ok(parsed)
}
