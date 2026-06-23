//! `pipeline.ocblueprint` schema_version 2 validation (role pack SSOT).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::blueprint_includes::validate_includes;
pub use crate::blueprint_v2_slot_registry::{
    apply_slot_override, default_slot_key_for_module, effective_slot_registry,
    merged_agent_directory_plugin_ids, plugin_backends_for_slot_entry,
    slot_registry_instances_sorted, slot_registry_to_plugin_backends, SlotOverridePatch,
};
use crate::disk_role_settings::{AutonomousSceneConfig, RemotePresenceConfig};
use crate::manifest::{
    DiskRoleManifest, EvolutionConfigDisk, IdentityBinding, KnowledgePackConfigDisk,
    MemoryConfigDisk, UserRelationDisk,
};
use crate::role_pack::{merge_role_pack_scene_ids, validate_default_personality_vector};
use crate::runtime_config::RuntimeConfig;
use crate::validate::{
    validate_disk_manifest, validate_interaction_mode_pack_setting,
    validate_knowledge_manifest_disk, validate_min_runtime_version,
};

pub const BLUEPRINT_V2_SCHEMA_VERSION: u32 = 2;
pub const PIPELINE_BLUEPRINT_FILENAME: &str = "pipeline.ocblueprint";

/// Do not persist: `module_relations` is derived at runtime by the host/frontend from `slot_registry` for diagram edges only — do not hand-edit JSON.
const FORBIDDEN_ROOT_KEYS: &[&str] = &["module_relations", "steps", "entry"];

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
    #[serde(default)]
    pub zone: Option<serde_json::Value>,
    /// Multi-instance merge policy: `fastest` | `fallback` | `ensemble` (default; LLM etc. use last-wins).
    #[serde(default)]
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
    pub preset_order: u32,
    pub remote_presence: Option<RemotePresenceConfig>,
    pub autonomous_scene: Option<AutonomousSceneConfig>,
    pub reply_quality_anchor: Option<String>,
}

fn default_preset_order() -> u32 {
    999
}

#[derive(Debug, Clone, Deserialize)]
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
    expert_overlay: Option<serde_json::Value>,
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
    )
}

/// Writes `slot_registry` back to `pipeline.ocblueprint` (keeps `meta` and other fields); full validation before write.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when blueprint is missing, JSON is invalid, or validation fails.
pub fn write_role_pack_blueprint_slot_registry(
    role_dir: &Path,
    slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    host_version: &str,
) -> Result<(), Vec<String>> {
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return Err(vec![format!(
            "缺少 {}：{}",
            PIPELINE_BLUEPRINT_FILENAME,
            blueprint_path.display()
        )]);
    }
    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", PIPELINE_BLUEPRINT_FILENAME, e)])?;
    let mut root: Value = serde_json::from_str(&raw).map_err(|e| {
        vec![format!(
            "{} JSON 解析失败: {}",
            PIPELINE_BLUEPRINT_FILENAME, e
        )]
    })?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| vec![format!("{} 根节点须为对象", PIPELINE_BLUEPRINT_FILENAME)])?;
    let reg_val = serde_json::to_value(slot_registry)
        .map_err(|e| vec![format!("slot_registry 序列化失败: {e}")])?;
    obj.insert("slot_registry".to_string(), reg_val);
    let out = serde_json::to_string_pretty(&root)
        .map_err(|e| vec![format!("{} 序列化失败: {e}", PIPELINE_BLUEPRINT_FILENAME)])?;
    let folder_name = role_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    validate_blueprint_v2_json_with_context(
        &out,
        BlueprintV2ValidateContext {
            folder_name: Some(folder_name),
            role_dir: Some(role_dir),
            host_version: Some(host_version),
        },
    )?;
    fs::write(&blueprint_path, format!("{out}\n"))
        .map_err(|e| vec![format!("写入 {} 失败: {e}", blueprint_path.display())])?;
    Ok(())
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
    let resolved = crate::blueprint_includes::merge_blueprint_includes_lenient(role_dir, &raw);
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

fn validate_blueprint_groups(
    groups: &BTreeMap<String, SlotGroupEntry>,
    registry: &BTreeMap<String, SlotRegistryEntry>,
) -> Vec<String> {
    let mut errs = Vec::new();
    let mut member_owner: HashMap<&str, &str> = HashMap::new();

    for (group_id, group) in groups {
        if group_id.trim().is_empty() {
            errs.push("groups 键名不能为空".into());
            continue;
        }
        if group.label.trim().is_empty() {
            errs.push(format!("groups[{group_id}].label 不能为空"));
        }
        let gt = group.group_type.trim();
        if !GROUP_SLOT_TYPES.contains(&gt) {
            errs.push(format!(
                "groups[{group_id}].type「{gt}」非法（允许: {}）",
                GROUP_SLOT_TYPES.join(", ")
            ));
            continue;
        }
        if group.members.is_empty() {
            errs.push(format!("groups[{group_id}].members 不能为空"));
            continue;
        }
        for member in &group.members {
            let m = member.trim();
            if m.is_empty() {
                errs.push(format!("groups[{group_id}].members 含空键名"));
                continue;
            }
            let Some(slot) = registry.get(m) else {
                errs.push(format!(
                    "groups[{group_id}].members 引用未知 slot_registry 键「{m}」"
                ));
                continue;
            };
            if slot.slot_type.trim() != gt {
                errs.push(format!(
                    "groups[{group_id}].members「{m}」的 type 为「{}」，与 groups.type「{gt}」不一致",
                    slot.slot_type.trim()
                ));
            }
            if let Some(prev) = member_owner.insert(m, group_id.as_str()) {
                errs.push(format!(
                    "slot_registry 键「{m}」同时属于 groups「{prev}」与「{group_id}」"
                ));
            }
        }
    }
    errs
}

fn blueprint_v2_file_to_load_result(bp: &BlueprintV2File) -> BlueprintV2LoadResult {
    BlueprintV2LoadResult {
        disk: meta_to_disk_manifest(&bp.meta),
        slot_registry: bp.slot_registry.clone(),
        groups: bp.groups.clone(),
        interaction_mode: bp.meta.interaction_mode.clone(),
        featured: bp.meta.featured,
        preset_order: bp.meta.preset_order,
        remote_presence: bp.meta.remote_presence.clone(),
        autonomous_scene: bp.meta.autonomous_scene.clone(),
        reply_quality_anchor: bp.meta.reply_quality_anchor.clone(),
    }
}

/// Shared meta.id / name / relations / personality checks for blueprint v2 and v3.
pub fn validate_blueprint_meta_core(
    meta: &BlueprintMeta,
    folder_name: Option<&str>,
    errs: &mut Vec<String>,
) {
    if meta.id.trim().is_empty() {
        errs.push("meta.id 不能为空".into());
    }
    if let Some(dir) = folder_name {
        if meta.id.trim() != dir {
            errs.push(format!(
                "meta.id「{}」与角色包目录名「{}」不一致（R4：ERROR）",
                meta.id.trim(),
                dir
            ));
        }
    }
    if let Some(ref p) = meta.personality {
        if let Err(e) = validate_meta_personality(p) {
            errs.push(e);
        }
    }
    if meta.relations.is_empty() {
        errs.push("meta.relations 至少需要配置一种用户身份".into());
    }
    if meta.name.trim().is_empty() {
        errs.push("meta.name 不能为空".into());
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

    if bp.slot_registry.is_empty() {
        errs.push("slot_registry 不能为空".into());
    }

    let mut llm_count = 0usize;
    let mut positions_by_type: HashMap<&str, HashSet<i64>> = HashMap::new();

    for (key, slot) in &bp.slot_registry {
        if key.trim().is_empty() {
            errs.push("slot_registry 键名不能为空".into());
            continue;
        }
        if slot.label.trim().is_empty() {
            errs.push(format!("slot_registry[{key}].label 不能为空"));
        }

        let t = slot.slot_type.trim();
        if !SLOT_TYPES.contains(&t) {
            errs.push(format!(
                "slot_registry[{key}].type「{t}」非法（允许: {}）",
                SLOT_TYPES.join(", ")
            ));
            continue;
        }

        if slot.position < 0 {
            errs.push(format!("slot_registry[{key}].position 须为非负整数"));
        }

        if !positions_by_type
            .entry(t)
            .or_default()
            .insert(slot.position)
        {
            errs.push(format!(
                "slot_registry：type「{t}」下 position {} 重复（B5）",
                slot.position
            ));
        }

        if t == "llm" {
            llm_count += 1;
        }

        if let Err(e) = validate_slot_backend_and_fields(key, slot) {
            errs.push(e);
        }
    }

    if llm_count == 0 {
        errs.push("slot_registry 须至少包含一个 type: llm 的实例".into());
    }

    errs.extend(validate_blueprint_groups(&bp.groups, &bp.slot_registry));

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

/// Validates `meta.personality` object or seven-dimensional array.
///
/// # Errors
///
/// Returns an `Err` message when a dimension is invalid or outside 0.0–1.0.
pub fn validate_meta_personality(value: &Value) -> Result<(), String> {
    match value {
        Value::Array(arr) => {
            let floats: Result<Vec<f32>, _> = arr
                .iter()
                .map(|v| {
                    v.as_f64()
                        .ok_or_else(|| "personality 数组元素须为数字".to_string())
                        .map(|x| x as f32)
                })
                .collect();
            let floats = floats?;
            validate_default_personality_vector(&floats)
                .map_err(|e| e.replace("manifest：", "meta.personality："))
        }
        Value::Object(map) => {
            let mut vec = Vec::with_capacity(7);
            for key in PERSONALITY_OBJECT_KEYS {
                let Some(v) = map.get(*key) else {
                    return Err(format!(
                        "meta.personality 对象缺少键「{key}」（须含七维: {}）",
                        PERSONALITY_OBJECT_KEYS.join(", ")
                    ));
                };
                let Some(n) = v.as_f64() else {
                    return Err(format!("meta.personality.{key} 须为数字"));
                };
                vec.push(n as f32);
            }
            validate_default_personality_vector(&vec)
                .map_err(|e| e.replace("manifest：", "meta.personality："))
        }
        _ => Err("meta.personality 须为七键对象或长度 7 的数组".into()),
    }
}

fn validate_slot_backend_and_fields(key: &str, slot: &SlotRegistryEntry) -> Result<(), String> {
    let t = slot.slot_type.trim();
    let b = slot.backend.trim();

    let allowed = allowed_backends_for_type(t);
    if !allowed.contains(&b) {
        return Err(format!(
            "slot_registry[{key}]：type「{t}」的 backend「{b}」非法（允许: {}）",
            allowed.join(", ")
        ));
    }

    let has_plugin = slot
        .plugin
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let plugins: Vec<&str> = slot
        .plugins
        .as_ref()
        .map(|ps| {
            ps.iter()
                .map(|s| s.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect()
        })
        .unwrap_or_default();
    let has_plugins = !plugins.is_empty();

    if t != "agent" && has_plugin && has_plugins {
        return Err(format!(
            "slot_registry[{key}]：非 agent 槽位不得同时包含 plugin 与 plugins（S4）"
        ));
    }

    if t == "agent" && b != "directory" && has_plugins {
        return Err(format!(
            "slot_registry[{key}]：agent backend 为「{b}」时不得包含 plugins（S3）"
        ));
    }

    if b == "directory" && !has_plugin && !has_plugins {
        return Err(format!(
            "slot_registry[{key}]：backend 为 directory 时须指定 plugin 或 plugins"
        ));
    }

    if t == "llm" && b == "ollama" {
        if let Some(ref m) = slot.model {
            if m.trim().is_empty() {
                return Err(format!(
                    "slot_registry[{key}]：ollama 槽位的 model 若存在则不得为空"
                ));
            }
        }
    }

    Ok(())
}

fn allowed_backends_for_type(slot_type: &str) -> &'static [&'static str] {
    match slot_type {
        "memory" => &["builtin", "remote", "directory", "local", "none"],
        "emotion" | "event" | "prompt" => &["builtin", "remote", "directory", "none"],
        "llm" => &["ollama", "remote", "directory", "none"],
        "agent" => &["builtin", "remote", "directory", "none"],
        "complex_emotion" => &["builtin", "remote", "directory"],
        _ => &[],
    }
}

/// Converts blueprint `meta` to disk manifest (shared by v2/v3).
#[must_use]
pub fn meta_to_disk_manifest(meta: &BlueprintMeta) -> DiskRoleManifest {
    let default_personality = meta
        .personality
        .as_ref()
        .and_then(personality_to_vector)
        .unwrap_or_default();

    DiskRoleManifest {
        id: meta.id.clone(),
        name: meta.name.clone(),
        version: meta.version.clone(),
        author: meta.author.clone(),
        description: meta.description.clone(),
        ollama_model: meta.ollama_model.clone(),
        default_personality,
        evolution: meta.evolution.clone(),
        scenes: meta.scenes.clone(),
        user_relations: meta.relations.clone(),
        default_relation: meta.default_relation.clone(),
        memory_config: meta.memory_config.clone(),
        identity_binding: meta.identity_binding,
        life_trajectory: None,
        life_schedule: None,
        dev_only: meta.dev_only,
        knowledge: meta.knowledge.clone(),
        min_runtime_version: meta.min_runtime_version.clone(),
    }
}

fn personality_to_vector(value: &Value) -> Option<Vec<f32>> {
    match value {
        Value::Array(arr) => {
            let mut out = Vec::new();
            for v in arr {
                out.push(v.as_f64()? as f32);
            }
            Some(out)
        }
        Value::Object(map) => {
            let mut out = Vec::with_capacity(7);
            for key in PERSONALITY_OBJECT_KEYS {
                out.push(map.get(*key)?.as_f64()? as f32);
            }
            Some(out)
        }
        _ => None,
    }
}
