//! Read-only loading preparation with synthetic files, never provider traffic.

use std::fs;

use oclive_validation::minimal_role_local_file::load_minimal_role_local_file;
use serde_json::{json, Value};

fn definition(references: &[&str]) -> Value {
    json!({"persona_prompt": "  Private guide.\n", "visual_assets": references})
}

#[test]
fn caller_selects_definition_file_and_assets_stay_relative_to_root() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("authoring")).unwrap();
    let raw = definition(&["portrait.asset"]).to_string();
    fs::write(dir.path().join("authoring/content.json"), &raw).unwrap();
    fs::write(dir.path().join("portrait.asset"), b"opaque-bytes").unwrap();
    // Same name near the definition must not silently change the asset root.
    fs::write(dir.path().join("authoring/portrait.asset"), b"wrong-root").unwrap();

    let snapshot =
        load_minimal_role_local_file(dir.path(), "authoring/content.json", raw.len(), 12, 12)
            .unwrap();
    assert_eq!(snapshot.definition().persona_prompt, "  Private guide.\n");
    assert_eq!(
        serde_json::to_value(snapshot.definition()).unwrap(),
        definition(&["portrait.asset"])
    );
    assert_eq!(
        snapshot.assets().collect::<Vec<_>>(),
        vec![("portrait.asset", &b"opaque-bytes"[..])]
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    assert_eq!(
        fs::read(dir.path().join("authoring/content.json")).unwrap(),
        raw.as_bytes()
    );
}

#[test]
fn unknown_product_fields_are_dropped_without_loading_their_paths_or_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let mut rich = definition(&["image"]);
    rich["relations"] = json!(false);
    rich["default_relation"] = json!(["product-owned"]);
    rich["favorability"] = json!({"product": 42});
    rich["slot_registry"] = json!({"path": "../must-not-be-read"});
    rich["runtime_policy"] = json!("product-owned");
    rich["extensions"] = json!({"visual_slots": {"custom": "not-created"}});
    let raw = rich.to_string();
    fs::write(dir.path().join("content"), &raw).unwrap();
    fs::write(dir.path().join("image"), b"x").unwrap();
    let snapshot = load_minimal_role_local_file(dir.path(), "content", raw.len(), 1, 1).unwrap();
    assert_eq!(
        serde_json::to_value(snapshot.definition()).unwrap(),
        definition(&["image"])
    );
    assert_eq!(snapshot.assets().len(), 1);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn unicode_order_duplicates_and_owned_pairings_survive_later_source_changes() {
    let dir = tempfile::tempdir().unwrap();
    let file_name = "\u{89d2}\u{8272}.json";
    let asset = "\u{7acb}\u{7ed8}.asset";
    let raw = definition(&[asset, "second", asset]).to_string();
    fs::write(dir.path().join(file_name), &raw).unwrap();
    fs::write(dir.path().join(asset), b"A").unwrap();
    fs::write(dir.path().join("second"), b"B").unwrap();
    let snapshot = load_minimal_role_local_file(dir.path(), file_name, raw.len(), 1, 3).unwrap();
    fs::write(dir.path().join(file_name), b"changed after load").unwrap();
    fs::write(dir.path().join(asset), b"changed after load").unwrap();
    fs::remove_file(dir.path().join("second")).unwrap();
    assert_eq!(snapshot.definition().persona_prompt, "  Private guide.\n");
    assert_eq!(
        snapshot.assets().collect::<Vec<_>>(),
        vec![
            (asset, &b"A"[..]),
            ("second", &b"B"[..]),
            (asset, &b"A"[..])
        ]
    );
    assert_eq!(
        snapshot.assets().len(),
        snapshot.definition().visual_assets.len()
    );
    // Default snapshot diagnostics must not dump prompt, names or byte payloads.
    assert_eq!(
        format!("{snapshot:?}"),
        "LocalMinimalRoleSnapshot { asset_count: 3, .. }"
    );
}

#[test]
fn invalid_budgets_are_rejected_before_any_path_or_root_access() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("absent");
    for (definition_limit, asset_limit, total_limit) in [
        (0, 1, 1),
        (1, 0, 1),
        (1, 1, 0),
        (usize::MAX, 1, 1),
        (1, usize::MAX, 1),
    ] {
        let errors = load_minimal_role_local_file(
            &missing,
            "../invalid",
            definition_limit,
            asset_limit,
            total_limit,
        )
        .unwrap_err();
        assert_eq!(
            errors,
            ["minimal role local file: invalid caller byte budgets"]
        );
    }
}

#[test]
fn unsafe_definition_paths_are_rejected_before_root_access() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("absent");
    for reference in [
        "",
        "../outside",
        "/absolute",
        "C:/outside",
        "C:relative",
        "file:stream",
        "https://invalid.example/role",
        "a\\b",
        "//server/share",
        "a//b",
        "a/./b",
        ".hidden",
        "trailing.",
        " leading",
        "trailing ",
        "CON.json",
        "a/NUL",
        "a/",
        "a/\0b",
        "a/*",
    ] {
        let errors = load_minimal_role_local_file(&missing, reference, 1024, 1, 1).unwrap_err();
        assert_eq!(
            errors,
            ["minimal role local file: expected a portable relative definition path"]
        );
    }
    assert!(load_minimal_role_local_file(&missing, &"x".repeat(129), 1024, 1, 1).is_err());
}

#[test]
fn unreadable_nonfile_or_empty_definitions_fail_without_sensitive_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("private-empty"), b"").unwrap();
    fs::write(dir.path().join("private-file"), b"x").unwrap();
    fs::create_dir(dir.path().join("private-folder")).unwrap();
    for (root, reference) in [
        (dir.path().join("private-missing"), "private-file"),
        (dir.path().join("private-file"), "private-file"),
        (dir.path().to_path_buf(), "private-missing"),
        (dir.path().to_path_buf(), "private-folder"),
        (dir.path().to_path_buf(), "private-empty"),
    ] {
        let errors = load_minimal_role_local_file(&root, reference, 1024, 1, 1).unwrap_err();
        assert!(errors[0].starts_with("minimal role local file:"));
        assert!(!errors[0].contains("private-"));
        assert!(!errors[0].contains(dir.path().to_string_lossy().as_ref()));
    }
}

#[test]
fn definition_and_asset_budgets_are_independent_and_duplicates_are_charged() {
    let dir = tempfile::tempdir().unwrap();
    let raw = definition(&["image", "image"]).to_string();
    fs::write(dir.path().join("content.json"), &raw).unwrap();
    fs::write(dir.path().join("image"), b"1234").unwrap();
    let snapshot =
        load_minimal_role_local_file(dir.path(), "content.json", raw.len(), 4, 8).unwrap();
    assert_eq!(snapshot.assets().len(), 2);
    for (definition_limit, asset_limit, total_limit) in
        [(raw.len() - 1, 4, 8), (raw.len(), 3, 8), (raw.len(), 4, 7)]
    {
        let errors = load_minimal_role_local_file(
            dir.path(),
            "content.json",
            definition_limit,
            asset_limit,
            total_limit,
        )
        .unwrap_err();
        assert!(errors[0].contains("byte budget"));
    }
}

#[test]
fn utf8_json_and_logical_content_are_checked_before_asset_access() {
    let dir = tempfile::tempdir().unwrap();
    for (bytes, expected) in [
        (&b"\xff\xfe\x00"[..], "valid UTF-8"),
        (&b"{private-malformed"[..], "invalid JSON projection"),
        (&b"{}"[..], "invalid JSON projection"),
        (
            &b"{\"persona_prompt\":\"private-persona\",\"visual_assets\":\"private-asset\"}"[..],
            "invalid JSON projection",
        ),
        (
            &b"{\"persona_prompt\":\" \t\",\"visual_assets\":[\"absent\"]}"[..],
            "invalid JSON projection",
        ),
        (
            &br#"{"persona_prompt":" ","visual_assets":["absent"]}"#[..],
            "persona_prompt must not be blank",
        ),
        (
            &br#"{"persona_prompt":"Guide","visual_assets":[]}"#[..],
            "at least one asset reference",
        ),
        (
            &br#"{"persona_prompt":"Guide","visual_assets":[""]}"#[..],
            "visual_assets[0] must not be blank",
        ),
    ] {
        fs::write(dir.path().join("content.json"), bytes).unwrap();
        let errors =
            load_minimal_role_local_file(dir.path(), "content.json", 1024, 1, 1).unwrap_err();
        assert!(errors[0].contains(expected), "{errors:?}");
        assert!(!errors[0].contains("private-"));
    }
}

#[test]
fn invalid_or_missing_assets_never_produce_a_partial_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("first"), b"x").unwrap();
    for (reference, expected) in [
        ("../outside", "portable relative path"),
        ("private-missing", "file is not accessible"),
    ] {
        let raw = definition(&["first", reference]).to_string();
        fs::write(dir.path().join("content.json"), &raw).unwrap();
        let errors =
            load_minimal_role_local_file(dir.path(), "content.json", raw.len(), 1, 2).unwrap_err();
        assert!(errors[0].contains("visual_assets[1]"));
        assert!(errors[0].contains(expected));
        assert!(!errors[0].contains(reference));
        assert!(!errors[0].contains("Private guide"));
        assert!(!errors[0].contains(dir.path().to_string_lossy().as_ref()));
    }
}

#[cfg(feature = "media-png")]
#[test]
fn media_adapter_is_an_explicit_next_step_and_does_not_change_the_definition() {
    use oclive_validation::static_png::{
        validate_static_png, StaticPngError, StaticPngInfo, StaticPngInvalid, StaticPngLimits,
        StaticPngUnsupported,
    };
    let dir = tempfile::tempdir().unwrap();
    let raw = definition(&["portrait"]).to_string();
    fs::write(dir.path().join("content.json"), &raw).unwrap();
    let limits = StaticPngLimits {
        max_input_bytes: 1024,
        max_width: 1,
        max_height: 1,
        max_pixels: 1,
        max_output_bytes: 4,
        max_decoder_bytes: 1024 * 1024,
    };
    let mut png_bytes = Vec::new();
    let mut writer = png::Encoder::new(&mut png_bytes, 1, 1)
        .write_header()
        .unwrap();
    writer.write_image_data(b"x").unwrap();
    writer.finish().unwrap();
    for (bytes, expected) in [
        (
            png_bytes.as_slice(),
            Ok(StaticPngInfo {
                width: 1,
                height: 1,
                decoded_bytes: 1,
            }),
        ),
        (
            &b"\xff\xd8\xff\xe0"[..],
            Err(StaticPngError::Unsupported(StaticPngUnsupported::NotPng)),
        ),
        (
            &b"RIFF\x00\x00\x00\x00WEBP"[..],
            Err(StaticPngError::Unsupported(StaticPngUnsupported::NotPng)),
        ),
        (
            &b"\x89PNG\r\n\x1a\n"[..],
            Err(StaticPngError::Invalid(StaticPngInvalid::Malformed)),
        ),
    ] {
        fs::write(dir.path().join("portrait"), bytes).unwrap();
        let snapshot =
            load_minimal_role_local_file(dir.path(), "content.json", raw.len(), 1024, 1024)
                .unwrap();
        let (_, media) = snapshot.assets().next().unwrap();
        assert_eq!(validate_static_png(media, &limits), expected);
        assert_eq!(
            serde_json::to_value(snapshot.definition()).unwrap(),
            definition(&["portrait"])
        );
    }
}
