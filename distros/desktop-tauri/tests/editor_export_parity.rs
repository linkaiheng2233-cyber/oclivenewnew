//! Smoke: editor-shaped export tree (pipeline + core_personality + config + anchor meta) loads via RoleStorage.

use oclive_kernel_host::infrastructure::storage::RoleStorage;
use oclive_validation::validate_role_pack_blueprint_v2_directory;
use oclive_validation::PIPELINE_BLUEPRINT_FILENAME;
use std::fs;
use tempfile::tempdir;

#[test]
fn editor_export_shape_validates_and_loads_role() {
    let root = tempdir().unwrap();
    let role_dir = root.path().join("editor_demo");
    fs::create_dir_all(role_dir.join("scenes/home")).unwrap();
    fs::create_dir_all(role_dir.join("prompts")).unwrap();

    let blueprint = serde_json::json!({
        "schema_version": 2,
        "meta": {
            "id": "editor_demo",
            "name": "Editor Demo",
            "version": "0.1.0",
            "author": "test",
            "description": "d",
            "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            "relations": {
                "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
            },
            "default_relation": "friend",
            "interaction_mode": "pure_chat",
            "featured": true,
            "preset_order": 2,
            "reply_quality_anchor": "【包级锚点】测试用。"
        },
        "slot_registry": {
            "memory": { "type": "memory", "label": "M", "backend": "builtin", "position": 0 },
            "emotion": { "type": "emotion", "label": "E", "backend": "builtin", "position": 0 },
            "event": { "type": "event", "label": "Ev", "backend": "builtin", "position": 0 },
            "prompt": { "type": "prompt", "label": "P", "backend": "builtin", "position": 0 },
            "llm": { "type": "llm", "label": "L", "backend": "ollama", "position": 0, "model": "qwen2.5:7b" },
            "agent": { "type": "agent", "label": "A", "backend": "builtin", "position": 0 },
            "complex_emotion": { "type": "complex_emotion", "label": "CE", "backend": "builtin", "position": 1 }
        }
    });
    fs::write(
        role_dir.join(PIPELINE_BLUEPRINT_FILENAME),
        serde_json::to_string_pretty(&blueprint).unwrap(),
    )
    .unwrap();
    fs::write(role_dir.join("core_personality.txt"), "测试人设\n").unwrap();
    fs::write(
        role_dir.join("config.json"),
        r#"{"reply_post_processor":{"enabled":false}}"#,
    )
    .unwrap();
    fs::write(
        role_dir.join("prompts/reply_quality_anchor.md"),
        "【包级锚点】测试用。\n",
    )
    .unwrap();
    fs::write(
        role_dir.join("scenes/home/scene.json"),
        r#"{"name":"home"}"#,
    )
    .unwrap();
    fs::write(role_dir.join("scenes/home/description.txt"), "home\n").unwrap();
    fs::create_dir_all(role_dir.join("user_identities")).unwrap();
    fs::write(
        role_dir.join("user_identities/index.json"),
        r#"{"schema_version":1,"default_identity_id":"friend","identities":{"friend":{"display_name":"好友","template_file":"friend.md"}}}"#,
    )
    .unwrap();
    fs::write(role_dir.join("user_identities/friend.md"), "好友身份模板\n").unwrap();

    validate_role_pack_blueprint_v2_directory(&role_dir, "0.3.0").expect("pack validate");

    let storage = RoleStorage::new(root.path());
    let role = storage
        .load_role_from_dir(&role_dir)
        .expect("load editor export shape");
    assert_eq!(role.id, "editor_demo");
    assert_eq!(role.interaction_mode.as_deref(), Some("pure_chat"));
    assert!(role.featured);
    assert_eq!(role.preset_order, 2);
    assert_eq!(
        role.reply_quality_anchor.as_deref(),
        Some("【包级锚点】测试用。")
    );
    assert!(!role.core_personality.is_empty());
}
