//! Filesystem-only adapter tests. No provider calls or real role packs are used.

use std::fs;
use std::path::Path;

use oclive_validation::minimal_role_local_file::load_minimal_role_local_file;
use oclive_validation::{
    load_minimal_role_local_assets, parse_minimal_role_definition,
    validate_minimal_role_definition, MinimalRoleDefinition,
};

fn role(references: &[&str]) -> MinimalRoleDefinition {
    MinimalRoleDefinition {
        persona_prompt: "A guide.".into(),
        visual_assets: references.iter().map(|value| (*value).into()).collect(),
    }
}

#[test]
fn reads_one_file_without_blueprint_relations_or_catalog() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("portrait.asset"), b"opaque-media-bytes").unwrap();
    let definition = parse_minimal_role_definition(
        r#"{"persona_prompt":"A guide.","visual_assets":["portrait.asset"]}"#,
    )
    .unwrap();
    let assets = load_minimal_role_local_assets(dir.path(), &definition, 32, 32).unwrap();
    assert_eq!(assets, vec![b"opaque-media-bytes".to_vec()]);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert_eq!(definition, role(&["portrait.asset"]));
}

#[test]
fn preserves_order_duplicates_and_unicode_paths_without_media_decoding() {
    let dir = tempfile::tempdir().unwrap();
    let folder = "\u{89c6}\u{89c9} assets";
    fs::create_dir(dir.path().join(folder)).unwrap();
    let relative = format!("{folder}/\u{7acb}\u{7ed8}.asset");
    fs::write(dir.path().join(&relative), b"B").unwrap();
    fs::write(dir.path().join("first"), b"A").unwrap();
    let assets =
        load_minimal_role_local_assets(dir.path(), &role(&[&relative, "first", &relative]), 1, 3)
            .unwrap();
    assert_eq!(assets, vec![b"B".to_vec(), b"A".to_vec(), b"B".to_vec()]);
}

#[test]
fn rejects_nonportable_or_escaping_references_before_root_access() {
    let dir = tempfile::tempdir().unwrap();
    let missing_root = dir.path().join("not-created");
    for reference in [
        "../outside",
        "/absolute",
        "C:/outside",
        "C:relative",
        "image:stream",
        "https://invalid.example/image",
        "file:///image",
        "assets\\image",
        "//server/share",
        "a//b",
        "a/./b",
        "a/../b",
        ".hidden",
        "trailing.",
        " leading",
        "trailing ",
        "CON.png",
        "a/NUL",
        "a/LPT1.jpg",
        "a/",
        "a/\0b",
        "a/*",
    ] {
        let definition = role(&[reference]);
        // These local path restrictions do not change the logical contract.
        validate_minimal_role_definition(&definition).unwrap();
        let errors = load_minimal_role_local_assets(&missing_root, &definition, 8, 8).unwrap_err();
        assert!(
            errors[0].contains("portable relative path"),
            "{reference}: {errors:?}"
        );
    }
    let long_segment = "x".repeat(129);
    assert!(load_minimal_role_local_assets(&missing_root, &role(&[&long_segment]), 8, 8).is_err());
}

#[test]
fn rejects_missing_root_file_root_missing_file_directory_and_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("empty"), b"").unwrap();
    fs::write(dir.path().join("regular"), b"x").unwrap();
    fs::create_dir(dir.path().join("folder")).unwrap();
    for root in [dir.path().join("absent"), dir.path().join("regular")] {
        assert!(load_minimal_role_local_assets(&root, &role(&["regular"]), 8, 8).is_err());
    }
    for reference in ["absent", "folder", "empty"] {
        assert!(load_minimal_role_local_assets(dir.path(), &role(&[reference]), 8, 8).is_err());
    }
}

#[test]
fn enforces_individual_and_aggregate_budgets_including_duplicate_references() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("four"), b"1234").unwrap();
    let single = role(&["four"]);
    let duplicate = role(&["four", "four"]);
    assert!(load_minimal_role_local_assets(dir.path(), &single, 4, 4).is_ok());
    assert!(load_minimal_role_local_assets(dir.path(), &duplicate, 4, 8).is_ok());
    for (definition, per_asset, total) in [
        (&single, 3, 8),
        (&single, 8, 3),
        (&duplicate, 4, 7),
        (&duplicate, 4, 4),
        (&single, 0, 8),
        (&single, 8, 0),
        (&single, usize::MAX, 8),
    ] {
        assert!(load_minimal_role_local_assets(dir.path(), definition, per_asset, total).is_err());
    }
}

#[test]
fn reuses_logical_validation_before_filesystem_access() {
    let dir = tempfile::tempdir().unwrap();
    let mut definition = role(&["asset"]);
    definition.persona_prompt.clear();
    let errors = load_minimal_role_local_assets(dir.path(), &definition, 8, 8).unwrap_err();
    assert!(errors[0].contains("persona_prompt"));
    assert!(load_minimal_role_local_assets(dir.path(), &role(&[]), 8, 8).is_err());
    assert!(load_minimal_role_local_assets(dir.path(), &role(&[" "]), 8, 8).is_err());
}

#[test]
fn returns_owned_snapshots_and_no_partial_success_or_sensitive_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let reference = "private-asset-name";
    fs::write(dir.path().join(reference), b"original").unwrap();
    let assets = load_minimal_role_local_assets(dir.path(), &role(&[reference]), 8, 8).unwrap();
    fs::write(dir.path().join(reference), b"changed").unwrap();
    assert_eq!(assets, vec![b"original".to_vec()]);
    let errors = load_minimal_role_local_assets(dir.path(), &role(&[reference, "missing"]), 8, 16)
        .unwrap_err();
    assert!(errors[0].contains("visual_assets[1]"));
    for error in errors {
        assert!(!error.contains(reference));
        assert!(!error.contains(dir.path().to_string_lossy().as_ref()));
        assert!(!error.contains("A guide."));
    }
}

#[cfg(any(unix, windows))]
fn link_directory(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    {
        // Junctions do not require symlink privileges. Both paths are test-owned
        // temporary directories, never pack-controlled strings or real user data.
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "temporary junction creation failed"
        );
    }
}

#[cfg(any(unix, windows))]
#[test]
fn allows_internal_directory_links_but_rejects_links_outside_the_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("pack");
    let sibling = dir.path().join("pack-sibling");
    fs::create_dir_all(root.join("inside")).unwrap();
    fs::create_dir(&sibling).unwrap();
    fs::write(root.join("inside/image"), b"inside").unwrap();
    fs::write(sibling.join("image"), b"outside").unwrap();
    let raw = r#"{"persona_prompt":"Guide","visual_assets":["inside/image"]}"#;
    fs::write(root.join("inside/content.json"), raw).unwrap();
    fs::write(sibling.join("content.json"), raw).unwrap();
    link_directory(&root.join("inside"), &root.join("internal-link"));
    link_directory(&sibling, &root.join("external-link"));
    let inside = load_minimal_role_local_assets(&root, &role(&["internal-link/image"]), 16, 16);
    let outside = load_minimal_role_local_assets(&root, &role(&["external-link/image"]), 16, 16);
    let definition_inside =
        load_minimal_role_local_file(&root, "internal-link/content.json", raw.len(), 16, 16);
    let definition_outside =
        load_minimal_role_local_file(&root, "external-link/content.json", raw.len(), 16, 16);
    // Remove test links themselves before assertions/temporary directory cleanup.
    #[cfg(windows)]
    for link in ["internal-link", "external-link"] {
        fs::remove_dir(root.join(link)).unwrap();
    }
    #[cfg(unix)]
    for link in ["internal-link", "external-link"] {
        fs::remove_file(root.join(link)).unwrap();
    }
    assert_eq!(inside.unwrap(), vec![b"inside".to_vec()]);
    assert!(outside.unwrap_err()[0].contains("escapes the asset root"));
    assert_eq!(
        definition_inside.unwrap().assets().next().unwrap().1,
        b"inside"
    );
    assert!(definition_outside.unwrap_err()[0].contains("escapes the asset root"));
    assert_eq!(fs::read(sibling.join("image")).unwrap(), b"outside");
}
