//! `manifest.json` / `settings.json` top-level key tightening: unknown keys are errors; `_`-prefixed annotation keys are preserved (see roles/README_MANIFEST.md).

use serde_json::{Map, Value};

const MANIFEST_KEYS: &[&str] = &[
    "id",
    "name",
    "version",
    "author",
    "description",
    "ollama_model",
    "model",
    "default_personality",
    "evolution",
    "scenes",
    "user_relations",
    "default_relation",
    "memory_config",
    "identity_binding",
    "life_trajectory",
    "life_schedule",
    "dev_only",
    "knowledge",
    "min_runtime_version",
    "creator_message_to_downloader",
];

const SETTINGS_KEYS: &[&str] = &[
    "schema_version",
    "identity_binding",
    "evolution",
    "memory_config",
    "ollama_model",
    "model",
    "remote_presence",
    "autonomous_scene",
    "interaction_mode",
    "plugin_backends",
    "knowledge",
    "reply_quality_anchor",
];

/// Validate the `manifest.json` root object key names.
///
/// # Errors
///
/// Returns `Err` when a disallowed top-level key is present (details in the error string).
pub fn validate_manifest_top_level_keys(map: &Map<String, Value>) -> Result<(), String> {
    validate_keys(map, MANIFEST_KEYS, "manifest.json")
}

/// Validate the `settings.json` root object key names.
///
/// # Errors
///
/// Returns `Err` when a disallowed top-level key is present (details in the error string).
pub fn validate_settings_top_level_keys(map: &Map<String, Value>) -> Result<(), String> {
    validate_keys(map, SETTINGS_KEYS, "settings.json")
}

fn validate_keys(map: &Map<String, Value>, allowed: &[&str], label: &str) -> Result<(), String> {
    let allowed_set: std::collections::HashSet<&str> = allowed.iter().copied().collect();
    for key in map.keys() {
        if key.starts_with('_') {
            continue;
        }
        if !allowed_set.contains(key.as_str()) {
            return Err(format!(
                "{}：存在未识别的顶层键「{}」。仅允许契约中的字段名，或以「_」开头的说明键（见 oclivenewnew roles/README_MANIFEST.md）。",
                label, key
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_unknown_manifest_key() {
        let map = json!({"id":"x","name":"n","version":"1","author":"a","description":"d",
            "default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"user_relations":{"f":{"prompt_hint":"x"}},
            "default_relation":"f","bad_key":1})
            .as_object()
            .unwrap()
            .clone();
        assert!(validate_manifest_top_level_keys(&map).is_err());
    }

    #[test]
    fn allows_underscore_key() {
        let map = json!({"id":"x","name":"n","version":"1","author":"a","description":"d",
            "default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"user_relations":{"f":{"prompt_hint":"x"}},
            "default_relation":"f","_note":"ok"})
            .as_object()
            .unwrap()
            .clone();
        assert!(validate_manifest_top_level_keys(&map).is_ok());
    }
}
