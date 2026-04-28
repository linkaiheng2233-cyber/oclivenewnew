use crate::error::AppError;
use oclive_validation::validate_plugin_reviews_index_v1;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub const DEFAULT_PLUGIN_REVIEWS_INDEX_URL: &str = "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugin-reviews/main/reviews.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewsIndexFile {
    pub schema_version: i32,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub reviews: Vec<PluginReviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewEntry {
    pub id: String,
    pub plugin_id: String,
    #[serde(default)]
    pub pubkey_id: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    pub rating: i32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub author_github: Option<String>,
}

fn cache_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("plugin_reviews_cache.json")
}

pub fn load_cached_plugin_reviews_index(
    app_data_dir: &Path,
) -> Result<PluginReviewsIndexFile, AppError> {
    let p = cache_path(app_data_dir);
    if !p.exists() {
        return Ok(PluginReviewsIndexFile {
            schema_version: 1,
            generated_at: None,
            reviews: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&p)?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse reviews.json cache failed: {}", e)))
}

pub fn sync_plugin_reviews_index_online(
    app_data_dir: &Path,
    url: Option<&str>,
) -> Result<PluginReviewsIndexFile, AppError> {
    let env_url = std::env::var("OCLIVE_PLUGIN_REVIEWS_INDEX_URL").ok();
    let u = url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| env_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .unwrap_or(DEFAULT_PLUGIN_REVIEWS_INDEX_URL);

    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("reviews http client failed: {}", e)))?;
    let resp = cli
        .get(u)
        .send()
        .map_err(|e| AppError::Unknown(format!("sync reviews index failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::Unknown(format!(
            "sync reviews index status={} url={}",
            resp.status(),
            u
        )));
    }
    let text = resp
        .text()
        .map_err(|e| AppError::Unknown(format!("read reviews index response failed: {}", e)))?;

    validate_plugin_reviews_index_v1(&text)
        .map_err(|e| AppError::Unknown(format!("reviews.json validate failed: {}", e)))?;

    let raw: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse reviews.json failed: {}", e)))?;
    let mut parsed: PluginReviewsIndexFile = serde_json::from_value(raw.clone())
        .map_err(|e| AppError::Unknown(format!("parse reviews.json dto failed: {}", e)))?;

    // Normalize author github field for easier frontend usage.
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

    let p = cache_path(app_data_dir);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        &p,
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| AppError::Unknown(format!("encode reviews cache failed: {}", e)))?,
    )?;

    Ok(parsed)
}
