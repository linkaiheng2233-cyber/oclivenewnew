//! 插件评价索引 HTTP 拉取、校验与磁盘缓存。

use crate::error::{AppError, Result};
use crate::infrastructure::blocking_http::block_on;
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
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse reviews.json cache failed: {}", e)))
}

pub fn sync_plugin_reviews_index_from_url(
    url: &str,
    cache_path: &Path,
) -> Result<PluginReviewsIndexFile> {
    let url = url.to_string();
    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("reviews http client failed: {}", e)))?;
    let text = block_on(async move {
        let resp = cli
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Unknown(format!("sync reviews index failed: {}", e)))?;
        if !resp.status().is_success() {
            return Err(AppError::Unknown(format!(
                "sync reviews index status={} url={}",
                resp.status(),
                url
            )));
        }
        resp.text()
            .await
            .map_err(|e| AppError::Unknown(format!("read reviews index response failed: {}", e)))
    })?;

    validate_plugin_reviews_index_v1(&text)
        .map_err(|e| AppError::Unknown(format!("reviews.json validate failed: {}", e)))?;

    let raw: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse reviews.json failed: {}", e)))?;
    let mut parsed: PluginReviewsIndexFile = serde_json::from_value(raw.clone())
        .map_err(|e| AppError::Unknown(format!("parse reviews.json dto failed: {}", e)))?;

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
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| AppError::Unknown(format!("encode reviews cache failed: {}", e)))?,
    )
    .map_err(AppError::IoError)?;

    Ok(parsed)
}
