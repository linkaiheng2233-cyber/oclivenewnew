//! Role pack directory validation (aligned with `RoleStorage::load_role_from_dir` disk phase; does not build full `Role`).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde_json::Value;

use crate::blueprint_v2::{
    validate_blueprint_v2_json_with_context, BlueprintV2ValidateContext,
    PIPELINE_BLUEPRINT_FILENAME,
};
use crate::blueprint_v3::{validate_blueprint_json_by_schema_version, BLUEPRINT_V3_SCHEMA_VERSION};
use crate::creator_profile::validate_role_pack_creator_directory;
use crate::disk_role_settings::DiskRoleSettings;
use crate::json_keys::{validate_manifest_top_level_keys, validate_settings_top_level_keys};
use crate::manifest::DiskRoleManifest;
use crate::validate::{
    validate_disk_manifest, validate_interaction_mode_pack_setting, validate_min_runtime_version,
    validate_settings_schema_version,
};

/// Extended role pack directory validation profile (rules appended after standard disk validation passes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RolePackValidationProfile {
    /// `pipeline.ocblueprint` v2 SSOT (`pack validate` default).
    #[default]
    BlueprintV2,
    /// Legacy: `manifest.json` + `settings.json` (`--profile legacy`).
    Legacy,
    /// Role pack only (creator): `meta` subset + `prompts/`; skips `slot_registry` / `runtime_config`.
    Creator,
    /// Robot / headless minimal soul pack: extra rules after legacy disk validation passes.
    RobotSoul,
}

impl FromStr for RolePackValidationProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "default" | "blueprint-v2" | "blueprint_v2" | "blueprintv2" => Ok(Self::BlueprintV2),
            "legacy" => Ok(Self::Legacy),
            "creator" => Ok(Self::Creator),
            "robot-soul" | "robotsoul" | "robot_soul" => Ok(Self::RobotSoul),
            other => Err(format!(
                "未知 pack validate profile「{other}」（支持 default | legacy | creator | robot-soul）"
            )),
        }
    }
}

fn robot_soul_profile_errors(
    role_dir: Option<&Path>,
    disk: &DiskRoleManifest,
    settings_json: Option<&str>,
) -> Vec<String> {
    let mut errs: Vec<String> = Vec::new();

    if disk
        .min_runtime_version
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_none()
    {
        errs.push(
            "robot-soul：manifest.json 须包含非空 min_runtime_version（semver，与目标 oclive 宿主对齐）"
                .into(),
        );
    }

    let Some(settings_raw) = settings_json else {
        errs.push("robot-soul：须存在 settings.json（含显式 plugin_backends）".into());
        return errs;
    };

    let settings: DiskRoleSettings = match serde_json::from_str(settings_raw) {
        Ok(s) => s,
        Err(e) => {
            errs.push(format!("robot-soul：settings.json 解析失败: {}", e));
            return errs;
        }
    };

    if settings.plugin_backends.is_none() {
        errs.push(
            "robot-soul：settings.json 须显式包含 plugin_backends（memory…agent 六槽；可选 complex_emotion 等扩展键）"
                .into(),
        );
    }

    match settings.interaction_mode.as_deref() {
        None | Some("") => {
            errs.push(
                "robot-soul：settings.json 须包含 interaction_mode（immersive 或 pure_chat）"
                    .into(),
            );
        }
        Some(m) => {
            if let Err(e) = validate_interaction_mode_pack_setting(Some(m)) {
                errs.push(format!("robot-soul：{}", e));
            }
        }
    }

    let core_ok = role_dir
        .map(|dir| dir.join("core_personality.txt"))
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let vec_ok = !disk.default_personality.is_empty()
        && validate_default_personality_vector(&disk.default_personality).is_ok();
    if !core_ok && !vec_ok {
        errs.push(if role_dir.is_some() {
            "robot-soul：须提供非空的 core_personality.txt，或 manifest.default_personality（恰好 7 维、0.0～1.0）"
                .into()
        } else {
            "robot-soul（内存校验）：manifest.default_personality 须为非空且恰好 7 维（0.0～1.0）；目录级校验可改用 core_personality.txt"
                .into()
        });
    }

    errs
}

/// Merges `manifest.scenes` with on-disk `scenes/` subdirectory into scene id list (always includes `default` when empty).
///
/// # Errors
///
/// Returns `Err` when reading `scenes/` fails.
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
        for entry in fs::read_dir(&scenes_dir).map_err(|e| format!("读取 scenes/ 失败: {}", e))?
        {
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

/// `manifest.json` `default_personality`: when non-empty, must be **7** finite values in \[0, 1\] (matches runtime `PersonalityDefaults`).
///
/// # Errors
///
/// Returns `Err` when dimension count or values are invalid.
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
            return Err(format!("manifest：default_personality[{}] 不是有限数字", i));
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

/// Parses manifest + optional settings, top-level keys, seven-dim vector, merges `settings` into `disk` (same order as directory load).
/// Does not run `validate_disk_manifest` / `min_runtime` (caller must supply merged scene ids).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on JSON parse failure, contract violation, or settings validation failure.
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
        if let Err(e) =
            validate_settings_schema_version(settings.schema_version, settings_schema_supported)
        {
            errs.push(e);
        }
        settings.apply_to_manifest(&mut disk);
        if let Err(e) = validate_interaction_mode_pack_setting(settings.interaction_mode.as_deref())
        {
            errs.push(e);
        }
        if let Some(ref pb) = settings.plugin_backends {
            if let Some(e) = crate::agent_backend::validate_implemented_agent_backend(pb) {
                errs.push(e);
            }
        }
    }

    if errs.is_empty() {
        Ok(disk)
    } else {
        Err(errs)
    }
}

/// Run `validate_disk_manifest` and `min_runtime` on the merged scene list.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when any sub-validation fails.
pub fn validate_role_pack_tail(
    disk: &DiskRoleManifest,
    merged_scene_ids: &[String],
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut errs: Vec<String> = Vec::new();
    if let Err(e) = validate_disk_manifest(disk, merged_scene_ids) {
        errs.push(e);
    }
    if let Err(e) = validate_min_runtime_version(disk.min_runtime_version.as_deref(), host_version)
    {
        errs.push(e);
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Same source as directory validation; for in-memory checks in wasm / pack editor (caller supplies merged scene ids, usually including `scenes/` scan).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on parse or tail validation failure.
pub fn validate_role_pack_loaded(
    manifest_json: &str,
    settings_json: Option<&str>,
    merged_scene_ids: &[String],
    host_version: &str,
    settings_schema_supported: u32,
) -> Result<(), Vec<String>> {
    validate_role_pack_loaded_with_profile(
        manifest_json,
        settings_json,
        merged_scene_ids,
        host_version,
        settings_schema_supported,
        RolePackValidationProfile::Legacy,
    )
}

/// Same as [`validate_role_pack_loaded`], plus `robot-soul` extension rules (in-memory validation without `core_personality.txt` accepts only the seven-dimension vector).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when manifest/settings/scene/host version or profile extension rules fail.
pub fn validate_role_pack_loaded_with_profile(
    manifest_json: &str,
    settings_json: Option<&str>,
    merged_scene_ids: &[String],
    host_version: &str,
    settings_schema_supported: u32,
    profile: RolePackValidationProfile,
) -> Result<(), Vec<String>> {
    let disk = validate_role_pack_manifest_settings_core(
        manifest_json,
        settings_json,
        settings_schema_supported,
    )?;
    validate_role_pack_tail(&disk, merged_scene_ids, host_version)?;
    if matches!(profile, RolePackValidationProfile::RobotSoul) {
        let extra = robot_soul_profile_errors(None, &disk, settings_json);
        if !extra.is_empty() {
            return Err(extra);
        }
    }
    Ok(())
}

/// Validate a role pack directory (matches host pre-load disk validation; standard path does not touch DB).
/// When `profile = robot-soul`, reads `core_personality.txt` to check persona-carrier rules.
///
/// `settings_schema_supported`: must match host `CURRENT_SETTINGS_SCHEMA_VERSION` (currently 1).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on missing files, read failure, or validation failure.
pub fn validate_role_pack_directory(
    role_dir: &Path,
    host_version: &str,
    settings_schema_supported: u32,
) -> Result<(), Vec<String>> {
    validate_role_pack_directory_with_profile(
        role_dir,
        host_version,
        settings_schema_supported,
        RolePackValidationProfile::BlueprintV2,
    )
}

/// Same as [`validate_role_pack_directory`], plus `--profile robot-soul` extension rules.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on missing files, read failure, or validation failure.
pub fn validate_role_pack_directory_with_profile(
    role_dir: &Path,
    host_version: &str,
    settings_schema_supported: u32,
    profile: RolePackValidationProfile,
) -> Result<(), Vec<String>> {
    if matches!(profile, RolePackValidationProfile::Creator) {
        return validate_role_pack_creator_directory(role_dir);
    }

    if matches!(profile, RolePackValidationProfile::BlueprintV2) {
        return validate_role_pack_blueprint_directory(role_dir, host_version);
    }

    // Legacy + RobotSoul: manifest/settings path
    let mut errs: Vec<String> = Vec::new();
    let manifest_path = role_dir.join("manifest.json");
    if !manifest_path.is_file() {
        errs.push(format!("缺少 manifest.json：{}", manifest_path.display()));
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

    validate_role_pack_tail(&disk, &merged_scenes, host_version)?;
    if matches!(profile, RolePackValidationProfile::RobotSoul) {
        let extra = robot_soul_profile_errors(Some(role_dir), &disk, settings_raw.as_deref());
        if !extra.is_empty() {
            return Err(extra);
        }
    }
    Ok(())
}

fn validate_role_pack_blueprint_directory(
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    let manifest_path = role_dir.join("manifest.json");
    if manifest_path.is_file() {
        errs.push(format!(
            "v2/v3 角色包不得包含 manifest.json（已废弃）：{}",
            manifest_path.display()
        ));
    }
    let settings_path = role_dir.join("settings.json");
    if settings_path.is_file() {
        errs.push(format!(
            "v2/v3 角色包不得包含 settings.json（已废弃）：{}",
            settings_path.display()
        ));
    }
    if !errs.is_empty() {
        return Err(errs);
    }

    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return Err(vec![format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        )]);
    }

    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", blueprint_path.display(), e)])?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str());

    let warnings = validate_blueprint_json_by_schema_version(&raw, folder_name)?;

    let version = serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| v.get("schema_version").and_then(|n| n.as_u64()))
        .unwrap_or(0) as u32;

    if version == BLUEPRINT_V3_SCHEMA_VERSION {
        if !warnings.is_empty() {
            print_pack_warnings(&warnings);
        }
        validate_role_pack_optional_extensions(role_dir)?;
        return Ok(());
    }

    validate_blueprint_v2_json_with_context(
        &raw,
        BlueprintV2ValidateContext {
            folder_name,
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )?;
    if !warnings.is_empty() {
        print_pack_warnings(&warnings);
    }
    validate_role_pack_optional_extensions(role_dir)?;
    Ok(())
}

fn validate_role_pack_optional_extensions(role_dir: &Path) -> Result<(), Vec<String>> {
    let mut warns = Vec::new();
    let identities_dir = role_dir.join("user_identities");
    if !identities_dir.is_dir() {
        warns.push("可选目录 user_identities/ 未配置；将回退 meta.relations.prompt_hint".into());
    } else {
        crate::user_identities::validate_user_identities_directory(role_dir)?;
    }
    crate::reply_post_processor::validate_reply_post_processor_config_file(
        &role_dir.join("config.json"),
    )?;
    crate::chat_storage::validate_chat_storage_config_file(&role_dir.join("config.json"))?;
    crate::meta_action_templates::validate_meta_action_templates_config_file(
        &role_dir.join("config.json"),
    )?;
    crate::penetration_templates::validate_penetration_templates_config_file(
        &role_dir.join("config.json"),
    )?;
    if !warns.is_empty() {
        print_pack_warnings(&warns);
    }
    Ok(())
}

fn print_pack_warnings(warnings: &[String]) {
    eprintln!("pack validate 警告:");
    for w in warnings {
        eprintln!("  - {w}");
    }
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

        validate_role_pack_directory_with_profile(
            &role,
            "999.0.0",
            1,
            RolePackValidationProfile::Legacy,
        )
        .unwrap();

        let merged = merge_role_pack_scene_ids(&role, &["default".to_string()]).unwrap();
        let settings_raw = fs::read_to_string(role.join("settings.json")).unwrap();
        validate_role_pack_loaded_with_profile(
            &fs::read_to_string(role.join("manifest.json")).unwrap(),
            Some(&settings_raw),
            &merged,
            "999.0.0",
            1,
            RolePackValidationProfile::Legacy,
        )
        .unwrap();
    }

    #[test]
    fn creator_profile_validates_meta_and_prompts_only() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("hero");
        fs::create_dir_all(role.join("prompts")).unwrap();
        let bp = r#"{
          "schema_version": 2,
          "meta": {
            "id": "hero",
            "name": "Hero",
            "version": "1",
            "author": "a",
            "description": "d",
            "personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
            "relations": { "f": { "initial_favorability": 50, "favor_multiplier": 1.0 } },
            "default_relation": "f",
            "interaction_mode": "immersive"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 1 }
          }
        }"#;
        fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), bp).unwrap();
        let errs = validate_role_pack_creator_directory(&role).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("interaction_mode")));
        assert!(errs.iter().any(|e| e.contains("非创作者字段")));
    }

    #[test]
    fn robot_soul_rejects_without_min_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("rs");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        let manifest = serde_json::json!({
            "id": "rs.demo",
            "name": "R",
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
        fs::write(role.join("manifest.json"), manifest.to_string()).unwrap();
        let settings = serde_json::json!({
            "schema_version": 1,
            "interaction_mode": "immersive",
            "plugin_backends": {
                "memory": "builtin",
                "emotion": "builtin",
                "event": "builtin",
                "prompt": "builtin",
                "llm": "ollama",
                "agent": "builtin"
            }
        });
        fs::write(role.join("settings.json"), settings.to_string()).unwrap();
        let r = validate_role_pack_directory_with_profile(
            &role,
            "999.0.0",
            1,
            RolePackValidationProfile::RobotSoul,
        );
        assert!(r.is_err());
        let errs = r.unwrap_err().join("\n");
        assert!(errs.contains("min_runtime_version"), "{}", errs);
    }

    #[test]
    fn robot_soul_accepts_minimal_vector_pack() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("rs2");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        let manifest = serde_json::json!({
            "id": "rs2.demo",
            "name": "R2",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "min_runtime_version": "0.2.0",
            "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "scenes": ["default"],
            "user_relations": {
                "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend"
        });
        fs::write(role.join("manifest.json"), manifest.to_string()).unwrap();
        let settings = serde_json::json!({
            "schema_version": 1,
            "interaction_mode": "pure_chat",
            "plugin_backends": {
                "memory": "builtin",
                "emotion": "builtin",
                "event": "builtin",
                "prompt": "builtin",
                "llm": "ollama",
                "agent": "builtin"
            }
        });
        fs::write(role.join("settings.json"), settings.to_string()).unwrap();
        validate_role_pack_directory_with_profile(
            &role,
            "999.0.0",
            1,
            RolePackValidationProfile::RobotSoul,
        )
        .unwrap();
    }

    #[test]
    fn robot_soul_accepts_core_file_without_vector() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path().join("rs3");
        fs::create_dir_all(role.join("scenes").join("default")).unwrap();
        let manifest = serde_json::json!({
            "id": "rs3.demo",
            "name": "R3",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "min_runtime_version": "0.2.0",
            "scenes": ["default"],
            "user_relations": {
                "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend"
        });
        fs::write(role.join("manifest.json"), manifest.to_string()).unwrap();
        let settings = serde_json::json!({
            "schema_version": 1,
            "interaction_mode": "immersive",
            "plugin_backends": {
                "memory": "builtin",
                "emotion": "builtin",
                "event": "builtin",
                "prompt": "builtin",
                "llm": "remote",
                "agent": "builtin"
            }
        });
        fs::write(role.join("settings.json"), settings.to_string()).unwrap();
        fs::write(
            role.join("core_personality.txt"),
            "核心人设：简短、稳定、可部署。\n",
        )
        .unwrap();
        let scene = serde_json::json!({
            "name": "Default",
            "time_windows": [],
            "keywords": [],
            "events": []
        });
        fs::write(
            role.join("scenes").join("default").join("scene.json"),
            scene.to_string(),
        )
        .unwrap();
        validate_role_pack_directory_with_profile(
            &role,
            "999.0.0",
            1,
            RolePackValidationProfile::RobotSoul,
        )
        .unwrap();
    }
}
