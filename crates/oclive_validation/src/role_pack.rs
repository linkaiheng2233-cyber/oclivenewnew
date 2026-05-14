//! 角色包目录级校验（与 `RoleStorage::load_role_from_dir` 磁盘阶段对齐，不构建完整 `Role`）。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::disk_role_settings::DiskRoleSettings;
use crate::json_keys::{validate_manifest_top_level_keys, validate_settings_top_level_keys};
use crate::manifest::DiskRoleManifest;
use crate::validate::{
    validate_disk_manifest, validate_interaction_mode_pack_setting, validate_min_runtime_version,
    validate_settings_schema_version,
};
/// 合并 `manifest.scenes` 与磁盘 `scenes/` 子目录，得到场景 id 列表（至少含 `default`）。
///
/// # Errors
///
/// 读取 `scenes/` 目录失败时返回 `Err`。
pub fn merge_role_pack_scene_ids(
    role_dir: &Path,
    manifest_scenes: &[String],
) -> Result<Vec<String>, String> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for s in manifest_scenes {
        if !s.trim().is_empty() {
            ids.insert(s.clone());
        }
    }

    let scenes_dir = role_dir.join("scenes");
    if scenes_dir.is_dir() {
        for entry in fs::read_dir(&scenes_dir).map_err(|e| format!("读取 scenes/ 失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取 scenes/ 项失败: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') {
                    ids.insert(name);
                }
            }
        }
    }

    if ids.is_empty() {
        ids.insert("default".to_string());
    }

    Ok(ids.into_iter().collect())
}

/// `manifest.json` 中 `default_personality`：非空时须 **7** 个有限数，且每维在 \[0, 1\]（与运行时 `PersonalityDefaults` 一致）。
///
/// # Errors
///
/// 维度数量或取值不合法时返回 `Err`。
pub fn validate_default_personality_vector(values: &[f32]) -> Result<(), String> {
    if values.is_empty() {
        return Ok(());
    }
    if values.len() != 7 {
        return Err(format!(
            "manifest：default_personality 须为 7 个浮点数（stubbornness…warmth），当前 {} 个",
            values.len()
        ));
    }
    for (i, x) in values.iter().enumerate() {
        if !x.is_finite() {
            return Err(format!(
                "manifest：default_personality[{}] 不是有限数字",
                i
            ));
        }
        if *x < 0.0 || *x > 1.0 {
            return Err(format!(
                "manifest：default_personality[{}] 须在 0.0～1.0 之间（当前为 {}）",
                i, x
            ));
        }
    }
    Ok(())
}

/// manifest + 可选 settings 解析、顶层键、七维、`settings` 合并进 `disk`（与目录加载顺序一致）。
/// 不含 `validate_disk_manifest` / `min_runtime`（需调用方提供合并后的场景 id）。
///
/// # Errors
///
/// JSON 解析失败、契约不符或 settings 校验失败时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_manifest_settings_core(
    manifest_json: &str,
    settings_json: Option<&str>,
    settings_schema_supported: u32,
) -> Result<DiskRoleManifest, Vec<String>> {
    let mut errs: Vec<String> = Vec::new();

    let manifest_value: Value = match serde_json::from_str(manifest_json) {
        Ok(v) => v,
        Err(e) => {
            errs.push(format!("manifest.json JSON 语法错误: {}", e));
            return Err(errs);
        }
    };

    if let Value::Object(ref map) = manifest_value {
        if let Err(e) = validate_manifest_top_level_keys(map) {
            errs.push(e);
        }
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    let mut disk: DiskRoleManifest = match serde_json::from_str(manifest_json) {
        Ok(d) => d,
        Err(e) => {
            errs.push(format!("manifest.json 结构不符合契约: {}", e));
            return Err(errs);
        }
    };

    if let Err(e) = validate_default_personality_vector(&disk.default_personality) {
        errs.push(e);
    }

    if let Some(sr) = settings_json {
        let settings_value: Value = match serde_json::from_str(sr) {
            Ok(v) => v,
            Err(e) => {
                errs.push(format!("settings.json JSON 语法错误: {}", e));
                return Err(errs);
            }
        };
        if let Value::Object(ref map) = settings_value {
            if let Err(e) = validate_settings_top_level_keys(map) {
                errs.push(e);
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }

        let settings: DiskRoleSettings = match serde_json::from_str(sr) {
            Ok(s) => s,
            Err(e) => {
                errs.push(format!("settings.json 结构不符合契约: {}", e));
                return Err(errs);
            }
        };
        if let Err(e) = validate_settings_schema_version(
            settings.schema_version,
            settings_schema_supported,
        ) {
            errs.push(e);
        }
        settings.apply_to_manifest(&mut disk);
        if let Err(e) = validate_interaction_mode_pack_setting(settings.interaction_mode.as_deref())
        {
            errs.push(e);
        }
    }

    if errs.is_empty() {
        Ok(disk)
    } else {
        Err(errs)
    }
}

/// 在已合并场景列表上跑 `validate_disk_manifest` 与 `min_runtime`。
///
/// # Errors
///
/// 任一子校验失败时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_tail(
    disk: &DiskRoleManifest,
    merged_scene_ids: &[String],
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut errs: Vec<String> = Vec::new();
    if let Err(e) = validate_disk_manifest(disk, merged_scene_ids) {
        errs.push(e);
    }
    if let Err(e) = validate_min_runtime_version(disk.min_runtime_version.as_deref(), host_version) {
        errs.push(e);
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// 与目录校验同源，供 wasm / 编写器在内存中校验（调用方提供合并后的场景 id，通常含 `scenes/` 扫描结果）。
///
/// # Errors
///
/// 解析或尾部校验失败时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_loaded(
    manifest_json: &str,
    settings_json: Option<&str>,
    merged_scene_ids: &[String],
    host_version: &str,
    settings_schema_supported: u32,
) -> Result<(), Vec<String>> {
    let disk = validate_role_pack_manifest_settings_core(
        manifest_json,
        settings_json,
        settings_schema_supported,
    )?;
    validate_role_pack_tail(&disk, merged_scene_ids, host_version)
}

/// 校验角色包目录（与宿主加载前磁盘校验一致；不读 `core_personality.txt`、不跑 DB）。
///
/// `settings_schema_supported`：与宿主 `CURRENT_SETTINGS_SCHEMA_VERSION` 一致（当前为 1）。
///
/// # Errors
///
/// 缺少文件、读盘失败或校验未通过时返回 `Err(Vec<String>)`。
pub fn validate_role_pack_directory(
    role_dir: &Path,
    host_version: &str,
    settings_schema_supported: u32,
) -> Result<(), Vec<String>> {
    let mut errs: Vec<String> = Vec::new();
    let manifest_path = role_dir.join("manifest.json");
    if !manifest_path.is_file() {
        errs.push(format!(
            "缺少 manifest.json：{}",
            manifest_path.display()
        ));
        return Err(errs);
    }

    let manifest_raw = match fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            errs.push(format!("读取 manifest.json 失败: {}", e));
            return Err(errs);
        }
    };

    let settings_path = role_dir.join("settings.json");
    let settings_raw = if settings_path.is_file() {
        match fs::read_to_string(&settings_path) {
            Ok(s) => Some(s),
            Err(e) => {
                errs.push(format!("读取 settings.json 失败: {}", e));
                return Err(errs);
            }
        }
    } else {
        None
    };

    let disk = validate_role_pack_manifest_settings_core(
        &manifest_raw,
        settings_raw.as_deref(),
        settings_schema_supported,
    )?;

    let merged_scenes = match merge_role_pack_scene_ids(role_dir, &disk.scenes) {
        Ok(s) => s,
        Err(e) => {
            errs.push(e);
            return Err(errs);
        }
    };

    validate_role_pack_tail(&disk, &merged_scenes, host_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn validate_minimal_role_pack_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("demo");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();

        let manifest = serde_json::json!({
            "id": "demo.pack",
            "name": "Demo",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "scenes": ["default"],
            "user_relations": {
                "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend"
        });
        let mut f = fs::File::create(role.join("manifest.json")).unwrap();
        f.write_all(manifest.to_string().as_bytes()).unwrap();

        let settings = serde_json::json!({
            "schema_version": 1,
            "plugin_backends": {
                "memory": "builtin",
                "emotion": "builtin",
                "event": "builtin",
                "prompt": "builtin",
                "llm": "ollama",
                "agent": "builtin"
            }
        });
        let mut f2 = fs::File::create(role.join("settings.json")).unwrap();
        f2.write_all(settings.to_string().as_bytes()).unwrap();

        validate_role_pack_directory(&role, "999.0.0", 1).unwrap();

        let merged = merge_role_pack_scene_ids(&role, &["default".to_string()]).unwrap();
        let settings_raw = fs::read_to_string(role.join("settings.json")).unwrap();
        validate_role_pack_loaded(
            &fs::read_to_string(role.join("manifest.json")).unwrap(),
            Some(&settings_raw),
            &merged,
            "999.0.0",
            1,
        )
        .unwrap();
    }
}
