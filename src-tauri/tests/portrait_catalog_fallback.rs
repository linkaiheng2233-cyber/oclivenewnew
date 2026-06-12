//! Portrait catalog rule fallback: legacy mumu unchanged; B1 catalog resolves visual_state_id.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::domain::portrait_facility::{
    portrait_catalog_active, resolve_visual_state_rule,
};
use oclive_kernel_host::infrastructure::storage::RoleStorage;
use oclive_kernel_types::models::{
    PortraitCatalogAsset, PortraitCatalogFile, SIMPLE_PORTRAIT_SLOT_IDS,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn roles_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[test]
fn mumu_has_no_portrait_catalog_active() {
    let storage = RoleStorage::new(roles_root());
    let role = storage.load_role("mumu").expect("mumu");
    assert!(!portrait_catalog_active(&role));
    assert!(role.portrait_catalog.is_none());
}

#[test]
fn catalog_rule_resolves_seven_slots() {
    let dir = tempfile::tempdir().unwrap();
    let role_dir = dir.path().join("catalog_demo");
    fs::create_dir_all(role_dir.join("assets/images")).unwrap();
    fs::write(role_dir.join("assets/images/happy.webp"), b"x").unwrap();

    let assets: Vec<PortraitCatalogAsset> = SIMPLE_PORTRAIT_SLOT_IDS
        .iter()
        .map(|id| {
            let tag = id.strip_suffix("_default").unwrap_or(id);
            PortraitCatalogAsset {
                id: (*id).to_string(),
                path: format!("assets/images/{tag}.webp"),
                desc: tag.to_string(),
                tags: vec![tag.to_string()],
                kind: Default::default(),
                cluster: None,
                context: None,
                resources: None,
            }
        })
        .collect();
    for a in &assets {
        let p = role_dir.join(&a.path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&p, b"x").ok();
    }

    let catalog = PortraitCatalogFile {
        schema_version: 1,
        assets,
    };
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
        "id": "catalog_demo",
        "name": "Catalog Demo",
        "version": "0.1.0",
        "author": "t",
        "description": "d",
        "default_personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
        "scenes": ["default"],
        "user_relations": { "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
        "default_relation": "friend"
    });
    let settings = serde_json::json!({ "schema_version": 1, "plugin_backends": {} });
    fs::create_dir_all(role_dir.join("scenes/default")).unwrap();
    let mut f = fs::File::create(role_dir.join("manifest.json")).unwrap();
    f.write_all(manifest.to_string().as_bytes()).unwrap();
    let mut f = fs::File::create(role_dir.join("settings.json")).unwrap();
    f.write_all(settings.to_string().as_bytes()).unwrap();

    let storage = RoleStorage::new(dir.path().to_path_buf());
    let role = storage.load_role("catalog_demo").expect("load");
    assert!(portrait_catalog_active(&role));
    let catalog = role.portrait_catalog.as_ref().unwrap();
    assert_eq!(
        resolve_visual_state_rule(catalog, "happy"),
        Some("happy_default".to_string())
    );
    assert_eq!(
        resolve_visual_state_rule(catalog, "unknown_tag"),
        Some("neutral_default".to_string())
    );
}

#[test]
fn disabled_toggle_skips_catalog_load() {
    let dir = tempfile::tempdir().unwrap();
    let role_dir = dir.path().join("off_demo");
    fs::create_dir_all(role_dir.join("scenes/default")).unwrap();
    fs::write(
        role_dir.join("config.json"),
        r#"{"portrait_catalog":{"enabled":false}}"#,
    )
    .unwrap();
    fs::write(
        role_dir.join("portrait_catalog.json"),
        r#"{"schema_version":1,"assets":[]}"#,
    )
    .unwrap();
    let manifest = serde_json::json!({
        "id": "off_demo",
        "name": "Off",
        "version": "0.1.0",
        "author": "t",
        "description": "d",
        "default_personality": [0.5,0.5,0.5,0.5,0.5,0.5,0.5],
        "scenes": ["default"],
        "user_relations": { "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 } },
        "default_relation": "friend"
    });
    let settings = serde_json::json!({ "schema_version": 1, "plugin_backends": {} });
    fs::write(role_dir.join("manifest.json"), manifest.to_string()).unwrap();
    fs::write(role_dir.join("settings.json"), settings.to_string()).unwrap();

    let storage = RoleStorage::new(dir.path().to_path_buf());
    let role = storage.load_role("off_demo").expect("load");
    assert!(!role.pack_portrait_catalog.enabled);
    assert!(role.portrait_catalog.is_none());
    assert!(!portrait_catalog_active(&role));
}
