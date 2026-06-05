//! `pipeline.ocblueprint` → `includes[]` satellite file fetching and merging.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single include: a path relative to the role pack root and its merge target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlueprintIncludeEntry {
    #[serde(default)]
    pub id: Option<String>,
    pub path: String,
    pub target: String,
    pub mode: String,
}

/// Allowed `target` values (dot-separated paths relative to the blueprint root object).
const ALLOWED_TARGETS: &[&str] = &[
    "meta.personality",
    "meta.life_trajectory",
    "meta.life_schedule",
    "runtime_config.expert_hints",
    "expert_overlay",
    "expert_routing",
];

const ALLOWED_MODES: &[&str] = &["merge", "replace"];

/// Validate `includes` entries (path safety + target + mode; the file must exist).
///
/// # Errors
pub fn validate_includes(
    role_dir: &Path,
    includes: &[BlueprintIncludeEntry],
) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    for (idx, inc) in includes.iter().enumerate() {
        let label = inc
            .id
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("includes[{idx}] id={s}"))
            .unwrap_or_else(|| format!("includes[{idx}]"));

        if inc.path.trim().is_empty() {
            errs.push(format!("{label}: path 不能为空"));
            continue;
        }
        if let Err(e) = validate_include_path(role_dir, &inc.path) {
            errs.push(format!("{label}: {e}"));
            continue;
        }
        let file_path = role_dir.join(&inc.path);
        if !file_path.is_file() {
            errs.push(format!("{label}: 文件不存在: {}", file_path.display()));
        }

        let target = inc.target.trim();
        if target.is_empty() {
            errs.push(format!("{label}: target 不能为空"));
        } else if !is_allowed_target(target) {
            errs.push(format!(
                "{label}: target「{target}」非法（允许: {}）",
                ALLOWED_TARGETS.join(", ")
            ));
        }

        let mode = inc.mode.trim().to_ascii_lowercase();
        if !ALLOWED_MODES.contains(&mode.as_str()) {
            errs.push(format!(
                "{label}: mode「{}」非法（允许: merge, replace）",
                inc.mode
            ));
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn is_allowed_target(target: &str) -> bool {
    if ALLOWED_TARGETS.contains(&target) {
        return true;
    }
    // Advanced: allow slot_registry sub-paths (e.g. slot_registry.llm.model).
    target.starts_with("slot_registry.")
}

/// Merge includes into the blueprint JSON text; missing or invalid entries only warn and do not block (for host loading).
#[must_use]
pub fn resolve_blueprint_includes_lenient(role_dir: &Path, raw: &str) -> String {
    let mut root: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };
    let Some(includes) = take_includes(&mut root) else {
        return raw.to_string();
    };
    if includes.is_empty() {
        return serde_json::to_string_pretty(&root).unwrap_or_else(|_| raw.to_string());
    }
    for inc in includes {
        apply_include_lenient(role_dir, &mut root, &inc);
    }
    serde_json::to_string_pretty(&root).unwrap_or_else(|_| raw.to_string())
}

/// Strict merge: includes must pass [`validate_includes`].
///
/// # Errors
pub fn resolve_blueprint_includes_strict(
    role_dir: &Path,
    raw: &str,
) -> Result<String, Vec<String>> {
    let mut root: Value = serde_json::from_str(raw)
        .map_err(|e| vec![format!("pipeline.ocblueprint JSON 语法错误: {e}")])?;
    let includes = take_includes(&mut root).unwrap_or_default();
    if !includes.is_empty() {
        validate_includes(role_dir, &includes)?;
        for inc in &includes {
            apply_include_strict(role_dir, &mut root, inc).map_err(|e| vec![e])?;
        }
    }
    serde_json::to_string_pretty(&root).map_err(|e| vec![format!("合并后序列化失败: {e}")])
}

fn take_includes(root: &mut Value) -> Option<Vec<BlueprintIncludeEntry>> {
    let obj = root.as_object_mut()?;
    let v = obj.remove("includes")?;
    serde_json::from_value(v).ok()
}

fn apply_include_lenient(role_dir: &Path, root: &mut Value, inc: &BlueprintIncludeEntry) {
    let _ = apply_include_strict(role_dir, root, inc);
}

fn apply_include_strict(
    role_dir: &Path,
    root: &mut Value,
    inc: &BlueprintIncludeEntry,
) -> Result<(), String> {
    validate_include_path(role_dir, &inc.path)?;
    let file_path = role_dir.join(&inc.path);
    let fragment: Value =
        serde_json::from_str(&std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("解析 {} 失败: {e}", file_path.display()))?;
    let mode = inc.mode.trim().to_ascii_lowercase();
    apply_at_target(root, inc.target.trim(), &fragment, &mode)?;
    Ok(())
}

fn validate_include_path(role_dir: &Path, path: &str) -> Result<(), String> {
    let rel = path.trim();
    if rel.is_empty() {
        return Err("path 不能为空".into());
    }
    if rel.contains('\\') {
        return Err("path 须使用正斜杠 /".into());
    }
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err("path 须为相对路径".into());
    }
    for c in p.components() {
        match c {
            Component::ParentDir => return Err("path 不得包含 ..".into()),
            Component::RootDir | Component::Prefix(_) => {
                return Err("path 不得为绝对路径".into());
            }
            _ => {}
        }
    }
    let joined = role_dir.join(p);
    if !is_path_inside_role(role_dir, &joined) {
        return Err("path 逃逸角色包目录".into());
    }
    Ok(())
}

fn is_path_inside_role(role_dir: &Path, candidate: &Path) -> bool {
    let role = normalize_path(role_dir);
    let cand = normalize_path(candidate);
    cand.starts_with(&role)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(s) => out.push(s),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::RootDir | Component::Prefix(_) => out.push(c.as_os_str()),
        }
    }
    out
}

fn apply_at_target(
    root: &mut Value,
    target: &str,
    fragment: &Value,
    mode: &str,
) -> Result<(), String> {
    if !is_allowed_target(target) {
        return Err(format!("非法 target: {target}"));
    }
    let parts: Vec<&str> = target.split('.').collect();
    if parts.is_empty() {
        return Err("target 为空".into());
    }
    let mut cursor = root;
    for (i, key) in parts.iter().enumerate() {
        let last = i == parts.len() - 1;
        let obj = cursor
            .as_object_mut()
            .ok_or_else(|| format!("target「{target}」路径无效"))?;
        if last {
            let new_val = match mode {
                "replace" => fragment.clone(),
                "merge" => merge_values(obj.get(*key).cloned().unwrap_or(Value::Null), fragment),
                other => return Err(format!("未知 mode: {other}")),
            };
            obj.insert((*key).to_string(), new_val);
            return Ok(());
        }
        if !obj.contains_key(*key) {
            obj.insert((*key).to_string(), Value::Object(Default::default()));
        }
        cursor = obj
            .get_mut(*key)
            .ok_or_else(|| format!("target「{target}」创建中间节点失败"))?;
        if !cursor.is_object() {
            return Err(format!("target「{target}」中间节点须为对象"));
        }
    }
    Err("target 解析失败".into())
}

fn merge_values(base: Value, patch: &Value) -> Value {
    match (base, patch) {
        (Value::Object(mut a), Value::Object(b)) => {
            for (k, v) in b {
                let key = k.clone();
                let entry = a.remove(&key).unwrap_or(Value::Null);
                a.insert(key, merge_values(entry, v));
            }
            Value::Object(a)
        }
        (_, p) => p.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn validate_rejects_parent_dir_escape() {
        let tmp = TempDir::new().unwrap();
        let inc = BlueprintIncludeEntry {
            id: None,
            path: "../evil.json".into(),
            target: "meta.personality".into(),
            mode: "merge".into(),
        };
        assert!(validate_includes(tmp.path(), &[inc]).is_err());
    }

    #[test]
    fn validate_rejects_illegal_mode() {
        let tmp = TempDir::new().unwrap();
        let role = tmp.path().join("role");
        fs::create_dir_all(&role).unwrap();
        fs::write(role.join("p.json"), r#"{"warmth":0.9}"#).unwrap();
        let inc = BlueprintIncludeEntry {
            id: None,
            path: "p.json".into(),
            target: "meta.personality".into(),
            mode: "file_text".into(),
        };
        assert!(validate_includes(&role, &[inc]).is_err());
    }

    #[test]
    fn resolve_merge_personality() {
        let tmp = TempDir::new().unwrap();
        let role = tmp.path().join("demo");
        fs::create_dir_all(&role).unwrap();
        fs::write(
            role.join("patch.json"),
            r#"{"warmth":0.88,"talkativeness":0.4}"#,
        )
        .unwrap();
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "demo",
            "name": "D",
            "version": "0.1.0",
            "author": "a",
            "description": "d",
            "relations": {"friend":{"initial_favorability":50,"favor_multiplier":1}},
            "default_relation": "friend",
            "personality": {"warmth":0.5,"stubbornness":0.5,"clinginess":0.5,"sensitivity":0.5,"assertiveness":0.5,"forgiveness":0.5,"talkativeness":0.5}
          },
          "slot_registry": {
            "llm": {"type":"llm","label":"L","backend":"builtin","position":0}
          },
          "includes": [
            {"path":"patch.json","target":"meta.personality","mode":"merge"}
          ]
        }"#;
        let out = resolve_blueprint_includes_strict(&role, raw).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let warmth = v["meta"]["personality"]["warmth"].as_f64().unwrap();
        assert!((warmth - 0.88).abs() < f64::EPSILON);
    }

    #[test]
    fn lenient_skips_missing_file() {
        let tmp = TempDir::new().unwrap();
        let role = tmp.path().join("demo");
        fs::create_dir_all(&role).unwrap();
        let raw = r#"{"schema_version":2,"meta":{"id":"demo","name":"D","version":"0.1.0","author":"a","description":"d","relations":{"friend":{"initial_favorability":50,"favor_multiplier":1}},"default_relation":"friend"},"slot_registry":{"llm":{"type":"llm","label":"L","backend":"builtin","position":0}},"includes":[{"path":"missing.json","target":"meta.personality","mode":"merge"}]}"#;
        let out = resolve_blueprint_includes_lenient(&role, raw);
        assert!(out.contains("\"id\": \"demo\""));
    }
}
