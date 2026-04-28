//! Role market index (`roles.json`) validation (v1).
//!
//! This is intentionally **no-code**: it validates the JSON contract and basic safety constraints.

use serde::{Deserialize, Serialize};

fn default_entry_type() -> String {
    "role".to_string()
}

fn default_download_kind() -> String {
    "direct".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleMarketIndexFileDisk {
    #[serde(default, alias = "generated_at")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub roles: Vec<RoleMarketIndexEntryDisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleMarketIndexEntryDisk {
    #[serde(rename = "type", default = "default_entry_type")]
    pub entry_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub downloads: Vec<RoleMarketDownloadDisk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleMarketDownloadDisk {
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_download_kind")]
    pub kind: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub trust: Option<String>,
}

pub fn validate_role_market_index_v1(text: &str) -> Result<(), String> {
    let file: RoleMarketIndexFileDisk =
        serde_json::from_str(text).map_err(|e| format!("roles.json：解析失败：{}", e))?;
    for (i, e) in file.roles.iter().enumerate() {
        validate_entry(i, e)?;
    }
    Ok(())
}

fn validate_entry(i: usize, e: &RoleMarketIndexEntryDisk) -> Result<(), String> {
    let ty = e.entry_type.trim();
    if ty.is_empty() {
        return Err(format!("roles.json：roles[{}].type 不能为空", i));
    }
    if ty != "role" {
        return Err(format!(
            "roles.json：roles[{}].type={} 不受支持（允许：role）",
            i, ty
        ));
    }
    if e.id.trim().is_empty() {
        return Err(format!("roles.json：roles[{}].id 不能为空", i));
    }
    if e.name.trim().is_empty() {
        return Err(format!("roles.json：roles[{}].name 不能为空", i));
    }
    if e.version.trim().is_empty() {
        return Err(format!("roles.json：roles[{}].version 不能为空", i));
    }
    if e.downloads.is_empty() {
        return Err(format!(
            "roles.json：roles[{}].downloads 至少需要 1 个镜像",
            i
        ));
    }
    for (j, d) in e.downloads.iter().enumerate() {
        validate_download(i, j, d)?;
    }
    Ok(())
}

fn validate_download(i: usize, j: usize, d: &RoleMarketDownloadDisk) -> Result<(), String> {
    if d.url.trim().is_empty() {
        return Err(format!(
            "roles.json：roles[{}].downloads[{}].url 不能为空",
            i, j
        ));
    }
    let sha = d.sha256.trim();
    if sha.is_empty() {
        return Err(format!(
            "roles.json：roles[{}].downloads[{}].sha256 不能为空",
            i, j
        ));
    }
    if !is_sha256_hex(sha) {
        return Err(format!(
            "roles.json：roles[{}].downloads[{}].sha256 必须是 64 位十六进制",
            i, j
        ));
    }
    let kind = d.kind.trim();
    match kind {
        "direct" | "page" | "pan" => {}
        other => {
            return Err(format!(
                "roles.json：roles[{}].downloads[{}].kind={} 不受支持（允许：direct|page|pan）",
                i, j, other
            ));
        }
    }
    if let Some(t) = d.trust.as_ref() {
        let v = t.trim();
        if !v.is_empty() {
            match v {
                "official" | "verified" | "community" | "unknown" => {}
                other => {
                    return Err(format!(
                        "roles.json：roles[{}].downloads[{}].trust={} 不受支持（允许：official|verified|community|unknown）",
                        i, j, other
                    ));
                }
            }
        }
    }
    Ok(())
}

fn is_sha256_hex(s: &str) -> bool {
    if s.len() != 64 {
        return false;
    }
    s.as_bytes().iter().all(|b| match b {
        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => true,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_basic_role_entry() {
        let text = r#"{
          "roles":[
            {"type":"role","id":"mumu","name":"M","version":"1.0.0","downloads":[{"url":"https://x","sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}]}
          ]
        }"#;
        assert!(validate_role_market_index_v1(text).is_ok());
    }

    #[test]
    fn rejects_missing_downloads() {
        let text = r#"{"roles":[{"type":"role","id":"mumu","name":"M","version":"1.0.0","downloads":[]}]} "#;
        assert!(validate_role_market_index_v1(text).is_err());
    }

    #[test]
    fn rejects_bad_sha() {
        let text = r#"{
          "roles":[
            {"type":"role","id":"mumu","name":"M","version":"1.0.0","downloads":[{"url":"https://x","sha256":"abc"}]}
          ]
        }"#;
        assert!(validate_role_market_index_v1(text).is_err());
    }
}

