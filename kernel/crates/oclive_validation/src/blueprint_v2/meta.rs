//! Blueprint meta validation and manifest conversion.

use crate::manifest::DiskRoleManifest;
use crate::role_pack::validate_default_personality_vector;
use serde_json::Value;

use super::{BlueprintMeta, PERSONALITY_OBJECT_KEYS};

/// Shared meta.id / name / relations / personality checks for blueprint v2 and v3.
pub fn validate_blueprint_meta_core(
    meta: &BlueprintMeta,
    folder_name: Option<&str>,
    errs: &mut Vec<String>,
) {
    if let Err(e) = crate::validate::validate_role_id(&meta.id) {
        errs.push(format!("meta.id 非法：{e}"));
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
    for scene_id in &meta.scenes {
        if let Err(e) = crate::validate::validate_scene_id(scene_id) {
            errs.push(format!("meta.scenes 中的「{scene_id}」非法：{e}"));
        }
    }
    if meta.relations.is_empty() {
        errs.push("meta.relations 至少需要配置一种用户身份".into());
    }
    if meta.name.trim().is_empty() {
        errs.push("meta.name 不能为空".into());
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

/// Converts blueprint `meta` to disk manifest (shared by v2/v3/v4).
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
        life_trajectory: meta.life_trajectory.clone(),
        life_schedule: meta.life_schedule.clone(),
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
