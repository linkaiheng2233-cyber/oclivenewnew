//! Portrait director unit test: narrative_hint steers closed-set visual_state_id.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_host::domain::portrait_facility::pick_portrait_with_catalog;
use oclive_kernel_host::infrastructure::llm::LlmClient;
use oclive_kernel_host::infrastructure::storage::RoleStorage;
use oclive_kernel_types::models::{Emotion, PersonalityVector};
use oclivenewnew_tauri::error::Result;
use std::fs;
use std::io::Write;
use std::sync::Arc;

struct TagFromHintLlm;

#[async_trait]
impl LlmClient for TagFromHintLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("ok".to_string())
    }

    async fn generate_tag(&self, _model: &str, prompt: &str) -> Result<String> {
        if prompt.contains("复杂情感叙事提示") {
            Ok("sad_default".to_string())
        } else {
            Ok("happy_default".to_string())
        }
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

struct ExactCatalogIdLlm;

#[async_trait]
impl LlmClient for ExactCatalogIdLlm {
    async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("ok".to_string())
    }

    async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
        Ok("happy_severe".to_string())
    }

    async fn startup_probe(&self) -> Result<()> {
        Ok(())
    }
}

fn write_catalog_role(base: &std::path::Path, role_id: &str) {
    let role_dir = base.join(role_id);
    fs::create_dir_all(role_dir.join("assets/images")).unwrap();
    fs::create_dir_all(role_dir.join("scenes/default")).unwrap();
    for tag in ["happy", "sad"] {
        fs::write(role_dir.join(format!("assets/images/{tag}.webp")), b"x").unwrap();
    }
    let catalog = serde_json::json!({
        "schema_version": 1,
        "assets": [
            { "id": "happy_severe", "path": "assets/images/happy.webp", "desc": "strong happiness", "tags": ["happy"], "kind": "image", "cluster": "happy" },
            { "id": "happy_default", "path": "assets/images/happy.webp", "desc": "开心", "tags": ["happy"], "kind": "image" },
            { "id": "sad_default", "path": "assets/images/sad.webp", "desc": "低落", "tags": ["sad"], "kind": "image" }
        ]
    });
    fs::write(
        role_dir.join("portrait_catalog.json"),
        serde_json::to_string_pretty(&catalog).unwrap(),
    )
    .unwrap();
    fs::write(
        role_dir.join("config.json"),
        r#"{"portrait_catalog":{"enabled":true}}"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "id": role_id,
        "name": "Director Test",
        "version": "0.1.0",
        "author": "t",
        "description": "d",
        "default_personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
        "scenes": ["default"],
        "user_relations": { "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
        "default_relation": "friend"
    });
    let settings = serde_json::json!({ "schema_version": 1, "plugin_backends": {} });
    let mut f = fs::File::create(role_dir.join("manifest.json")).unwrap();
    f.write_all(manifest.to_string().as_bytes()).unwrap();
    let mut f = fs::File::create(role_dir.join("settings.json")).unwrap();
    f.write_all(settings.to_string().as_bytes()).unwrap();
}

#[tokio::test]
async fn narrative_hint_changes_visual_state_id() {
    std::env::set_var("OCLIVE_PORTRAIT_EMOTION_LLM", "1");

    let dir = tempfile::tempdir().unwrap();
    write_catalog_role(dir.path(), "director_role");
    let storage = RoleStorage::new(dir.path());
    let role = storage.load_role("director_role").expect("load");
    let catalog = role.portrait_catalog.as_ref().expect("catalog");
    let llm: Arc<dyn LlmClient> = Arc::new(TagFromHintLlm);
    let core = PersonalityVector::from(&role.default_personality);
    let personality = core.clone();

    let (tag, vsid) = pick_portrait_with_catalog(
        &llm,
        "test-model",
        &role,
        catalog,
        &core,
        &personality,
        50.0,
        "接着说正事",
        "好的呀",
        "neutral",
        &Emotion::Neutral,
        &[],
        &[],
        Some("用户可能缺乏兴致"),
        0.5,
    )
    .await
    .expect("pick");

    assert_eq!(vsid, "sad_default");
    assert_eq!(tag, "sad");
}

#[tokio::test]
async fn without_hint_prefers_happy_id() {
    std::env::set_var("OCLIVE_PORTRAIT_EMOTION_LLM", "1");

    let dir = tempfile::tempdir().unwrap();
    write_catalog_role(dir.path(), "director_role2");
    let storage = RoleStorage::new(dir.path());
    let role = storage.load_role("director_role2").expect("load");
    let catalog = role.portrait_catalog.as_ref().expect("catalog");
    let llm: Arc<dyn LlmClient> = Arc::new(TagFromHintLlm);
    let core = PersonalityVector::from(&role.default_personality);

    let (_tag, vsid) = pick_portrait_with_catalog(
        &llm,
        "test-model",
        &role,
        catalog,
        &core,
        &core,
        50.0,
        "你好",
        "嗨",
        "neutral",
        &Emotion::Neutral,
        &[],
        &[],
        None,
        0.5,
    )
    .await
    .expect("pick");

    // The catalog has a severe variant but no moderate variant; the intensity
    // resolver therefore falls back to the first matching happy asset.
    assert_eq!(vsid, "happy_severe");
}

#[tokio::test]
async fn preserves_exact_catalog_id_within_the_same_emotion_cluster() {
    std::env::set_var("OCLIVE_PORTRAIT_EMOTION_LLM", "1");

    let dir = tempfile::tempdir().unwrap();
    write_catalog_role(dir.path(), "director_role3");
    let storage = RoleStorage::new(dir.path());
    let role = storage.load_role("director_role3").expect("load");
    let catalog = role.portrait_catalog.as_ref().expect("catalog");
    let llm: Arc<dyn LlmClient> = Arc::new(ExactCatalogIdLlm);
    let core = PersonalityVector::from(&role.default_personality);

    let (tag, vsid) = pick_portrait_with_catalog(
        &llm,
        "test-model",
        &role,
        catalog,
        &core,
        &core,
        50.0,
        "a very happy moment",
        "I am especially happy too!",
        "happy",
        &Emotion::Happy,
        &[],
        &[],
        None,
        0.9,
    )
    .await
    .expect("pick");

    assert_eq!(tag, "happy");
    assert_eq!(vsid, "happy_severe");
}
