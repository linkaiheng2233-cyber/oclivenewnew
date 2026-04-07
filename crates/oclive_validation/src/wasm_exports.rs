//! wasm-bindgen 导出（仅 `wasm32` + `feature = "wasm"`）。

use crate::manifest::DiskRoleManifest;
use crate::validate::validate_disk_manifest;
use serde_json;
use wasm_bindgen::prelude::*;

/// 校验 manifest JSON 与合并后的场景 id 列表（JSON 数组字符串）。
/// 错误时返回与运行时一致的中文 `Err` 字符串。
#[wasm_bindgen(js_name = validateManifestWasm)]
pub fn validate_manifest_wasm(manifest_json: &str, merged_scene_ids_json: &str) -> Result<(), JsValue> {
    let disk: DiskRoleManifest = serde_json::from_str(manifest_json).map_err(|e| {
        JsValue::from_str(&format!("manifest.json 解析失败：{}", e))
    })?;
    let scenes: Vec<String> = serde_json::from_str(merged_scene_ids_json).map_err(|e| {
        JsValue::from_str(&format!("merged_scene_ids JSON 解析失败：{}", e))
    })?;
    validate_disk_manifest(&disk, &scenes).map_err(JsValue::from_str)
}
