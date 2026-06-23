//! Rule-based `visual_state_id` resolution from catalog tags (Phase 1).

#[cfg(test)]
use oclive_kernel_types::models::SIMPLE_PORTRAIT_SLOT_IDS;
use oclive_kernel_types::models::{PortraitCatalogFile, Role};

#[must_use]
pub fn portrait_catalog_active(role: &Role) -> bool {
    role.pack_portrait_catalog.enabled && role.portrait_catalog.is_some()
}

#[must_use]
pub fn resolve_visual_state_rule(
    catalog: &PortraitCatalogFile,
    emotion_tag: &str,
) -> Option<String> {
    let tag = emotion_tag.trim().to_ascii_lowercase();
    if tag.is_empty() {
        return default_neutral_id(catalog);
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
    fn validate_known_id() {
        let catalog = sample_catalog();
        assert_eq!(
            validate_asset_id(&catalog, "angry_default"),
            Some("angry_default".to_string())
        );
        assert_eq!(validate_asset_id(&catalog, "unknown"), None);
    }
}
