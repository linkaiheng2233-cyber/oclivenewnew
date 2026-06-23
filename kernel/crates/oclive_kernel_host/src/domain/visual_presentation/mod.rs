//! Visual presentation facility (#4): `visual_state_id` → `performance_directive` (no LLM).

use oclive_kernel_types::models::{
    PerformanceDirective, PortraitAssetKind, Role, RolePackVisualPresentationConfig,
    VisualPresentationBackendKind,
};

#[must_use]
pub fn visual_presentation_active(role: &Role) -> bool {
    role.pack_visual_presentation_config.enabled && role.portrait_catalog.is_some()
}

#[must_use]
pub fn materialize_directive(role: &Role, visual_state_id: &str) -> Option<PerformanceDirective> {
    if !visual_presentation_active(role) {
        return None;
    }
    let catalog = role.portrait_catalog.as_ref()?;
    let asset = catalog.assets.iter().find(|a| a.id == visual_state_id)?;
    let backend = role.pack_visual_presentation_config.backend;

    match asset.kind {
        PortraitAssetKind::Image => Some(image_directive(visual_state_id, &asset.path)),
        PortraitAssetKind::Live2d
            if matches!(
                backend,
                VisualPresentationBackendKind::Live2d | VisualPresentationBackendKind::Directory
            ) =>
        {
            Some(live2d_directive(
                visual_state_id,
                &asset.path,
                asset
                    .resources
                    .as_ref()
                    .and_then(|r| r.live2d_model.as_deref()),
                asset.context.as_deref(),
            ))
        }
        PortraitAssetKind::Rig3d
            if matches!(
                backend,
                VisualPresentationBackendKind::Rig3d | VisualPresentationBackendKind::Directory
            ) =>
        {
            Some(rig3d_directive(
                visual_state_id,
                &asset.path,
                asset
                    .resources
                    .as_ref()
                    .and_then(|r| r.rig3d_model.as_deref()),
                asset.context.as_deref(),
            ))
        }
        PortraitAssetKind::Procedural
            if matches!(
                backend,
                VisualPresentationBackendKind::Procedural
                    | VisualPresentationBackendKind::Directory
            ) =>
        {
            Some(procedural_directive(
                visual_state_id,
                &asset.path,
                asset.context.as_deref(),
            ))
        }
        _ => Some(image_directive(visual_state_id, &asset.path)),
    }
}

fn image_directive(visual_state_id: &str, path: &str) -> PerformanceDirective {
    PerformanceDirective {
        visual_state_id: visual_state_id.to_string(),
        kind: "image".to_string(),
        path: Some(path.to_string()),
        expression: None,
        motion: None,
        fallback_image: Some(path.to_string()),
        live2d_model: None,
        rig3d_model: None,
        context: None,
    }
}

fn live2d_directive(
    visual_state_id: &str,
    fallback: &str,
    model: Option<&str>,
    context: Option<&str>,
) -> PerformanceDirective {
    PerformanceDirective {
        visual_state_id: visual_state_id.to_string(),
        kind: "live2d".to_string(),
        path: Some(fallback.to_string()),
        expression: None,
        motion: Some("idle".to_string()),
        fallback_image: Some(fallback.to_string()),
        live2d_model: model.map(str::to_string),
        rig3d_model: None,
        context: context.map(str::to_string),
    }
}

fn rig3d_directive(
    visual_state_id: &str,
    fallback: &str,
    model: Option<&str>,
    context: Option<&str>,
) -> PerformanceDirective {
    PerformanceDirective {
        visual_state_id: visual_state_id.to_string(),
        kind: "rig3d".to_string(),
        path: Some(fallback.to_string()),
        expression: None,
        motion: Some("idle".to_string()),
        fallback_image: Some(fallback.to_string()),
        live2d_model: None,
        rig3d_model: model.map(str::to_string),
        context: context.map(str::to_string),
    }
}

fn procedural_directive(
    visual_state_id: &str,
    path: &str,
    context: Option<&str>,
) -> PerformanceDirective {
    PerformanceDirective {
        visual_state_id: visual_state_id.to_string(),
        kind: "procedural".to_string(),
        path: Some(path.to_string()),
        expression: None,
        motion: None,
        fallback_image: Some(path.to_string()),
        live2d_model: None,
        rig3d_model: None,
        context: context.map(str::to_string),
    }
}

/// Distro profile gating: `off` suppresses directives even when pack enables visual_presentation.
#[must_use]
pub fn distro_allows_visual_presentation(mode: &str) -> bool {
    !matches!(mode.trim().to_ascii_lowercase().as_str(), "" | "off")
}

/// Apply distro `image_only` / `stage_full` policy to a materialized directive.
#[must_use]
pub fn filter_directive_by_distro_mode(
    distro_mode: Option<&str>,
    directive: PerformanceDirective,
) -> Option<PerformanceDirective> {
    let mode = distro_mode?.trim().to_ascii_lowercase();
    if mode.is_empty() || mode == "off" {
        return None;
    }
    if mode == "image_only" && directive.kind != "image" {
        return None;
    }
    Some(directive)
}

/// Materialize + apply pack `visual_presentation.enabled` and distro mode gating.
#[must_use]
pub fn materialize_directive_gated(
    role: &Role,
    visual_state_id: &str,
    distro_mode: Option<&str>,
) -> Option<PerformanceDirective> {
    if !effective_visual_presentation_enabled(&role.pack_visual_presentation_config, distro_mode) {
        return None;
    }
    let directive = materialize_directive(role, visual_state_id)?;
    filter_directive_by_distro_mode(distro_mode, directive)
}

#[must_use]
pub fn effective_visual_presentation_enabled(
    pack: &RolePackVisualPresentationConfig,
    distro_mode: Option<&str>,
) -> bool {
    if !pack.enabled {
        return false;
    }
    match distro_mode {
        Some(mode) => distro_allows_visual_presentation(mode),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::models::{
        PortraitCatalogAsset, PortraitCatalogFile, PortraitCatalogToggle,
    };

    fn role_with_catalog(enabled_vp: bool) -> Role {
        Role {
            id: "test".to_string(),
            name: "Test".to_string(),
            pack_portrait_catalog: PortraitCatalogToggle { enabled: true },
            portrait_catalog: Some(PortraitCatalogFile {
                schema_version: 1,
                assets: vec![PortraitCatalogAsset {
                    id: "happy_default".to_string(),
                    path: "assets/images/happy.webp".to_string(),
                    desc: "开心".to_string(),
                    tags: vec!["happy".to_string()],
                    kind: PortraitAssetKind::Image,
                    cluster: None,
                    context: None,
                    resources: None,
                }],
            }),
            pack_visual_presentation_config: RolePackVisualPresentationConfig {
                enabled: enabled_vp,
                backend: VisualPresentationBackendKind::Image,
            },
            ..Default::default()
        }
    }

    #[test]
    fn materialize_when_enabled() {
        let role = role_with_catalog(true);
        let d = materialize_directive(&role, "happy_default").expect("directive");
        assert_eq!(d.kind, "image");
        assert_eq!(d.path.as_deref(), Some("assets/images/happy.webp"));
    }

    #[test]
    fn no_directive_when_disabled() {
        let role = role_with_catalog(false);
        assert!(materialize_directive(&role, "happy_default").is_none());
    }

    #[test]
    fn distro_off_suppresses_gated_directive() {
        let role = role_with_catalog(true);
        assert!(materialize_directive_gated(&role, "happy_default", Some("off")).is_none());
    }

    #[test]
    fn image_only_blocks_live2d_kind() {
        let mut role = role_with_catalog(true);
        role.portrait_catalog.as_mut().unwrap().assets[0].kind = PortraitAssetKind::Live2d;
        role.pack_visual_presentation_config.backend = VisualPresentationBackendKind::Live2d;
        let d = materialize_directive(&role, "happy_default").expect("live2d directive");
        assert_eq!(d.kind, "live2d");
        assert!(filter_directive_by_distro_mode(Some("image_only"), d).is_none());
    }
}
