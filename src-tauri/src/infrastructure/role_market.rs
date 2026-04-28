use crate::error::{AppError, Result};
use crate::infrastructure::import_role_pack;
use crate::infrastructure::storage::RoleStorage;
use crate::models::dto::ImportProgress;
use oclive_validation::{
    validate_role_market_index_v1, RoleMarketIndexEntryDisk, RoleMarketIndexFileDisk,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const DEFAULT_ROLES_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-roles/main/roles.json";

const MAX_ROLE_PACK_BYTES: u64 = 80 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexFile {
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub roles: Vec<RoleIndexEntry>,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexEntry {
    #[serde(rename = "type")]
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
    pub downloads: Vec<RoleIndexDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleIndexDownload {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub kind: String, // direct|page|pan
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub trust: Option<String>,
}

fn cache_path_for_source(app_data_dir: &Path, url: &str) -> PathBuf {
    let hex = sha256_hex(url.as_bytes());
    app_data_dir
        .join("cache")
        .join("role_market")
        .join(format!("roles-{}.json", &hex[..16]))
}

pub fn sync_role_index_online(
    app_data_dir: &Path,
    source_url: Option<&str>,
) -> Result<RoleIndexFile> {
    let url = source_url.unwrap_or(DEFAULT_ROLES_INDEX_URL);
    let cache = cache_path_for_source(app_data_dir, url);
    sync_role_index_online_at(url, &cache)
}

fn sync_role_index_online_at(url: &str, cache: &Path) -> Result<RoleIndexFile> {
    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("index http client failed: {}", e)))?;
    let resp = cli
        .get(url)
        .send()
        .map_err(|e| AppError::Unknown(format!("sync role index failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::Unknown(format!(
            "sync role index status={} url={}",
            resp.status(),
            url
        )));
    }
    let text = resp
        .text()
        .map_err(|e| AppError::Unknown(format!("read role index response failed: {}", e)))?;
    validate_role_market_index_v1(&text)
        .map_err(|e| AppError::Unknown(format!("roles.json validate failed: {}", e)))?;
    let parsed_disk: RoleMarketIndexFileDisk = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse roles.json failed: {}", e)))?;
    let mut roles = parsed_disk
        .roles
        .into_iter()
        .map(map_role_entry)
        .collect::<Vec<_>>();
    roles.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(parent) = cache.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(cache, &text).ok();
    Ok(RoleIndexFile {
        generated_at: parsed_disk.generated_at,
        roles,
        warning: None,
    })
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

pub fn install_role_pack_from_direct_url<F>(
    storage: &RoleStorage,
    app_data_dir: &Path,
    role_id: &str,
    url: &str,
    expected_sha256: &str,
    overwrite: bool,
    mut on_progress: F,
) -> Result<String>
where
    F: FnMut(ImportProgress),
{
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id 不能为空".into()));
    }
    let u = url.trim();
    if u.is_empty() {
        return Err(AppError::InvalidParameter("download url 不能为空".into()));
    }
    let exp = expected_sha256.trim();
    if exp.len() != 64 {
        return Err(AppError::InvalidParameter(
            "sha256 必须为 64 位十六进制".into(),
        ));
    }

    on_progress(ImportProgress {
        percent: 2,
        message: "正在下载角色包…".into(),
    });

    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Unknown(format!("download http client failed: {}", e)))?;
    let mut resp = cli
        .get(u)
        .send()
        .map_err(|e| AppError::Unknown(format!("download role pack failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::Unknown(format!(
            "download role pack status={} url={}",
            resp.status(),
            u
        )));
    }

    let mut bytes: Vec<u8> = Vec::new();
    let mut buf = [0u8; 32 * 1024];
    let mut total: u64 = 0;
    loop {
        let n = resp
            .read(&mut buf)
            .map_err(|e| AppError::Unknown(format!("read role pack response failed: {}", e)))?;
        if n == 0 {
            break;
        }
        total = total.saturating_add(n as u64);
        if total > MAX_ROLE_PACK_BYTES {
            return Err(AppError::InvalidParameter(format!(
                "[ROLE_PACK_TOO_LARGE] role pack too large (>{} bytes)",
                MAX_ROLE_PACK_BYTES
            )));
        }
        bytes.extend_from_slice(&buf[..n]);
    }

    on_progress(ImportProgress {
        percent: 20,
        message: "正在校验 SHA-256…".into(),
    });
    let got = sha256_hex(&bytes);
    if !eq_hex_sha256(&got, exp) {
        return Err(AppError::InvalidParameter(format!(
            "[ROLE_PACK_SHA256_MISMATCH] sha256 mismatch expected={} got={}",
            exp, got
        )));
    }

    on_progress(ImportProgress {
        percent: 30,
        message: "正在写入临时文件…".into(),
    });
    let tmp_root = app_data_dir.join("tmp");
    let _ = fs::create_dir_all(&tmp_root);
    let td = TempDir::new_in(&tmp_root).map_err(AppError::IoError)?;
    let tmp = td.path().join(format!("{}.ocpak", rid));
    let mut f = fs::File::create(&tmp).map_err(AppError::IoError)?;
    f.write_all(&bytes).map_err(AppError::IoError)?;
    f.flush().ok();

    on_progress(ImportProgress {
        percent: 35,
        message: "正在解压与导入…".into(),
    });
    let path = tmp.clone();
    let installed = import_role_pack(storage, &path, overwrite, |p| {
        // Map import progress to 35~100
        let pct = 35 + ((p.percent.clamp(0, 100) * 65) / 100);
        on_progress(ImportProgress {
            percent: pct,
            message: p.message,
        });
    })?;
    Ok(installed)
}

fn eq_hex_sha256(got: &str, expected: &str) -> bool {
    got.trim().eq_ignore_ascii_case(expected.trim())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
