//! `pipeline.ocblueprint` v2 经 `RoleStorage::load_role_from_dir` 加载。

use oclive_validation::PIPELINE_BLUEPRINT_FILENAME;
use oclivenewnew_tauri::infrastructure::storage::RoleStorage;
use std::fs;
use std::io::Write;

#[test]
fn load_role_from_blueprint_v2_pack() {
    let dir = tempfile::tempdir().unwrap();
    let role_dir = dir.path().join("demo.pack");
    fs::create_dir_all(role_dir.join("scenes").join("default")).unwrap();

    let blueprint = serde_json::json!({
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
            "interaction_mode": "immersive"
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "M", "backend": "builtin", "position": 0 },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0, "model": "llama3.2" }
        }
    });
    let mut f = fs::File::create(role_dir.join(PIPELINE_BLUEPRINT_FILENAME)).unwrap();
    f.write_all(blueprint.to_string().as_bytes()).unwrap();

    let storage = RoleStorage::new(dir.path());
    let role = storage.load_role_from_dir(&role_dir).expect("load v2 role");
    assert_eq!(role.id, "demo.pack");
    assert_eq!(role.interaction_mode.as_deref(), Some("immersive"));
    assert_eq!(role.ollama_model.as_deref(), Some("llama3.2"));
    assert!(role.slot_registry.as_ref().is_some_and(|m| m.contains_key("llm")));
}
