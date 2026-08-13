//! `pipeline.ocblueprint` schema_version 2 validation (role pack SSOT).

pub use crate::blueprint_v2_slot_registry::{
    apply_slot_override, default_slot_key_for_module, effective_slot_registry,
    merged_agent_directory_plugin_ids, plugin_backends_for_slot_entry,
    slot_registry_instances_sorted, slot_registry_to_plugin_backends, SlotOverridePatch,
};

mod meta;
mod slots;

pub use meta::{meta_to_disk_manifest, validate_blueprint_meta_core, validate_meta_personality};
pub use slots::write_role_pack_blueprint_slot_registry;
pub(crate) use slots::{allowed_backends_for_type, validate_slot_registry_contract};

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blueprint_includes::validate_includes;
use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    LifeScheduleDisk, LifeTrajectoryDisk, MemoryConfigDisk, UserRelationDisk,
};
use crate::role_pack::merge_role_pack_scene_ids;
use crate::runtime_config::RuntimeConfig;
use crate::validate::{
    validate_disk_manifest, validate_interaction_mode_pack_setting,
    validate_knowledge_manifest_disk, validate_min_runtime_version,
};

pub const BLUEPRINT_V2_SCHEMA_VERSION: u32 = 2;
pub const PIPELINE_BLUEPRINT_FILENAME: &str = "pipeline.ocblueprint";

/// Do not persist: `module_relations` is derived at runtime by the host/frontend from `slot_registry` for diagram edges only — do not hand-edit JSON.
const FORBIDDEN_ROOT_KEYS: &[&str] = &["module_relations", "steps", "entry"];
const V2_ROOT_KEYS: &[&str] = &[
    "schema_version",
    "meta",
    "slot_registry",
    "groups",
    "includes",
    "expert_overlay",
    "runtime_config",
];
const V2_SLOT_ENTRY_KEYS: &[&str] = &[
    "type",
    "label",
    "backend",
    "position",
    "plugin",
    "plugins",
    "model",
    "url",
    "local_memory_provider_id",
];

const SLOT_TYPES: &[&str] = &[
    "memory",
    "emotion",
    "event",
    "prompt",
    "llm",
    "agent",
    "complex_emotion",
];

/// Six orchestratable module types (`groups.type` allows only this set; excludes `complex_emotion`).
pub const GROUP_SLOT_TYPES: &[&str] = &["memory", "emotion", "event", "prompt", "llm", "agent"];

const PERSONALITY_OBJECT_KEYS: &[&str] = &[
    "stubbornness",
    "clinginess",
    "sensitivity",
    "assertiveness",
    "forgiveness",
    "talkativeness",
    "warmth",
];

/// One blueprint `groups` entry: logical grouping of `slot_registry` instances sharing a `type`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlotGroupEntry {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub group_type: String,
    pub members: Vec<String>,
}

/// One blueprint `slot_registry` instance (matches `pipeline.ocblueprint` v2 on disk).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotRegistryEntry {
    #[serde(rename = "type")]
    pub slot_type: String,
    pub label: String,
    pub backend: String,
    pub position: i64,
    #[serde(default)]
    pub plugin: Option<String>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    /// v3 optional: `stable` / `experimental` (architecture diagram zone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<serde_json::Value>,
    /// Internal-only merge-policy hook. It is not part of the v2/v3/v4 disk contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
}

/// Validated v2 blueprint load result (for host `RoleStorage` to map into `Role`).
#[derive(Debug, Clone)]
pub struct BlueprintV2LoadResult {
    pub disk: DiskRoleManifest,
    pub slot_registry: BTreeMap<String, SlotRegistryEntry>,
    pub groups: BTreeMap<String, SlotGroupEntry>,
    pub interaction_mode: Option<String>,
    pub featured: bool,
    pub deep_capsule_enabled: bool,
    pub preset_order: u32,
    pub remote_presence: Option<RemotePresenceConfig>,
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

fn default_preset_order() -> u32 {
    999
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BlueprintV2File {
    schema_version: u32,
    meta: BlueprintMeta,
    slot_registry: BTreeMap<String, SlotRegistryEntry>,
    #[serde(default)]
    groups: BTreeMap<String, SlotGroupEntry>,
    /// Satellite file include list (validated after merge for meta / slot_registry).
    #[serde(default)]
    includes: Vec<crate::blueprint_includes::BlueprintIncludeEntry>,
    /// Expert facility pointer (optional; no long-form content).
    #[serde(default)]
    #[allow(dead_code)]
    expert_overlay: Option<BTreeMap<String, serde_json::Value>>,
    /// v3 target segment; passes v2 validation but is not loaded on the v2 path (see `validate_blueprint_json_by_schema_version` warning).
    #[serde(default)]
    #[allow(dead_code)]
    runtime_config: Option<RuntimeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub personality: Option<serde_json::Value>,
    #[serde(default)]
    pub relations: HashMap<String, UserRelationDisk>,
    #[serde(default)]
    pub default_relation: String,
    #[serde(default)]
    pub scenes: Vec<String>,
    #[serde(default)]
    pub evolution: EvolutionConfigDisk,
    #[serde(default)]
    pub memory_config: MemoryConfigDisk,
    #[serde(default)]
    pub identity_binding: IdentityBinding,
    #[serde(default)]
    pub life_trajectory: Option<LifeTrajectoryDisk>,
    #[serde(default)]
    pub life_schedule: Option<LifeScheduleDisk>,
    #[serde(default, alias = "model")]
    pub ollama_model: Option<String>,
    #[serde(default)]
    pub interaction_mode: Option<String>,
    #[serde(default)]
    pub knowledge: Option<KnowledgePackConfigDisk>,
    #[serde(default)]
    pub min_runtime_version: Option<String>,
    #[serde(default)]
    pub dev_only: bool,
    /// Official preset gallery: show in first-run picker when `true`.
    #[serde(default)]
    pub featured: bool,
    /// When true, Small models may use `prompts/deep_capsule.txt` as an offline persona capsule.
    #[serde(default)]
    pub deep_capsule_enabled: bool,
    /// Sort order in preset gallery (lower first); default 999.
    #[serde(default = "default_preset_order")]
    pub preset_order: u32,
    #[serde(default)]
    pub remote_presence: Option<RemotePresenceConfig>,
    #[serde(default)]
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    #[serde(default)]
    pub reply_quality_anchor: Option<String>,
}

/// Optional context: folder name check, `scenes/` merge, `min_runtime_version` vs host.
#[derive(Debug, Clone, Copy, Default)]
pub struct BlueprintV2ValidateContext<'a> {
    /// Role pack folder name, compared with `meta.id` (R4).
    pub folder_name: Option<&'a str>,
    /// When set, merges `scenes/` subdirectory and runs full `validate_disk_manifest`.
    pub role_dir: Option<&'a Path>,
    /// When set, validates `meta.min_runtime_version`.
    pub host_version: Option<&'a str>,
}

/// Validates v2 blueprint JSON text (slots + meta shape; `min_runtime` optional when host version omitted).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when the contract is violated.
pub fn validate_blueprint_v2_json(raw: &str) -> Result<(), Vec<String>> {
    validate_blueprint_v2_json_with_context(raw, BlueprintV2ValidateContext::default())
}

/// v2 blueprint JSON validation with context (shared by CLI / directory validation).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when the contract is violated.
pub fn validate_blueprint_v2_json_with_context(
    raw: &str,
    ctx: BlueprintV2ValidateContext<'_>,
) -> Result<(), Vec<String>> {
    let bp = parse_blueprint_v2_root(raw)?;
    validate_blueprint_v2_file(&bp, ctx)
}

fn parse_blueprint_v2_root(raw: &str) -> Result<BlueprintV2File, Vec<String>> {
    let root: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => return Err(vec![format!("pipeline.ocblueprint JSON 语法错误: {}", e)]),
    };

    let mut errs = Vec::new();
    if let Value::Object(map) = &root {
        for key in FORBIDDEN_ROOT_KEYS {
            if map.contains_key(*key) {
                errs.push(format!(
                    "pipeline.ocblueprint：禁止顶层字段「{key}」（B3：module_relations 仅运行时派生；steps/entry 已废弃）"
                ));
            }
        }
        for key in map.keys() {
            if !V2_ROOT_KEYS.contains(&key.as_str()) {
                errs.push(format!(
                    "pipeline.ocblueprint：未知顶层字段「{key}」（v2 根契约为严格模式）"
                ));
            }
        }
        if let Some(Value::Object(registry)) = map.get("slot_registry") {
            for (slot_key, entry) in registry {
                let Some(entry) = entry.as_object() else {
                    continue;
                };
                for field in entry.keys() {
                    if !V2_SLOT_ENTRY_KEYS.contains(&field.as_str()) {
                        errs.push(format!(
                            "slot_registry[{slot_key}]：未知字段「{field}」（zone 仅属于冻结 v3；policy 未进入 v2/v3/v4 磁盘契约）"
                        ));
                    }
                }
            }
        }
        for field in ["expert_overlay", "runtime_config"] {
            if map.get(field).is_some_and(|value| !value.is_object()) {
                errs.push(format!(
                    "pipeline.ocblueprint.{field} 若存在则须为 JSON 对象"
                ));
            }
        }
    } else {
        return Err(vec!["pipeline.ocblueprint 根节点须为 JSON 对象".into()]);
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    serde_json::from_value(root)
        .map_err(|e| vec![format!("pipeline.ocblueprint 结构不符合 v2 契约: {}", e)])
}

fn validate_blueprint_v2_file(
    bp: &BlueprintV2File,
    ctx: BlueprintV2ValidateContext<'_>,
) -> Result<(), Vec<String>> {
    if let Some(role_dir) = ctx.role_dir {
        if !bp.includes.is_empty() {
            validate_includes(role_dir, &bp.includes)?;
        }
    } else if !bp.includes.is_empty() {
        return Err(vec![
            "includes 校验需要 role_dir 上下文（目录校验 / load 路径）".into(),
        ]);
    }
    validate_blueprint_v2_parsed(bp, ctx.folder_name)?;

    let disk = meta_to_disk_manifest(&bp.meta);
    let merged_scenes = merged_scenes_for_validate(ctx.role_dir, &disk.scenes)?;
    if let Err(e) = validate_disk_manifest(&disk, &merged_scenes) {
        return Err(vec![e]);
    }

    if let Some(ref m) = bp.meta.interaction_mode {
        if let Err(e) = validate_interaction_mode_pack_setting(Some(m.as_str())) {
            return Err(vec![e]);
        }
    }

    if let Some(host) = ctx.host_version {
        if let Err(e) = validate_min_runtime_version(bp.meta.min_runtime_version.as_deref(), host) {
            return Err(vec![e]);
        }
    }

    Ok(())
}

fn merged_scenes_for_validate(
    role_dir: Option<&Path>,
    manifest_scenes: &[String],
) -> Result<Vec<String>, Vec<String>> {
    match role_dir {
        Some(dir) => merge_role_pack_scene_ids(dir, manifest_scenes).map_err(|e| vec![e]),
        None => {
            let mut scenes: Vec<String> = manifest_scenes
                .iter()
                .filter(|s| !s.trim().is_empty())
                .cloned()
                .collect();
            if scenes.is_empty() {
                scenes.push("default".into());
            }
            Ok(scenes)
        }
    }
}

/// Validates role pack directory (v2 SSOT: `pipeline.ocblueprint`; must not contain manifest/settings).
///
/// # Errors
///
/// Returns `Err(Vec<String>)` on missing files, read failures, or validation failure.
pub fn validate_role_pack_blueprint_v2_directory(
    role_dir: &Path,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    let manifest_path = role_dir.join("manifest.json");
    if manifest_path.is_file() {
        errs.push(format!(
            "v2 角色包不得包含 manifest.json（已废弃）：{}",
            manifest_path.display()
        ));
    }
    let settings_path = role_dir.join("settings.json");
    if settings_path.is_file() {
        errs.push(format!(
            "v2 角色包不得包含 settings.json（已废弃）：{}",
            settings_path.display()
        ));
    }

    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        errs.push(format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        ));
        return Err(errs);
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    let raw = match fs::read_to_string(&blueprint_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(vec![format!(
                "读取 {} 失败: {}",
                PIPELINE_BLUEPRINT_FILENAME, e
            )]);
        }
    };

    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_v2_json_with_context(
        &raw,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )?;

    let resolved = crate::blueprint_includes::merge_blueprint_includes_strict(role_dir, &raw)?;
    validate_blueprint_v2_json_with_context(
        &resolved,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )
}

/// Reads and validates v2 blueprint in role pack directory; returns host load structure.
///
/// # Errors
///
/// Same as [`validate_role_pack_blueprint_v2_directory`].
pub fn load_blueprint_v2_for_role_dir(
    role_dir: &Path,
    host_version: &str,
) -> Result<BlueprintV2LoadResult, Vec<String>> {
    validate_role_pack_blueprint_v2_directory(role_dir, host_version)?;
    let raw = fs::read_to_string(role_dir.join(PIPELINE_BLUEPRINT_FILENAME))
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;
    let resolved = crate::blueprint_includes::merge_blueprint_includes_strict(role_dir, &raw)?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_v2_json_with_context(
        &resolved,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )?;
    let bp = parse_blueprint_v2_root(&resolved)?;
    Ok(blueprint_v2_file_to_load_result(&bp))
}

fn blueprint_v2_file_to_load_result(bp: &BlueprintV2File) -> BlueprintV2LoadResult {
    BlueprintV2LoadResult {
        disk: meta_to_disk_manifest(&bp.meta),
        slot_registry: bp.slot_registry.clone(),
        groups: bp.groups.clone(),
        interaction_mode: bp.meta.interaction_mode.clone(),
        featured: bp.meta.featured,
        deep_capsule_enabled: bp.meta.deep_capsule_enabled,
        preset_order: bp.meta.preset_order,
        remote_presence: bp.meta.remote_presence.clone(),
        autonomous_scene: bp.meta.autonomous_scene.clone(),
        reply_quality_anchor: bp.meta.reply_quality_anchor.clone(),
    }
}

fn validate_blueprint_v2_parsed(
    bp: &BlueprintV2File,
    folder_name: Option<&str>,
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();

    if bp.schema_version != BLUEPRINT_V2_SCHEMA_VERSION {
        errs.push(format!(
            "pipeline.ocblueprint：schema_version 须为 {}（当前 {}）",
            BLUEPRINT_V2_SCHEMA_VERSION, bp.schema_version
        ));
    }

    validate_blueprint_meta_core(&bp.meta, folder_name, &mut errs);

    errs.extend(validate_slot_registry_contract(
        &bp.slot_registry,
        &bp.groups,
        false,
    ));

    if let Some(ref k) = bp.meta.knowledge {
        if let Err(e) = validate_knowledge_manifest_disk(k) {
            errs.push(e);
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_emotion_allows_none_backend() {
        let allowed = allowed_backends_for_type("complex_emotion");
        assert!(allowed.contains(&"none"));
        assert!(allowed.contains(&"builtin"));
        assert!(allowed.contains(&"remote"));
        assert!(allowed.contains(&"directory"));
    }
}
