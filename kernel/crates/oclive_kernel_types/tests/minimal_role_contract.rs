//! Synthetic adapter mappings through the canonical kernel type and shared validator.
//! These tests do not load a distro, resolve resources, or run a kernel lifecycle.

use oclive_kernel_types::MinimalRoleDefinition;
use oclive_validation::{parse_minimal_role_definition, validate_minimal_role_definition};
use serde::Deserialize;

#[derive(Deserialize)]
struct CompanionPack {
    persona: String,
    portrait: String,
    relations: serde_json::Value,
}

#[derive(Deserialize)]
struct StreamPack {
    role_prompt: String,
    visuals: Vec<String>,
    stream_state: serde_json::Value,
}

#[test]
fn different_product_shapes_map_to_the_same_minimum_without_product_semantics() {
    let companion: CompanionPack = serde_json::from_str(
        r#"{"persona":"Guide","portrait":"image-handle","relations":{"friend":{}}}"#,
    )
    .unwrap();
    let stream: StreamPack = serde_json::from_str(
        r#"{"role_prompt":"Guide","visuals":["image-handle"],"stream_state":{"live":true}}"#,
    )
    .unwrap();
    assert!(companion.relations.is_object());
    assert_eq!(stream.stream_state["live"], true);

    let from_companion = MinimalRoleDefinition {
        persona_prompt: companion.persona,
        visual_assets: vec![companion.portrait],
    };
    let from_stream = MinimalRoleDefinition {
        persona_prompt: stream.role_prompt,
        visual_assets: stream.visuals,
    };
    validate_minimal_role_definition(&from_companion).unwrap();
    validate_minimal_role_definition(&from_stream).unwrap();
    assert_eq!(from_companion, from_stream);
    let round_trip: MinimalRoleDefinition =
        parse_minimal_role_definition(&serde_json::to_string(&from_stream).unwrap()).unwrap();
    assert_eq!(round_trip, from_stream);
}

#[test]
fn adapter_cannot_replace_required_content_with_product_metadata() {
    let no_visual = MinimalRoleDefinition {
        persona_prompt: "Guide".into(),
        visual_assets: vec![],
    };
    assert!(validate_minimal_role_definition(&no_visual).is_err());
    assert!(parse_minimal_role_definition(
        r#"{"meta":{"name":"Guide"},"relations":{"friend":{}},"visual_assets":["image-handle"]}"#
    )
    .is_err());
}
