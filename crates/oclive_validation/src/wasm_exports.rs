//! wasm-bindgen 导出（仅 `wasm32` + `feature = "wasm"`）。

use crate::manifest::DiskRoleManifest;
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
    let disk: DiskRoleManifest = serde_json::from_str(manifest_json).map_err(|e| {
        JsValue::from_str(&format!("manifest.json 解析失败：{}", e))
    })?;
    let scenes: Vec<String> = serde_json::from_str(merged_scene_ids_json).map_err(|e| {
        JsValue::from_str(&format!("merged_scene_ids JSON 解析失败：{}", e))
    })?;
    validate_disk_manifest(&disk, &scenes).map_err(JsValue::from_str)?;
    if !host_runtime_version.trim().is_empty() {
        validate_min_runtime_version(
            disk.min_runtime_version.as_deref(),
            host_runtime_version.trim(),
        )
        .map_err(JsValue::from_str)?;
    }
    Ok(())
}
