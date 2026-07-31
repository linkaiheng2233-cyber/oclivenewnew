//! Optional `scenes/{scene_id}/scene.json` narrative-continuity contract.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const MAX_INITIAL_STATES: usize = 8;
const MAX_TRANSITIONS: usize = 32;
const MAX_STATE_TEXT_CHARS: usize = 120;
const MAX_MARKER_CHARS: usize = 80;

fn default_weight() -> u32 {
    1
}

/// Time window used to select an initial continuity state.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SceneContinuityTimeWindow {
    pub start: String,
    pub end: String,
}

/// One creator-authored state candidate inside a scene.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SceneContinuityInitialState {
    pub id: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub time_windows: Vec<SceneContinuityTimeWindow>,
    pub sub_location: String,
    pub anchor: String,
    pub posture: String,
    pub activity: String,
}

/// Deterministic state transition recognized from the character's final visible reply.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SceneContinuityTransition {
    #[serde(default)]
    pub from: Vec<String>,
    pub to: String,
    #[serde(default)]
    pub assistant_reply_markers: Vec<String>,
}

/// Scene-local initial states and deterministic transitions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SceneContinuityConfig {
    #[serde(default)]
    pub default_state_id: Option<String>,
    #[serde(default)]
    pub initial_states: Vec<SceneContinuityInitialState>,
    #[serde(default)]
    pub transitions: Vec<SceneContinuityTransition>,
}

fn valid_id(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn text_field_error(scene_id: &str, state_id: &str, field: &str, value: &str) -> Option<String> {
    let chars = value.trim().chars().count();
    if chars == 0 {
        Some(format!(
            "scene continuity「{scene_id}/{state_id}」字段 {field} 不得为空"
        ))
    } else if chars > MAX_STATE_TEXT_CHARS {
        Some(format!(
            "scene continuity「{scene_id}/{state_id}」字段 {field} 超过 {MAX_STATE_TEXT_CHARS} 字符"
        ))
    } else {
        None
    }
}

/// Validate one parsed continuity block.
#[must_use]
pub fn validate_scene_continuity_config(
    scene_id: &str,
    config: &SceneContinuityConfig,
) -> Vec<String> {
    let mut errors = Vec::new();
    if config.initial_states.is_empty() {
        errors.push(format!(
            "scene continuity「{scene_id}」initial_states 至少需要 1 项"
        ));
        return errors;
    }
    if config.initial_states.len() > MAX_INITIAL_STATES {
        errors.push(format!(
            "scene continuity「{scene_id}」initial_states 最多 {MAX_INITIAL_STATES} 项"
        ));
    }
    if config.transitions.len() > MAX_TRANSITIONS {
        errors.push(format!(
            "scene continuity「{scene_id}」transitions 最多 {MAX_TRANSITIONS} 项"
        ));
    }

    let mut state_ids = BTreeSet::new();
    for state in &config.initial_states {
        let state_id = state.id.trim();
        if !valid_id(state_id) {
            errors.push(format!(
                "scene continuity「{scene_id}」state id「{}」须为 1～64 位 ASCII 字母、数字、点、下划线或连字符",
                state.id
            ));
        } else if !state_ids.insert(state_id.to_string()) {
            errors.push(format!(
                "scene continuity「{scene_id}」state id 重复：{state_id}"
            ));
        }
        if !(1..=100).contains(&state.weight) {
            errors.push(format!(
                "scene continuity「{scene_id}/{state_id}」weight 须在 1～100"
            ));
        }
        for (field, value) in [
            ("sub_location", state.sub_location.as_str()),
            ("anchor", state.anchor.as_str()),
            ("posture", state.posture.as_str()),
            ("activity", state.activity.as_str()),
        ] {
            if let Some(error) = text_field_error(scene_id, state_id, field, value) {
                errors.push(error);
            }
        }
        for window in &state.time_windows {
            if crate::validate::parse_hhmm(&window.start).is_none()
                || crate::validate::parse_hhmm(&window.end).is_none()
            {
                errors.push(format!(
                    "scene continuity「{scene_id}/{state_id}」time_windows 须使用 HH:mm：{}～{}",
                    window.start, window.end
                ));
            }
        }
    }

    if let Some(default_id) = config.default_state_id.as_deref() {
        if !state_ids.contains(default_id.trim()) {
            errors.push(format!(
                "scene continuity「{scene_id}」default_state_id 不存在：{default_id}"
            ));
        }
    }

    let mut reply_markers = BTreeSet::new();
    for transition in &config.transitions {
        let target = transition.to.trim();
        if !state_ids.contains(target) {
            errors.push(format!(
                "scene continuity「{scene_id}」transition.to 不存在：{}",
                transition.to
            ));
        }
        for source in &transition.from {
            if !state_ids.contains(source.trim()) {
                errors.push(format!(
                    "scene continuity「{scene_id}」transition.from 不存在：{source}"
                ));
            }
        }
        if transition.assistant_reply_markers.is_empty() {
            errors.push(format!(
                "scene continuity「{scene_id}」指向「{target}」的 transition 至少需要 1 个 assistant_reply_marker"
            ));
        }
        for marker in &transition.assistant_reply_markers {
            let marker = marker.trim();
            let chars = marker.chars().count();
            if chars == 0 || chars > MAX_MARKER_CHARS {
                errors.push(format!(
                    "scene continuity「{scene_id}」transition marker 须为 1～{MAX_MARKER_CHARS} 字符"
                ));
            } else if !reply_markers.insert(marker.to_string()) {
                errors.push(format!(
                    "scene continuity「{scene_id}」transition marker 重复：{marker}"
                ));
            }
        }
    }
    errors
}

/// Validate every optional continuity block under `scenes/*/scene.json`.
///
/// # Errors
///
/// Returns all scene JSON parse and continuity-contract violations.
pub fn validate_scene_continuity_directory(role_dir: &Path) -> Result<(), Vec<String>> {
    let scenes_dir = role_dir.join("scenes");
    if !scenes_dir.is_dir() {
        return Ok(());
    }

    let mut errors = Vec::new();
    let source_entries =
        fs::read_dir(&scenes_dir).map_err(|e| vec![format!("读取 scenes/ 失败: {e}")])?;
    let mut entries = Vec::new();
    for entry in source_entries {
        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!("读取 scenes/ 项失败: {error}"));
                continue;
            }
        };
        entries.push(entry);
    }
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        if !entry.path().is_dir() {
            continue;
        }
        let scene_id = entry.file_name().to_string_lossy().to_string();
        let scene_path = entry.path().join("scene.json");
        if !scene_path.is_file() {
            continue;
        }
        let raw = match fs::read_to_string(&scene_path) {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!("读取 {} 失败: {error}", scene_path.display()));
                continue;
            }
        };
        let root: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!("{} JSON 语法错误: {error}", scene_path.display()));
                continue;
            }
        };
        let Some(value) = root.get("continuity") else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        let config: SceneContinuityConfig = match serde_json::from_value(value.clone()) {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!(
                    "{} continuity 结构不符合契约: {error}",
                    scene_path.display()
                ));
                continue;
            }
        };
        errors.extend(validate_scene_continuity_config(&scene_id, &config));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(id: &str) -> SceneContinuityInitialState {
        SceneContinuityInitialState {
            id: id.to_string(),
            weight: 1,
            time_windows: Vec::new(),
            sub_location: "客厅".into(),
            anchor: "沙发".into(),
            posture: "坐着".into(),
            activity: "聊天".into(),
        }
    }

    #[test]
    fn accepts_valid_scene_continuity() {
        let config = SceneContinuityConfig {
            default_state_id: Some("sofa".into()),
            initial_states: vec![state("sofa"), state("bed")],
            transitions: vec![SceneContinuityTransition {
                from: vec!["sofa".into()],
                to: "bed".into(),
                assistant_reply_markers: vec!["走进卧室".into()],
            }],
        };
        assert!(validate_scene_continuity_config("home", &config).is_empty());
    }

    #[test]
    fn rejects_duplicate_and_unknown_state_references() {
        let config = SceneContinuityConfig {
            default_state_id: Some("missing".into()),
            initial_states: vec![state("same"), state("same")],
            transitions: vec![SceneContinuityTransition {
                from: vec!["unknown".into()],
                to: "missing".into(),
                assistant_reply_markers: Vec::new(),
            }],
        };
        let errors = validate_scene_continuity_config("home", &config);
        assert!(errors.iter().any(|e| e.contains("state id 重复")));
        assert!(errors.iter().any(|e| e.contains("default_state_id")));
        assert!(errors.iter().any(|e| e.contains("transition.from")));
        assert!(errors.iter().any(|e| e.contains("assistant_reply_marker")));
    }

    #[test]
    fn rejects_ambiguous_duplicate_reply_markers() {
        let config = SceneContinuityConfig {
            default_state_id: Some("sofa".into()),
            initial_states: vec![state("sofa"), state("bed"), state("kitchen")],
            transitions: vec![
                SceneContinuityTransition {
                    from: Vec::new(),
                    to: "bed".into(),
                    assistant_reply_markers: vec!["站起来".into()],
                },
                SceneContinuityTransition {
                    from: Vec::new(),
                    to: "kitchen".into(),
                    assistant_reply_markers: vec!["站起来".into()],
                },
            ],
        };
        let errors = validate_scene_continuity_config("home", &config);
        assert!(errors.iter().any(|error| error.contains("marker 重复")));
    }
}
