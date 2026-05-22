//! `pack validate --profile creator`：仅校验角色包（创作者）子集。

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::blueprint_v2::{validate_meta_personality, PIPELINE_BLUEPRINT_FILENAME};
use crate::role_pack::validate_default_personality_vector;

const CREATOR_META_KEYS: &[&str] = &[
    "id",
    "name",
    "version",
    "author",
    "description",
    "personality",
    "relations",
    "default_relation",
    "scenes",
];

/// 校验角色包目录（创作者 profile）：`meta` 子集 + `prompts/` 存在。
///
/// # Errors
///
/// 缺少蓝图、meta 不合规或 `prompts/` 不存在时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_creator_directory(role_dir: &Path) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        errs.push(format!(
            "creator profile：缺少 {}（请使用 v2/v3 蓝图包）",
            PIPELINE_BLUEPRINT_FILENAME
        ));
        return Err(errs);
    }

    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;

    let root: Value = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("{} JSON 语法错误: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;

    let meta = root.get("meta").ok_or_else(|| {
        vec![format!(
            "creator profile：{} 须包含 meta 对象",
            PIPELINE_BLUEPRINT_FILENAME
        )]
    })?;

    let meta_obj = meta.as_object().ok_or_else(|| {
        vec!["creator profile：meta 须为 JSON 对象".into()]
    })?;

    for key in meta_obj.keys() {
        if key.starts_with('_') {
            continue;
        }
        if !CREATOR_META_KEYS.contains(&key.as_str()) {
            errs.push(format!(
                "creator profile：meta 含非创作者字段「{key}」（系统配置请使用 runtime_config / 蓝图视图，见 ROLE_PACK_BOUNDARY.md）"
            ));
        }
    }

    for required in ["name", "version", "author"] {
        let empty = meta_obj
            .get(required)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().is_empty())
            .unwrap_or(true);
        if empty {
            errs.push(format!("creator profile：meta.{required} 必填且非空"));
        }
    }

    if !meta_obj.contains_key("relations") {
        errs.push("creator profile：meta.relations 必填".into());
    }

    if let Some(p) = meta_obj.get("personality") {
        if let Err(e) = validate_meta_personality(p) {
            errs.push(format!("creator profile：meta.personality {e}"));
        }
    } else if let Some(arr) = meta_obj.get("default_personality") {
        if let Ok(vals) = serde_json::from_value::<Vec<f32>>(arr.clone()) {
            if let Err(e) = validate_default_personality_vector(&vals) {
                errs.push(format!("creator profile：{e}"));
            }
        }
    } else {
        errs.push(
            "creator profile：须提供 meta.personality（或 legacy default_personality 七维数组）"
                .into(),
        );
    }

    let prompts_dir = role_dir.join("prompts");
    if !prompts_dir.is_dir() {
        errs.push(format!(
            "creator profile：缺少 prompts/ 目录（{}）",
            prompts_dir.display()
        ));
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
