//! 插件评价索引 HTTP 拉取、校验与磁盘缓存。
//!
//! HTTP 为原生 `async`。

use crate::error::{AppError, Result};
use crate::models::plugin_reviews_index::PluginReviewsIndexFile;
use oclive_validation::validate_plugin_reviews_index_v1;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_PLUGIN_REVIEWS_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugin-reviews/main/reviews.json";

#[must_use]
pub fn plugin_reviews_index_default_cache_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("plugin_reviews_cache.json")
}

#[must_use]
pub fn resolve_plugin_reviews_index_url(url_override: Option<&str>) -> String {
    let env_url = std::env::var("OCLIVE_PLUGIN_REVIEWS_INDEX_URL").ok();
    url_override
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
        .unwrap_or_else(|| DEFAULT_PLUGIN_REVIEWS_INDEX_URL.to_string())
}

pub fn load_plugin_reviews_index_cache(cache_path: &Path) -> Result<PluginReviewsIndexFile> {
    if !cache_path.exists() {
        return Ok(PluginReviewsIndexFile {
            schema_version: 1,
            generated_at: None,
            reviews: Vec::new(),
        });
    }
    let raw = fs::read_to_string(cache_path).map_err(AppError::IoError)?;
    serde_json::from_str(&raw).map_err(AppError::from)
}

pub async fn sync_plugin_reviews_index_from_url(
    url: &str,
    cache_path: &Path,
) -> Result<PluginReviewsIndexFile> {
    let url = url.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            AppError::InvalidParameter(format!("[PLUGIN_REVIEWS_HTTP] client build: {}", e))
        })?;
    let resp = cli
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_REVIEWS_HTTP] get: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::InvalidParameter(format!(
            "[PLUGIN_REVIEWS_HTTP] status={} url={}",
            resp.status(),
            url
        )));
    }
    let text = resp.text().await.map_err(|e| {
        AppError::InvalidParameter(format!("[PLUGIN_REVIEWS_HTTP] read body: {}", e))
    })?;

    validate_plugin_reviews_index_v1(&text)
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_REVIEWS_VALIDATE] {}", e)))?;

    let raw: serde_json::Value = serde_json::from_str(&text).map_err(AppError::from)?;
    let mut parsed: PluginReviewsIndexFile =
        serde_json::from_value(raw.clone()).map_err(AppError::from)?;

    for r in parsed.reviews.iter_mut() {
        if r.author_github
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
        {
            r.author_github = None;
        }
    }
    parsed.reviews.sort_by(|a, b| a.id.cmp(&b.id));

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        cache_path,
        serde_json::to_string_pretty(&parsed).map_err(AppError::from)?,
    )
    .map_err(AppError::IoError)?;

    Ok(parsed)
}
