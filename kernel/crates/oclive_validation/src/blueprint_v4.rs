//! `pipeline.ocblueprint` schema_version 4.
//!
//! v4 is the Stable successor to v2: it activates `runtime_config` and adds a
//! deliberately small extension-declaration envelope. It does not inherit the
//! frozen v3 `pipeline` / `zone` dual-core experiment.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blueprint_includes::{
    merge_blueprint_includes_strict, validate_blueprint_relative_path,
    validate_blueprint_relative_path_syntax, validate_existing_blueprint_file,
    BlueprintIncludeEntry,
};
use crate::blueprint_v2::{
    meta_to_disk_manifest, validate_blueprint_meta_core, validate_slot_registry_contract,
    BlueprintMeta, SlotGroupEntry, SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME,
};
use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::DiskRoleManifest;
use crate::role_pack::merge_role_pack_scene_ids;
use crate::runtime_config::{validate_runtime_config, RuntimeConfig};
use crate::validate::{
    validate_disk_manifest, validate_knowledge_manifest_disk, validate_min_runtime_version,
};

pub const BLUEPRINT_V4_SCHEMA_VERSION: u32 = 4;

/// One namespaced extension instance declared by a v4 role pack.
///
/// This is only a portable declaration. Capability resolution and provider
/// execution remain host responsibilities and are intentionally not encoded in
/// the role-pack schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BlueprintExtensionDecl {
    /// Namespaced capability identifier, for example `com.example.live2d`.
    pub capability: String,
    /// Optional namespaced provider preference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Whether activation must fail when the host cannot resolve the capability.
    #[serde(default)]
    pub required: bool,
    /// Schema version of the referenced extension-owned JSON payload.
    pub config_schema_version: u32,
    /// Role-root-relative JSON path owned by this extension instance.
    pub config_ref: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BlueprintV4File {
    schema_version: u32,
    meta: BlueprintMeta,
    slot_registry: BTreeMap<String, SlotRegistryEntry>,
    #[serde(default)]
    groups: BTreeMap<String, SlotGroupEntry>,
    #[serde(default)]
    runtime_config: Option<RuntimeConfig>,
    #[serde(default)]
    #[allow(dead_code)]
    includes: Vec<BlueprintIncludeEntry>,
    #[serde(default)]
    #[allow(dead_code)]
    expert_overlay: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    extensions: BTreeMap<String, BlueprintExtensionDecl>,
}

/// Validated v4 blueprint data consumed by the host.
#[derive(Debug, Clone)]
pub struct BlueprintV4LoadResult {
    pub disk: DiskRoleManifest,
    pub slot_registry: BTreeMap<String, SlotRegistryEntry>,
    pub groups: BTreeMap<String, SlotGroupEntry>,
    pub runtime_config: Option<RuntimeConfig>,
    pub extensions: BTreeMap<String, BlueprintExtensionDecl>,
    pub interaction_mode: Option<String>,
    pub featured: bool,
    pub deep_capsule_enabled: bool,
    pub preset_order: u32,
    pub remote_presence: Option<RemotePresenceConfig>,
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

/// Validate v4 blueprint JSON without touching referenced files.
///
/// Directory validation is required before activation because it additionally
/// checks every extension payload for containment, existence, and valid JSON.
///
/// # Errors
///
/// Returns all structural contract failures.
pub fn validate_blueprint_v4_json(raw: &str, folder_name: Option<&str>) -> Result<(), Vec<String>> {
    let root: Value = serde_json::from_str(raw)
        .map_err(|e| vec![format!("pipeline.ocblueprint v4 JSON 语法错误: {e}")])?;
    let object = root
        .as_object()
        .ok_or_else(|| vec!["pipeline.ocblueprint v4 根节点须为 JSON 对象".into()])?;

    let mut shape_errors = Vec::new();
    for field in [
        "meta",
        "slot_registry",
        "groups",
        "runtime_config",
        "expert_overlay",
        "extensions",
    ] {
        if object.get(field).is_some_and(|value| !value.is_object()) {
            shape_errors.push(format!(
                "pipeline.ocblueprint v4.{field} 若存在则须为 JSON 对象"
            ));
        }
    }
    if !shape_errors.is_empty() {
        return Err(shape_errors);
    }

    let blueprint: BlueprintV4File = serde_json::from_value(root)
        .map_err(|e| vec![format!("pipeline.ocblueprint v4 结构不符合契约: {e}")])?;
    validate_blueprint_v4_parsed(&blueprint, folder_name)
}

fn validate_blueprint_v4_parsed(
    blueprint: &BlueprintV4File,
    folder_name: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if blueprint.schema_version != BLUEPRINT_V4_SCHEMA_VERSION {
        errors.push(format!(
            "schema_version 须为 {}（当前 {}）",
            BLUEPRINT_V4_SCHEMA_VERSION, blueprint.schema_version
        ));
    }

    validate_blueprint_meta_core(&blueprint.meta, folder_name, &mut errors);
    if let Some(ref knowledge) = blueprint.meta.knowledge {
        if let Err(error) = validate_knowledge_manifest_disk(knowledge) {
            errors.push(error);
        }
    }

    if let Some(ref runtime_config) = blueprint.runtime_config {
        if let Err(mut runtime_errors) = validate_runtime_config(runtime_config) {
            errors.append(&mut runtime_errors);
        }
        if runtime_config.dual_core.is_some() {
            errors.push(
                "runtime_config.dual_core 仅属于冻结的 schema_version 3；稳定 v4 不接受该字段"
                    .into(),
            );
        }
    }

    errors.extend(validate_slot_registry_contract(
        &blueprint.slot_registry,
        &blueprint.groups,
        false,
    ));
    errors.extend(validate_extension_declarations(&blueprint.extensions));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_extension_declarations(
    extensions: &BTreeMap<String, BlueprintExtensionDecl>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for (instance_id, extension) in extensions {
        let label = format!("extensions[{instance_id}]");
        if let Err(error) = validate_namespaced_id(instance_id) {
            errors.push(format!("{label} 实例 id 非法：{error}"));
        }
        if let Err(error) = validate_namespaced_id(&extension.capability) {
            errors.push(format!("{label}.capability 非法：{error}"));
        }
        if let Some(ref provider) = extension.provider {
            if let Err(error) = validate_namespaced_id(provider) {
                errors.push(format!("{label}.provider 非法：{error}"));
            }
        }
        if extension.config_schema_version == 0 {
            errors.push(format!("{label}.config_schema_version 须大于 0"));
        }
        if let Err(error) = validate_blueprint_relative_path_syntax(&extension.config_ref) {
            errors.push(format!("{label}.config_ref 非法：{error}"));
            continue;
        }
        if !extension.config_ref.ends_with(".json") {
            errors.push(format!("{label}.config_ref 须指向 .json 文件"));
        }
        let expected_prefix = format!("blueprint/extensions/{instance_id}/");
        if !extension.config_ref.starts_with(&expected_prefix) {
            errors.push(format!(
                "{label}.config_ref 须位于该实例目录 {expected_prefix}"
            ));
        }
    }
    errors
}

fn validate_namespaced_id(value: &str) -> Result<(), String> {
    if value != value.trim() {
        return Err("不得含首尾空白，须使用规范命名空间原文".into());
    }
    if value.is_empty() || value.len() > 160 {
        return Err("须为 1–160 字符的命名空间标识".into());
    }
    let segments: Vec<&str> = value.split('.').collect();
    if segments.len() < 2
        || segments.iter().any(|segment| {
            segment.is_empty()
                || !segment.chars().all(|character| {
                    character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || matches!(character, '_' | '-')
                })
        })
    {
        return Err(
            "须使用至少两段的小写命名空间（仅 a-z、0-9、_、-，例如 com.example.live2d）".into(),
        );
    }
    Ok(())
}

fn validate_extension_payloads(
    role_dir: &Path,
    extensions: &BTreeMap<String, BlueprintExtensionDecl>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (instance_id, extension) in extensions {
        let label = format!("extensions[{instance_id}].config_ref");
        if let Err(error) = validate_blueprint_relative_path(role_dir, &extension.config_ref) {
            errors.push(format!("{label} 非法：{error}"));
            continue;
        }
        let payload_path = role_dir.join(&extension.config_ref);
        if !payload_path.is_file() {
            errors.push(format!("{label} 文件不存在：{}", payload_path.display()));
            continue;
        }
        if let Err(error) = validate_existing_blueprint_file(role_dir, &payload_path) {
            errors.push(format!("{label}：{error}"));
            continue;
        }
        match fs::read_to_string(&payload_path) {
            Ok(raw) => {
                if let Err(error) = serde_json::from_str::<Value>(&raw) {
                    errors.push(format!(
                        "{label} JSON 解析失败 {}: {error}",
                        payload_path.display()
                    ));
                }
            }
            Err(error) => errors.push(format!(
                "{label} 读取失败 {}: {error}",
                payload_path.display()
            )),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(crate) fn validate_blueprint_v4_extension_payloads_for_raw(
    role_dir: &Path,
    raw: &str,
) -> Result<(), Vec<String>> {
    let blueprint: BlueprintV4File = serde_json::from_str(raw)
        .map_err(|error| vec![format!("pipeline.ocblueprint v4 结构不符合契约: {error}")])?;
    validate_extension_payloads(role_dir, &blueprint.extensions)
}

fn reject_legacy_role_files(role_dir: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for legacy_name in ["manifest.json", "settings.json"] {
        let legacy_path = role_dir.join(legacy_name);
        if legacy_path.is_file() {
            errors.push(format!(
                "v4 角色包不得包含 {legacy_name}（已废弃）：{}",
                legacy_path.display()
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn read_resolved_v4_blueprint(role_dir: &Path) -> Result<BlueprintV4File, Vec<String>> {
    reject_legacy_role_files(role_dir)?;
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return Err(vec![format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        )]);
    }
    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|error| vec![format!("读取 {} 失败: {error}", blueprint_path.display())])?;
    let folder_name = role_dir.file_name().and_then(|value| value.to_str());
    validate_blueprint_v4_json(&raw, folder_name)?;
    let resolved = merge_blueprint_includes_strict(role_dir, &raw)?;
    validate_blueprint_v4_json(&resolved, folder_name)?;
    let blueprint: BlueprintV4File = serde_json::from_str(&resolved)
        .map_err(|error| vec![format!("pipeline.ocblueprint v4 结构不符合契约: {error}")])?;
    validate_extension_payloads(role_dir, &blueprint.extensions)?;
    Ok(blueprint)
}

fn blueprint_v4_file_to_load_result(blueprint: &BlueprintV4File) -> BlueprintV4LoadResult {
    let mut disk = meta_to_disk_manifest(&blueprint.meta);
    let interaction_mode = blueprint
        .runtime_config
        .as_ref()
        .and_then(|config| config.interaction_mode.clone())
        .or_else(|| blueprint.meta.interaction_mode.clone());
    let remote_presence = blueprint
        .runtime_config
        .as_ref()
        .and_then(|config| config.remote_presence.clone())
        .or_else(|| blueprint.meta.remote_presence.clone());
    let autonomous_scene = blueprint
        .runtime_config
        .as_ref()
        .and_then(|config| config.autonomous_scene.clone())
        .or_else(|| blueprint.meta.autonomous_scene.clone());
    let reply_quality_anchor = blueprint
        .runtime_config
        .as_ref()
        .and_then(|config| config.reply_quality_anchor.clone())
        .or_else(|| blueprint.meta.reply_quality_anchor.clone());

    if let Some(ref runtime_config) = blueprint.runtime_config {
        crate::blueprint_v3::apply_runtime_config_to_disk(&mut disk, runtime_config);
    }

    BlueprintV4LoadResult {
        disk,
        slot_registry: blueprint.slot_registry.clone(),
        groups: blueprint.groups.clone(),
        runtime_config: blueprint.runtime_config.clone(),
        extensions: blueprint.extensions.clone(),
        interaction_mode,
        featured: blueprint.meta.featured,
        deep_capsule_enabled: blueprint.meta.deep_capsule_enabled,
        preset_order: blueprint.meta.preset_order,
        remote_presence,
        autonomous_scene,
        reply_quality_anchor,
    }
}

/// Validate an on-disk v4 role pack.
///
/// # Errors
///
/// Returns contract, path-safety, payload, scene, or runtime-version failures.
pub fn validate_role_pack_blueprint_v4_directory(
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let blueprint = read_resolved_v4_blueprint(role_dir)?;
    let mut disk = meta_to_disk_manifest(&blueprint.meta);
    if let Some(ref runtime_config) = blueprint.runtime_config {
        crate::blueprint_v3::apply_runtime_config_to_disk(&mut disk, runtime_config);
    }
    let merged_scenes =
        merge_role_pack_scene_ids(role_dir, &disk.scenes).map_err(|error| vec![error])?;
    validate_disk_manifest(&disk, &merged_scenes).map_err(|error| vec![error])?;
    validate_min_runtime_version(disk.min_runtime_version.as_deref(), host_version)
        .map_err(|error| vec![error])?;
    Ok(())
}

/// Load a validated v4 role pack for host activation.
///
/// # Errors
///
/// Same as [`validate_role_pack_blueprint_v4_directory`].
pub fn load_blueprint_v4_for_role_dir(
    role_dir: &Path,
    host_version: &str,
) -> Result<BlueprintV4LoadResult, Vec<String>> {
    validate_role_pack_blueprint_v4_directory(role_dir, host_version)?;
    let blueprint = read_resolved_v4_blueprint(role_dir)?;
    Ok(blueprint_v4_file_to_load_result(&blueprint))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_v4(extra: &str) -> String {
        format!(
            r#"{{
              "schema_version": 4,
              "meta": {{
                "id": "demo",
                "name": "Demo",
                "version": "1.0.0",
                "author": "A",
                "description": "D",
                "relations": {{
                  "friend": {{
                    "display_name": "Friend",
                    "initial_favorability": 50,
                    "favor_multiplier": 1
                  }}
                }},
                "default_relation": "friend"
              }},
              "slot_registry": {{
                "llm": {{
                  "type": "llm",
                  "label": "LLM",
                  "backend": "ollama",
                  "position": 0
                }}
              }}{extra}
            }}"#
        )
    }

    #[test]
    fn validates_minimal_v4_with_extension() {
        let raw = minimal_v4(
            r#",
              "runtime_config": {"interaction_mode": "pure_chat"},
              "extensions": {
                "com.example.live2d": {
                  "capability": "com.example.live2d",
                  "provider": "com.example.live2d.runtime",
                  "required": false,
                  "config_schema_version": 1,
                  "config_ref": "blueprint/extensions/com.example.live2d/config.json"
                }
              }"#,
        );
        validate_blueprint_v4_json(&raw, Some("demo")).unwrap();
    }

    #[test]
    fn rejects_namespaced_ids_with_outer_whitespace() {
        for value in [
            " com.example.live2d",
            "com.example.live2d ",
            "\tcom.example.live2d",
        ] {
            let error = validate_namespaced_id(value).unwrap_err();
            assert!(error.contains("首尾空白"));
        }

        let raw = minimal_v4(
            r#",
              "extensions": {
                "com.example.live2d": {
                  "capability": " com.example.live2d ",
                  "provider": "com.example.live2d.runtime ",
                  "config_schema_version": 1,
                  "config_ref": "blueprint/extensions/com.example.live2d/config.json"
                }
              }"#,
        );
        let message = validate_blueprint_v4_json(&raw, Some("demo"))
            .unwrap_err()
            .join("\n");
        assert!(message.contains("capability 非法"));
        assert!(message.contains("provider 非法"));
        assert!(message.contains("首尾空白"));
    }

    #[test]
    fn rejects_v3_only_and_unknown_fields() {
        for extra in [
            r#","pipeline":{"stable":[]}"#,
            r#","runtime_config":{"dual_core":{"enabled":false}}"#,
            r#","slot_registry":{"llm":{"type":"llm","label":"L","backend":"ollama","position":0,"zone":"stable"}}"#,
        ] {
            let errors = validate_blueprint_v4_json(&minimal_v4(extra), Some("demo")).unwrap_err();
            assert!(!errors.is_empty());
        }
    }

    #[test]
    fn rejects_invalid_extension_contract() {
        let raw = minimal_v4(
            r#",
              "extensions": {
                "live2d": {
                  "capability": "Live2D",
                  "config_schema_version": 0,
                  "config_ref": "../outside.json"
                }
              }"#,
        );
        let errors = validate_blueprint_v4_json(&raw, Some("demo")).unwrap_err();
        let message = errors.join("\n");
        assert!(message.contains("命名空间"));
        assert!(message.contains("config_schema_version"));
        assert!(message.contains("config_ref"));
    }

    #[test]
    fn directory_validation_checks_extension_payload() {
        let temp = tempfile::tempdir().unwrap();
        let role_dir = temp.path().join("demo");
        let payload_dir = role_dir
            .join("blueprint")
            .join("extensions")
            .join("com.example.live2d");
        fs::create_dir_all(&payload_dir).unwrap();
        fs::write(
            role_dir.join(PIPELINE_BLUEPRINT_FILENAME),
            minimal_v4(
                r#",
                  "extensions": {
                    "com.example.live2d": {
                      "capability": "com.example.live2d",
                      "config_schema_version": 1,
                      "config_ref": "blueprint/extensions/com.example.live2d/config.json"
                    }
                  }"#,
            ),
        )
        .unwrap();

        let errors = validate_role_pack_blueprint_v4_directory(&role_dir, "999.0.0").unwrap_err();
        assert!(errors.join("\n").contains("文件不存在"));

        fs::write(payload_dir.join("config.json"), r#"{"enabled":true}"#).unwrap();
        validate_role_pack_blueprint_v4_directory(&role_dir, "999.0.0").unwrap();

        fs::write(payload_dir.join("config.json"), "{").unwrap();
        let errors = validate_role_pack_blueprint_v4_directory(&role_dir, "999.0.0").unwrap_err();
        assert!(errors.join("\n").contains("JSON 解析失败"));
    }

    #[test]
    fn slot_registry_write_preserves_v4_extension_envelope_and_payload() {
        let temp = tempfile::tempdir().unwrap();
        let role_dir = temp.path().join("demo");
        let payload_dir = role_dir
            .join("blueprint")
            .join("extensions")
            .join("com.example.live2d");
        fs::create_dir_all(&payload_dir).unwrap();
        let payload_path = payload_dir.join("config.json");
        fs::write(&payload_path, r#"{"enabled":true}"#).unwrap();
        fs::write(
            role_dir.join(PIPELINE_BLUEPRINT_FILENAME),
            minimal_v4(
                r#",
                  "extensions": {
                    "com.example.live2d": {
                      "capability": "com.example.live2d",
                      "config_schema_version": 1,
                      "config_ref": "blueprint/extensions/com.example.live2d/config.json"
                    }
                  }"#,
            ),
        )
        .unwrap();

        let mut slots = crate::blueprint_dispatch::load_blueprint_slot_registry_for_role_dir(
            &role_dir, "999.0.0",
        )
        .unwrap();
        slots.get_mut("llm").unwrap().label = "Primary LLM".to_string();
        crate::blueprint_v2::write_role_pack_blueprint_slot_registry(&role_dir, &slots, "999.0.0")
            .unwrap();

        let rewritten: Value = serde_json::from_str(
            &fs::read_to_string(role_dir.join(PIPELINE_BLUEPRINT_FILENAME)).unwrap(),
        )
        .unwrap();
        assert_eq!(rewritten["schema_version"], 4);
        assert_eq!(
            rewritten["extensions"]["com.example.live2d"]["config_ref"],
            "blueprint/extensions/com.example.live2d/config.json"
        );
        assert_eq!(rewritten["slot_registry"]["llm"]["label"], "Primary LLM");
        assert_eq!(
            fs::read_to_string(payload_path).unwrap(),
            r#"{"enabled":true}"#
        );
    }
}
