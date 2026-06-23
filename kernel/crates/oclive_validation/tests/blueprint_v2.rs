use std::collections::BTreeMap;
use std::fs;

use oclive_validation::{
    load_blueprint_v2_for_role_dir, slot_registry_to_plugin_backends, validate_blueprint_v2_json,
    validate_role_pack_blueprint_v2_directory, write_role_pack_blueprint_slot_registry, LlmBackend,
    SlotRegistryEntry, PIPELINE_BLUEPRINT_FILENAME,
};
use serde_json::Value;

fn minimal_v2_json() -> String {
    r#"{
          "schema_version": 2,
          "meta": {
            "id": "demo.pack",
            "name": "Demo",
            "version": "0.1.0",
            "author": "t",
            "description": "d",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
              "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend",
            "scenes": ["default"]
          },
          "slot_registry": {
            "memory": { "type": "memory", "label": "Memory", "backend": "builtin", "position": 0 },
            "emotion": { "type": "emotion", "label": "Emotion", "backend": "builtin", "position": 0 },
            "event": { "type": "event", "label": "Event", "backend": "builtin", "position": 0 },
            "prompt": { "type": "prompt", "label": "Prompt", "backend": "builtin", "position": 0 },
            "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 },
            "agent": { "type": "agent", "label": "Agent", "backend": "builtin", "position": 0 },
            "complex_emotion": { "type": "complex_emotion", "label": "Complex", "backend": "builtin", "position": 1 }
          }
        }"#
    .to_string()
}

#[test]
fn valid_minimal_v2_passes() {
    validate_blueprint_v2_json(&minimal_v2_json()).unwrap();
}

#[test]
fn rejects_module_relations() {
    let mut v: Value = serde_json::from_str(&minimal_v2_json()).unwrap();
    v.as_object_mut()
        .unwrap()
        .insert("module_relations".into(), serde_json::json!({}));
    let errs = validate_blueprint_v2_json(&v.to_string()).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("module_relations")));
}

#[test]
fn rejects_schema_version_not_2() {
    let raw = minimal_v2_json().replace("\"schema_version\": 2", "\"schema_version\": 1");
    let errs = validate_blueprint_v2_json(&raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("schema_version")));
}

#[test]
fn rejects_no_llm() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "memory": { "type": "memory", "label": "M", "backend": "builtin", "position": 0 }
          }
        }"#;
    let errs = validate_blueprint_v2_json(raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("llm")));
}

#[test]
fn rejects_duplicate_position_same_type() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm_a": { "type": "llm", "label": "A", "backend": "ollama", "position": 0 },
            "llm_b": { "type": "llm", "label": "B", "backend": "ollama", "position": 0 }
          }
        }"#;
    let errs = validate_blueprint_v2_json(raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("position")));
}

#[test]
fn rejects_invalid_relation_favorability_via_disk_manifest() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 200.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0 }
          }
        }"#;
    let errs = validate_blueprint_v2_json(raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("favorability")));
}

#[test]
fn slot_registry_last_wins_maps_to_plugin_backends() {
    let mut reg = BTreeMap::new();
    reg.insert(
        "llm_a".into(),
        SlotRegistryEntry {
            slot_type: "llm".into(),
            label: "A".into(),
            backend: "ollama".into(),
            position: 0,
            plugin: None,
            plugins: None,
            model: Some("model-a".into()),
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    reg.insert(
        "llm_b".into(),
        SlotRegistryEntry {
            slot_type: "llm".into(),
            label: "B".into(),
            backend: "remote".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    let pb = slot_registry_to_plugin_backends(&reg);
    assert_eq!(pb.llm, LlmBackend::Remote);
}

#[test]
fn blueprint_v2_directory_minimal_pack() {
    let dir = tempfile::tempdir().unwrap();
    let role = dir.path().join("demo.pack");
    fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    let bp = minimal_v2_json();
    fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), bp).unwrap();
    validate_role_pack_blueprint_v2_directory(&role, "999.0.0").unwrap();
}

#[test]
fn blueprint_v2_directory_rejects_legacy_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let role = dir.path().join("demo.pack");
    fs::create_dir_all(&role).unwrap();
    fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), minimal_v2_json()).unwrap();
    fs::write(role.join("manifest.json"), "{}").unwrap();
    let errs = validate_role_pack_blueprint_v2_directory(&role, "999.0.0").unwrap_err();
    assert!(errs.iter().any(|e| e.contains("manifest.json")));
}

#[test]
fn rejects_directory_without_plugin() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "directory", "position": 0 }
          }
        }"#;
    let errs = validate_blueprint_v2_json(raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("directory")));
}

#[test]
fn groups_valid_when_members_match_type() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "mem_a": { "type": "memory", "label": "A", "backend": "builtin", "position": 0 },
            "mem_b": { "type": "memory", "label": "B", "backend": "builtin", "position": 1 },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0 }
          },
          "groups": {
            "mem_group": {
              "label": "Memory tier",
              "type": "memory",
              "members": ["mem_a", "mem_b"]
            }
          }
        }"#;
    assert!(validate_blueprint_v2_json(raw).is_ok());
}

#[test]
fn groups_reject_empty_members_and_type_mismatch() {
    let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0 }
          },
          "groups": {
            "bad": { "label": "G", "type": "memory", "members": [] }
          }
        }"#;
    let errs = validate_blueprint_v2_json(raw).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("members")));

    let raw2 = r#"{
          "schema_version": 2,
          "meta": {
            "id": "x", "name": "X", "version": "0.1.0", "author": "a", "description": "d",
            "relations": { "f": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
            "default_relation": "f"
          },
          "slot_registry": {
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0 }
          },
          "groups": {
            "bad": { "label": "G", "type": "memory", "members": ["llm"] }
          }
        }"#;
    let errs2 = validate_blueprint_v2_json(raw2).unwrap_err();
    assert!(errs2
        .iter()
        .any(|e| e.contains("不一致") || e.contains("type")));
}

#[test]
fn write_role_pack_blueprint_slot_registry_persists_and_validates() {
    let dir = tempfile::tempdir().unwrap();
    let role = dir.path().join("demo.pack");
    fs::create_dir_all(role.join("scenes").join("default")).unwrap();
    fs::write(role.join(PIPELINE_BLUEPRINT_FILENAME), minimal_v2_json()).unwrap();
    let mut reg = BTreeMap::new();
    reg.insert(
        "llm".into(),
        SlotRegistryEntry {
            slot_type: "llm".into(),
            label: "L".into(),
            backend: "remote".into(),
            position: 0,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    write_role_pack_blueprint_slot_registry(&role, &reg, "999.0.0").unwrap();
    let loaded = load_blueprint_v2_for_role_dir(&role, "999.0.0").unwrap();
    assert_eq!(loaded.slot_registry.get("llm").unwrap().backend, "remote");
}
