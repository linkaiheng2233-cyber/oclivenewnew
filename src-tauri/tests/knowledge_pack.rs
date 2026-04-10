//! 世界观知识包：加载、检索、Prompt 片段与 event_hints 合并。

use oclivenewnew_tauri::domain::knowledge_loader::load_knowledge_index;
use oclivenewnew_tauri::domain::prompt_builder::{PromptBuilder, PromptInput};
use oclivenewnew_tauri::models::knowledge::KnowledgeIndex;
use oclivenewnew_tauri::models::role_manifest_disk::DiskRoleManifest;
use oclivenewnew_tauri::models::{EventType, PersonalityVector};
use std::fs;
use tempfile::tempdir;

#[test]
fn load_knowledge_index_parses_front_matter_and_merge_hints() {
    let root = tempdir().unwrap();
    let role_dir = root.path().join("r1");
    fs::create_dir_all(role_dir.join("knowledge")).unwrap();
    let manifest = r#"{
        "id": "r1",
        "name": "R",
        "version": "1",
        "author": "a",
        "description": "d",
        "default_personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
        "scenes": [],
        "user_relations": {
            "friend": {
                "display_name": "F",
                "prompt_hint": "h",
                "favor_multiplier": 1.0,
                "initial_favorability": 50.0
            }
        },
        "default_relation": "friend",
        "knowledge": { "enabled": true, "glob": "knowledge/**/*.md" }
    }"#;
    fs::write(role_dir.join("manifest.json"), manifest).unwrap();
    fs::write(
        role_dir.join("knowledge/lore.md"),
        "---\nid: alpha\ntags: [雾]\nevent_hints:\n  praise:\n    keywords: [\"神作\"]\n---\n\n故事发生在雾城。",
    )
    .unwrap();

    let disk: DiskRoleManifest = serde_json::from_str(manifest).unwrap();
    let idx = load_knowledge_index(&role_dir, &disk).expect("load knowledge");
    assert_eq!(idx.chunks.len(), 1);
    assert_eq!(idx.chunks[0].id, "alpha");

    let chunks = idx.retrieve("雾城故事", Some("default"), 5);
    assert_eq!(chunks.len(), 1);
    let aug = KnowledgeIndex::merge_event_augment(chunks.as_slice());
    assert_eq!(
        aug.by_event.get(&EventType::Praise).map(|v| v.as_slice()),
        Some(&["神作".to_string()][..])
    );

    let snippet = KnowledgeIndex::format_for_prompt(chunks.as_slice(), 2000);
    let disk_role = oclivenewnew_tauri::models::role_manifest_disk::disk_manifest_to_role(&disk);
    let personality = PersonalityVector::from(&disk_role.default_personality);
    let prompt = PromptBuilder::build_prompt(&PromptInput {
        role: &disk_role,
        personality: &personality,
        memories: &[],
        user_input: "讲讲设定",
        user_emotion: "neutral",
        user_relation_id: "",
        relation_hint: "",
        relation_before: "Stranger",
        favorability_before: 50.0,
        relation_preview: "Stranger",
        favorability_preview: 50.0,
        event_type: &EventType::Ignore,
        impact_factor: 0.0,
        scene_label: "",
        scene_detail: "",
        topic_hint_line: "",
        life_context_line: "",
        worldview_snippet: snippet.as_str(),
        mutable_personality: "",
    });
    assert!(prompt.contains("【世界观设定】"));
    assert!(prompt.contains("雾城"));
}
