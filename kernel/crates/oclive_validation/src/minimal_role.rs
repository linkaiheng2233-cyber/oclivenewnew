//! Shared logical role content, independent of distro pack formats and host assembly.
//!
//! Asset strings are opaque references supplied by an adapter. This module performs
//! no I/O: passing validation does not prove that a referenced asset exists, is safe
//! to open, or can be rendered. The adapter must establish those properties before
//! use. This JSON projection is not an on-disk pack format or a lifecycle loader.

use serde::{Deserialize, Serialize};

/// Minimum authored role content: a persona prompt and at least one visual asset reference.
///
/// Relations, identity/version envelopes, backend bindings, and visual slot semantics
/// belong to adapters or distros. Unknown JSON fields are ignored, not retained or
/// interpreted; this type is not a lossless editor model for richer product packs.
/// Deserialization alone does not validate non-empty content; use
/// [`parse_minimal_role_definition`] or [`validate_minimal_role_definition`].
/// Kernel consumers import this same type from `oclive_kernel_types`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimalRoleDefinition {
    /// Non-empty authored persona prose, preserved verbatim.
    pub persona_prompt: String,
    /// One or more non-empty adapter-owned asset references, preserved in input order.
    /// No filename, URI scheme, seven-image set, or emotion label is imposed here.
    pub visual_assets: Vec<String>,
}

/// Check minimum content without resolving resources or supplying product defaults.
///
/// # Errors
///
/// Returns field/index diagnostics for a blank prompt, an empty asset list, or any
/// blank asset reference. Asset existence and media validity are outside this check.
///
/// ```
/// use oclive_validation::{MinimalRoleDefinition, validate_minimal_role_definition};
///
/// let role = MinimalRoleDefinition {
///     persona_prompt: "A curious guide.".into(),
///     visual_assets: vec!["adapter-owned-portrait".into()],
/// };
/// assert!(validate_minimal_role_definition(&role).is_ok());
/// ```
pub fn validate_minimal_role_definition(role: &MinimalRoleDefinition) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if role.persona_prompt.trim().is_empty() {
        errors.push("minimal role: persona_prompt must not be blank".into());
    }
    if role.visual_assets.is_empty() {
        errors.push("minimal role: visual_assets must contain at least one asset reference".into());
    }
    for (index, asset) in role.visual_assets.iter().enumerate() {
        if asset.trim().is_empty() {
            errors.push(format!(
                "minimal role: visual_assets[{index}] must not be blank"
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Parse the JSON projection of minimum role content and check its logical requirements.
///
/// Adapters validate their own product/transport version before producing this input.
/// No file is opened and no relation, backend, or default visual slot is injected.
///
/// # Errors
///
/// Returns diagnostics for invalid JSON, missing/wrongly typed required fields, or
/// invalid logical content. Diagnostics do not include persona or asset values.
pub fn parse_minimal_role_definition(raw: &str) -> Result<MinimalRoleDefinition, Vec<String>> {
    let role: MinimalRoleDefinition = serde_json::from_str(raw).map_err(|error| {
        vec![format!(
            "minimal role: invalid JSON projection at line {}, column {} ({:?})",
            error.line(),
            error.column(),
            error.classify()
        )]
    })?;
    validate_minimal_role_definition(&role)?;
    Ok(role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn one_asset_and_prompt_are_sufficient_and_preserved_verbatim() {
        let raw = json!({
            "persona_prompt": "  A curious guide.\n",
            "visual_assets": ["asset:portrait"]
        });
        let role = parse_minimal_role_definition(&raw.to_string()).unwrap();
        assert_eq!(role.persona_prompt, "  A curious guide.\n");
        assert_eq!(role.visual_assets, ["asset:portrait"]);
        assert_eq!(serde_json::to_value(role).unwrap(), raw);
    }

    #[test]
    fn missing_null_or_wrongly_typed_required_fields_are_rejected() {
        for raw in [
            json!({}),
            json!({"visual_assets": ["portrait"]}),
            json!({"persona_prompt": "Guide"}),
            json!({"persona_prompt": null, "visual_assets": ["portrait"]}),
            json!({"persona_prompt": 42, "visual_assets": ["portrait"]}),
            json!({"persona_prompt": "Guide", "visual_assets": null}),
            json!({"persona_prompt": "Guide", "visual_assets": "portrait"}),
            json!({"persona_prompt": "Guide", "visual_assets": [null]}),
            json!({"persona_prompt": "Guide", "visual_assets": [{"path": "portrait"}]}),
        ] {
            assert!(parse_minimal_role_definition(&raw.to_string()).is_err());
        }
    }

    #[test]
    fn blank_content_is_rejected_even_when_constructed_directly() {
        let role = MinimalRoleDefinition {
            persona_prompt: " \t\n\u{3000}".into(),
            visual_assets: vec!["first".into(), " \t\u{3000}".into()],
        };
        let errors = validate_minimal_role_definition(&role).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("persona_prompt"));
        assert!(errors[1].contains("visual_assets[1]"));
        assert!(
            parse_minimal_role_definition(r#"{"persona_prompt":"Guide","visual_assets":[]}"#)
                .is_err()
        );
        assert!(parse_minimal_role_definition(
            r#"{"persona_prompt":"","visual_assets":["portrait"]}"#
        )
        .is_err());
    }

    #[test]
    fn product_fields_are_opaque_and_cannot_inject_defaults() {
        let minimum = json!({"persona_prompt": "Guide", "visual_assets": ["portrait"]});
        // Even values incompatible with the reference host's relation/blueprint
        // schemas must not affect this independent content projection.
        let mut rich = minimum.clone();
        rich["relations"] = json!(false);
        rich["default_relation"] = json!(["product-owned"]);
        rich["favorability"] = json!("product-owned");
        rich["slot_registry"] = json!(false);
        rich["runtime_config"] = json!([1, 2]);
        rich["extensions"] = json!({"visual_slots": {"custom": "portrait"}});
        rich["meta"] = json!({"name": "A product display name", "version": "product-v9"});

        let role = parse_minimal_role_definition(&rich.to_string()).unwrap();
        assert_eq!(serde_json::to_value(role).unwrap(), minimum);
    }

    #[test]
    fn any_nonempty_asset_list_preserves_order_without_emotion_slot_rules() {
        for assets in [
            vec!["only"],
            vec!["second", "first", "second"],
            vec!["x"; 7],
        ] {
            let role = parse_minimal_role_definition(
                &json!({"persona_prompt": "Guide", "visual_assets": assets}).to_string(),
            )
            .unwrap();
            assert_eq!(role.visual_assets, assets);
        }
    }

    #[test]
    fn errors_do_not_echo_invalid_content() {
        let errors = parse_minimal_role_definition(
            r#"{"persona_prompt":"private-persona","visual_assets":"private-asset"}"#,
        )
        .unwrap_err();
        assert!(errors.iter().all(|error| {
            !error.contains("private-persona") && !error.contains("private-asset")
        }));
        assert!(parse_minimal_role_definition("{broken").is_err());
        assert!(parse_minimal_role_definition(
            r#"{"persona_prompt":"A","persona_prompt":"B","visual_assets":["x"]}"#
        )
        .is_err());
    }
}
