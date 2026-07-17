//! Rule-based `visual_state_id` resolution from catalog tags (Phase 1).

#[cfg(test)]
use oclive_kernel_types::models::SIMPLE_PORTRAIT_SLOT_IDS;
use oclive_kernel_types::models::{PortraitCatalogFile, Role};

#[must_use]
pub fn portrait_catalog_active(role: &Role) -> bool {
    role.pack_portrait_catalog.enabled && role.portrait_catalog.is_some()
}

/// Startup-resolved gate for the enhanced portrait route.
///
/// A catalog by itself is not enough: when visual presentation is disabled,
/// running the catalog director would only add work without producing a
/// directive. Such packs stay on the legacy filename fallback path.
#[must_use]
pub fn enhanced_portrait_active(role: &Role, distro_mode: Option<&str>) -> bool {
    portrait_catalog_active(role)
        && crate::domain::visual_presentation::effective_visual_presentation_enabled(
            &role.pack_visual_presentation_config,
            distro_mode,
        )
}

#[must_use]
pub fn resolve_visual_state_rule(
    catalog: &PortraitCatalogFile,
    emotion_tag: &str,
) -> Option<String> {
    resolve_visual_state_rule_with_intensity(catalog, emotion_tag, None)
}

/// Resolve an emotion tag to the closest catalog intensity variant.
/// Packs without intensity variants retain the legacy/default behavior.
#[must_use]
pub fn resolve_visual_state_rule_with_intensity(
    catalog: &PortraitCatalogFile,
    emotion_tag: &str,
    intensity: Option<f64>,
) -> Option<String> {
    let tag = emotion_tag.trim().to_ascii_lowercase();
    if tag.is_empty() {
        return default_neutral_id(catalog);
    }
    if let Some(level) = intensity.and_then(intensity_level) {
        let id = format!("{tag}_{level}");
        if catalog.assets.iter().any(|asset| asset.id == id) {
            return Some(id);
        }
    }
    for asset in &catalog.assets {
        if asset
            .tags
            .iter()
            .any(|t| t.trim().eq_ignore_ascii_case(&tag))
        {
            return Some(asset.id.clone());
        }
    }
    default_neutral_id(catalog)
}

fn intensity_level(intensity: f64) -> Option<&'static str> {
    if !intensity.is_finite() {
        return None;
    }
    Some(if intensity < 0.34 {
        "mild"
    } else if intensity < 0.67 {
        "moderate"
    } else {
        "severe"
    })
}

fn default_neutral_id(catalog: &PortraitCatalogFile) -> Option<String> {
    catalog
        .assets
        .iter()
        .find(|a| {
            a.id == "neutral_default" || a.tags.iter().any(|t| t.eq_ignore_ascii_case("neutral"))
        })
        .map(|a| a.id.clone())
        .or_else(|| catalog.assets.first().map(|a| a.id.clone()))
}

#[must_use]
pub fn asset_path_by_id<'a>(catalog: &'a PortraitCatalogFile, id: &str) -> Option<&'a str> {
    catalog
        .assets
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.path.as_str())
}

#[must_use]
pub fn validate_asset_id(catalog: &PortraitCatalogFile, id: &str) -> Option<String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return None;
    }
    if catalog.assets.iter().any(|a| a.id == trimmed) {
        return Some(trimmed.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::models::PortraitCatalogAsset;

    fn sample_catalog() -> PortraitCatalogFile {
        PortraitCatalogFile {
            schema_version: 1,
            assets: SIMPLE_PORTRAIT_SLOT_IDS
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
                .collect(),
        }
    }

    #[test]
    fn resolve_by_tag() {
        let catalog = sample_catalog();
        assert_eq!(
            resolve_visual_state_rule(&catalog, "happy"),
            Some("happy_default".to_string())
        );
        assert_eq!(
            resolve_visual_state_rule(&catalog, "SHY"),
            Some("shy_default".to_string())
        );
    }

    #[test]
    fn resolve_empty_falls_back_neutral() {
        let catalog = sample_catalog();
        assert_eq!(
            resolve_visual_state_rule(&catalog, ""),
            Some("neutral_default".to_string())
        );
    }

    #[test]
    fn resolve_by_intensity_prefers_variant_and_falls_back() {
        let mut catalog = sample_catalog();
        catalog.assets.push(PortraitCatalogAsset {
            id: "happy_mild".to_string(),
            path: "assets/images/happy_mild.png".to_string(),
            desc: "mild".to_string(),
            tags: vec!["happy".to_string()],
            kind: Default::default(),
            cluster: None,
            context: None,
            resources: None,
        });
        assert_eq!(
            resolve_visual_state_rule_with_intensity(&catalog, "happy", Some(0.2)),
            Some("happy_mild".to_string())
        );
        assert_eq!(
            resolve_visual_state_rule_with_intensity(&catalog, "sad", Some(0.9)),
            Some("sad_default".to_string())
        );
    }

    #[test]
    fn validate_known_id() {
        let catalog = sample_catalog();
        assert_eq!(
            validate_asset_id(&catalog, "angry_default"),
            Some("angry_default".to_string())
        );
        assert_eq!(validate_asset_id(&catalog, "unknown"), None);
    }
}
