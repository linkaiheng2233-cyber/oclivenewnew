use serde::Deserialize;

/// `reviews.json` disk DTO (schema v1).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewsIndexFileDisk {
    pub schema_version: i32,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub reviews: Vec<PluginReviewEntryDisk>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewEntryDisk {
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
    pub author: Option<PluginReviewAuthorDisk>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginReviewAuthorDisk {
    #[serde(default)]
    pub github: Option<String>,
}

pub fn validate_plugin_reviews_index_v1(text: &str) -> Result<(), String> {
    let f: PluginReviewsIndexFileDisk =
        serde_json::from_str(text).map_err(|e| format!("reviews.json：解析失败：{}", e))?;
    if f.schema_version != 1 {
        return Err(format!(
            "reviews.json：schemaVersion={} 不受支持（仅支持 1）",
            f.schema_version
        ));
    }
    if f.reviews.len() > 200_000 {
        return Err("reviews.json：reviews 条目过多（>200000）".into());
    }
    for (i, r) in f.reviews.iter().enumerate() {
        let id = r.id.trim();
        if id.is_empty() {
            return Err(format!("reviews.json：reviews[{}].id 不能为空", i));
        }
        let pid = r.plugin_id.trim();
        if pid.is_empty() {
            return Err(format!("reviews.json：reviews[{}].pluginId 不能为空", i));
        }
        if let Some(pk) = r.pubkey_id.as_ref() {
            if pk.trim().is_empty() {
                return Err(format!(
                    "reviews.json：reviews[{}].pubkeyId 若提供则不能为空字符串",
                    i
                ));
            }
        }
        if let Some(v) = r.version.as_ref() {
            if v.trim().is_empty() {
                return Err(format!(
                    "reviews.json：reviews[{}].version 若提供则不能为空字符串",
                    i
                ));
            }
        }
        if !(1..=5).contains(&r.rating) {
            return Err(format!(
                "reviews.json：reviews[{}].rating={} 非法（允许 1~5）",
                i, r.rating
            ));
        }
        let created_at = r.created_at.trim();
        if created_at.is_empty() {
            return Err(format!("reviews.json：reviews[{}].createdAt 不能为空", i));
        }

        if let Some(t) = r.title.as_ref() {
            let s = t.trim();
            if s.len() > 80 {
                return Err(format!("reviews.json：reviews[{}].title 过长（>80）", i));
            }
        }
        if let Some(b) = r.body.as_ref() {
            let s = b.trim();
            if s.len() > 4000 {
                return Err(format!("reviews.json：reviews[{}].body 过长（>4000）", i));
            }
        }
        if let Some(a) = r.author.as_ref() {
            if let Some(gh) = a.github.as_ref() {
                let s = gh.trim();
                if s.len() > 64 {
                    return Err(format!(
                        "reviews.json：reviews[{}].author.github 过长（>64）",
                        i
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty_index() {
        let text = r#"{"schemaVersion":1,"generatedAt":"2026-04-28T00:00:00Z","reviews":[]}"#;
        assert!(validate_plugin_reviews_index_v1(text).is_ok());
    }

    #[test]
    fn rejects_rating_out_of_range() {
        let text = r#"{
          "schemaVersion":1,
          "reviews":[{"id":"r1","pluginId":"x","rating":6,"createdAt":"t"}]
        }"#;
        assert!(validate_plugin_reviews_index_v1(text).is_err());
    }
}
