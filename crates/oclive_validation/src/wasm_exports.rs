//! wasm-bindgen 导出（仅 `wasm32` + `feature = "wasm"`）。

use crate::manifest::DiskRoleManifest;
use crate::role_pack::validate_role_pack_loaded;
use crate::validate::{validate_disk_manifest, validate_min_runtime_version};
use serde_json;
use wasm_bindgen::prelude::*;

/// 校验合并后的 manifest JSON、场景 id 列表，以及（可选）最低宿主版本。
/// `host_runtime_version` 传与 oclivenewnew `Cargo.toml` 对齐的 semver（如 `0.2.0`）；空字符串则跳过 `min_runtime_version` 检查。
/// 错误时返回与运行时一致的中文 `Err` 字符串。
#[wasm_bindgen(js_name = validateManifestWasm)]
pub fn validate_manifest_wasm(
    manifest_json: &str,
    merged_scene_ids_json: &str,
    host_runtime_version: &str,
) -> Result<(), JsValue> {
    let disk: DiskRoleManifest = serde_json::from_str(manifest_json)
        .map_err(|e| JsValue::from_str(&format!("manifest.json 解析失败：{}", e)))?;
    let scenes: Vec<String> = serde_json::from_str(merged_scene_ids_json)
        .map_err(|e| JsValue::from_str(&format!("merged_scene_ids JSON 解析失败：{}", e)))?;
    validate_disk_manifest(&disk, &scenes).map_err(|e| JsValue::from_str(&e))?;
    if !host_runtime_version.trim().is_empty() {
        validate_min_runtime_version(
            disk.min_runtime_version.as_deref(),
            host_runtime_version.trim(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
    }
    Ok(())
}

/// 内存中校验角色包（`manifest` + 可选 `settings` + 已合并场景 id 列表）。
/// `settings_json` 为空字符串表示无 `settings.json`；`merged_scene_ids_json` 为 JSON 字符串数组。
#[wasm_bindgen(js_name = validateRolePackWasm)]
pub fn validate_role_pack_wasm(
    manifest_json: &str,
    settings_json: &str,
    merged_scene_ids_json: &str,
    host_runtime_version: &str,
    settings_schema_supported: u32,
) -> Result<(), JsValue> {
    let merged: Vec<String> = serde_json::from_str(merged_scene_ids_json)
        .map_err(|e| JsValue::from_str(&format!("merged_scene_ids JSON 解析失败：{}", e)))?;
    let settings_opt = if settings_json.trim().is_empty() {
        None
    } else {
        Some(settings_json)
    };
    validate_role_pack_loaded(
        manifest_json,
        settings_opt,
        &merged,
        host_runtime_version.trim(),
        settings_schema_supported,
    )
    .map_err(|errs| JsValue::from_str(&errs.join("\n")))
}
