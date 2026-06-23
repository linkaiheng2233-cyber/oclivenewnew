//! wasm-bindgen exports (only `wasm32` + `feature = "wasm"`).

use crate::manifest::DiskRoleManifest;
use crate::role_pack::validate_role_pack_loaded;
use crate::validate::{validate_disk_manifest, validate_min_runtime_version};
use serde_json;
use wasm_bindgen::prelude::*;

/// Validate merged manifest JSON, scene id list, and (optionally) minimum host version.
/// Pass `host_runtime_version` as semver aligned with oclivenewnew `Cargo.toml` (e.g. `0.2.0`); empty string skips `min_runtime_version` check.
/// On error, returns Chinese `Err` strings consistent with runtime.
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

/// In-memory role pack validation (`manifest` + optional `settings` + merged scene id list).
/// Empty `settings_json` means no `settings.json`; `merged_scene_ids_json` is a JSON string array.
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
