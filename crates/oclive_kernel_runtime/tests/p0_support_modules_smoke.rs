//! P0.T：支撑模块烟测（无网络、无完整 `KernelAppState`），纳入 `cargo test -p oclive_kernel_runtime`。

use oclive_kernel_runtime::domain::expert_models::compile_graph_to_llama_local_config;
use oclive_kernel_runtime::domain::local_imports::{list_local_import_candidates, LocalImportKind};
#[cfg(feature = "market-sync")]
use oclive_kernel_runtime::error::AppError;
use oclive_kernel_runtime::infrastructure::plugin_install::missing_plugin_dependencies;
#[cfg(feature = "market-sync")]
use oclive_kernel_runtime::infrastructure::{
    plugin_index_sync, plugin_reviews_index_sync, role_market_index_sync,
};
use oclive_kernel_runtime::models::expert_models::{ExpertGraph, ExpertNode};
use semver::Version;
use std::collections::HashMap;
use std::fs;

#[test]
fn missing_plugin_dependencies_empty_ok() {
    let installed = HashMap::new();
    let deps = HashMap::new();
    let miss = missing_plugin_dependencies(&installed, &deps).unwrap();
    assert!(miss.is_empty());
}

#[test]
fn missing_plugin_dependencies_detects_absent_id() {
    let installed = HashMap::new();
    let mut deps = HashMap::new();
    deps.insert("need_me".to_string(), "^1.0".to_string());
    let miss = missing_plugin_dependencies(&installed, &deps).unwrap();
    assert_eq!(miss.len(), 1);
    assert!(miss[0].contains("need_me"));
}

#[test]
fn missing_plugin_dependencies_invalid_range_is_invalid_parameter() {
    let installed = HashMap::new();
    let mut deps = HashMap::new();
    deps.insert("a".to_string(), "not-semver".to_string());
    let e = missing_plugin_dependencies(&installed, &deps).unwrap_err();
    assert_eq!(e.code(), "INVALID_PARAMETER");
}

#[test]
fn missing_plugin_dependencies_respects_installed_version() {
    let mut installed = HashMap::new();
    installed.insert("x".to_string(), Version::new(1, 2, 3));
    let mut deps = HashMap::new();
    deps.insert("x".to_string(), "^1.0".to_string());
    assert!(missing_plugin_dependencies(&installed, &deps)
        .unwrap()
        .is_empty());

    deps.insert("x".to_string(), "^2.0".to_string());
    let miss = missing_plugin_dependencies(&installed, &deps).unwrap();
    assert_eq!(miss.len(), 1);
}

#[test]
fn compile_graph_single_base_under_models_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let gguf = tmp.path().join("models").join("gguf");
    fs::create_dir_all(&gguf).unwrap();
    let model_path = gguf.join("stub.gguf");
    fs::write(&model_path, b"stub").unwrap();
    let loras = tmp.path().join("models").join("loras");
    fs::create_dir_all(&loras).unwrap();

    let graph = ExpertGraph {
        version: 1,
        nodes: vec![ExpertNode::BaseModel {
            id: "base1".to_string(),
            gguf_path: model_path.to_string_lossy().to_string(),
            ui: None,
        }],
        edges: vec![],
    };

    let cfg = compile_graph_to_llama_local_config(&graph, gguf.as_path(), loras.as_path()).unwrap();
    assert_eq!(
        cfg.model_path.as_deref(),
        Some(model_path.to_string_lossy().as_ref())
    );
    assert!(cfg.llama_args.is_none());
}

#[test]
fn list_local_import_candidates_sees_role_pack_and_plugin_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("imports");
    oclive_kernel_runtime::domain::local_imports::ensure_import_folders_exist(&root).unwrap();

    let roles = root.join("roles");
    fs::write(roles.join("pack.ocpak"), b"x").unwrap();

    let plugin_slot = root.join("plugins").join("plugin");
    let plug_dir = plugin_slot.join("myplug");
    fs::create_dir_all(&plug_dir).unwrap();
    fs::write(plug_dir.join("manifest.json"), b"{}").unwrap();

    let list = list_local_import_candidates(&root).unwrap();
    let kinds: Vec<_> = list.iter().map(|c| &c.kind).collect();
    assert!(kinds.contains(&&LocalImportKind::RolePack));
    assert!(kinds.contains(&&LocalImportKind::PluginDir));
}

#[cfg(feature = "market-sync")]
#[test]
fn plugin_index_cache_missing_file_yields_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("nope.json");
    let f = plugin_index_sync::load_plugin_index_cache(&p).unwrap();
    assert!(f.plugins.is_empty());
}

#[cfg(feature = "market-sync")]
#[test]
fn plugin_index_cache_invalid_json_is_serde_error() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("bad.json");
    fs::write(&p, b"{").unwrap();
    let e = plugin_index_sync::load_plugin_index_cache(&p).unwrap_err();
    assert_eq!(e.code(), "SERDE_ERROR");
}

#[cfg(feature = "market-sync")]
#[test]
fn plugin_index_cache_roundtrip_minimal_json() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("cache.json");
    fs::write(&p, br#"{"plugins":[]}"#).unwrap();
    let f = plugin_index_sync::load_plugin_index_cache(&p).unwrap();
    assert!(f.plugins.is_empty());
}

#[cfg(feature = "market-sync")]
#[test]
fn role_market_index_cache_missing_file_yields_empty_roles() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("roles.json");
    let f = role_market_index_sync::load_role_market_index_cache(&p).unwrap();
    assert!(f.roles.is_empty());
}

#[cfg(feature = "market-sync")]
#[test]
fn role_market_index_cache_invalid_json_is_serde_error() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("bad.json");
    fs::write(&p, b"not-json").unwrap();
    let e = role_market_index_sync::load_role_market_index_cache(&p).unwrap_err();
    assert!(matches!(e, AppError::SerializationError(_)));
}

#[cfg(feature = "market-sync")]
#[test]
fn plugin_reviews_cache_missing_file_yields_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("reviews.json");
    let f = plugin_reviews_index_sync::load_plugin_reviews_index_cache(&p).unwrap();
    assert!(f.reviews.is_empty());
}

#[cfg(feature = "market-sync")]
#[test]
fn plugin_reviews_cache_roundtrip_minimal_json() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("reviews.json");
    fs::write(
        &p,
        br#"{"schemaVersion":1,"generatedAt":null,"reviews":[]}"#,
    )
    .unwrap();
    let f = plugin_reviews_index_sync::load_plugin_reviews_index_cache(&p).unwrap();
    assert_eq!(f.schema_version, 1);
    assert!(f.reviews.is_empty());
}

#[cfg(feature = "market-sync")]
#[test]
fn resolve_plugin_index_url_override_wins() {
    let u =
        plugin_index_sync::resolve_plugin_index_url(Some("  https://example.invalid/index.json  "));
    assert_eq!(u, "https://example.invalid/index.json");
}
