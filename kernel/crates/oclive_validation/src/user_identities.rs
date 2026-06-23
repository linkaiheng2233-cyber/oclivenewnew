//! `user_identities/index.json` validation.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UserIdentityIndexDisk {
    schema_version: u32,
    default_identity_id: String,
    identities: HashMap<String, UserIdentityIndexEntryDisk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserIdentityIndexEntryDisk {
    display_name: String,
    template_file: String,
    #[serde(default)]
    maps_to_relation_id: Option<String>,
}

/// Validate `user_identities/index.json` and referenced template files.
///
/// # Errors
///
/// Returns human-readable error strings (empty when valid).
#[must_use]
pub fn validate_user_identities_index(
    role_dir: &Path,
    index: &UserIdentityIndexDisk,
) -> Vec<String> {
    let mut errs = Vec::new();

    if index.schema_version != 1 {
        errs.push(format!(
            "user_identities/index.json: schema_version 须为 1（当前 {}）",
            index.schema_version
        ));
    }

    if index.default_identity_id.trim().is_empty() {
        errs.push("user_identities/index.json: default_identity_id 不得为空".into());
    } else if !index.identities.contains_key(&index.default_identity_id) {
        errs.push(format!(
            "user_identities/index.json: default_identity_id「{}」不在 identities 中",
            index.default_identity_id
        ));
    }

    if index.identities.is_empty() {
        errs.push("user_identities/index.json: identities 不得为空".into());
        return errs;
    }

    let base = role_dir.join("user_identities");
    let mut seen_files = HashSet::new();

    for (id, entry) in &index.identities {
        if id.trim().is_empty() {
            errs.push("user_identities/index.json: identity id 不得为空".into());
            continue;
        }
        errs.extend(validate_user_identity_entry(
            &base,
            id,
            entry,
            &mut seen_files,
        ));
    }

    errs
}

fn validate_user_identity_entry(
    base: &Path,
    id: &str,
    entry: &UserIdentityIndexEntryDisk,
    seen_files: &mut HashSet<PathBuf>,
) -> Vec<String> {
    let mut errs = Vec::new();

    if entry.display_name.trim().is_empty() {
        errs.push(format!(
            "user_identities/index.json: identities.{id}.display_name 不得为空"
        ));
    }

    let rel = entry.template_file.trim();
    if rel.is_empty() {
        errs.push(format!(
            "user_identities/index.json: identities.{id}.template_file 不得为空"
        ));
        return errs;
    }
    if rel.contains('\\') || rel.starts_with('/') || rel.contains("..") {
        errs.push(format!(
            "user_identities/index.json: identities.{id}.template_file 须为相对 user_identities/ 的安全路径"
        ));
        return errs;
    }

    let path = base.join(rel);
    if !seen_files.insert(path.clone()) {
        errs.push(format!(
            "user_identities/index.json: identities.{id}.template_file 与其它身份重复"
        ));
    }
    if !path.is_file() {
        errs.push(format!(
            "user_identities/index.json: identities.{id} 模板文件不可读: {}",
            path.display()
        ));
    } else if let Ok(body) = fs::read_to_string(&path) {
        if body.trim().is_empty() {
            errs.push(format!(
                "user_identities/index.json: identities.{id} 模板文件为空: {}",
                path.display()
            ));
        }
    }

    if let Some(ref maps) = entry.maps_to_relation_id {
        if maps.trim().is_empty() {
            errs.push(format!(
                "user_identities/index.json: identities.{id}.maps_to_relation_id 若存在则不得为空"
            ));
        }
    }

    errs
}

/// Warn when `user_identities/` is absent; strict validate when `index.json` exists.
///
/// # Errors
///
/// Returns validation error strings when the catalog directory or entries are invalid.
pub fn validate_user_identities_directory(role_dir: &Path) -> Result<(), Vec<String>> {
    let dir = role_dir.join("user_identities");
    let index_path = dir.join("index.json");
    if !index_path.is_file() {
        if dir.is_dir() {
            return Err(vec![format!(
                "user_identities/ 存在但缺少 index.json: {}",
                index_path.display()
            )]);
        }
        return Ok(());
    }

    let raw = fs::read_to_string(&index_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", index_path.display(), e)])?;
    let index: UserIdentityIndexDisk = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("解析 {} 失败: {}", index_path.display(), e)])?;
    let errs = validate_user_identities_index(role_dir, &index);
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
