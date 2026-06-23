//! Distro `visual_presentation.mode` gating on performance_directive emission.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::domain::host_profile::load_host_profile_file;
use oclive_kernel_host::domain::visual_presentation::{
    filter_directive_by_distro_mode, materialize_directive_gated,
};
use oclive_kernel_types::models::{
    PortraitAssetKind, PortraitCatalogAsset, PortraitCatalogFile, PortraitCatalogToggle, Role,
    RolePackVisualPresentationConfig, VisualPresentationBackendKind,
};

fn catalog_role() -> Role {
    Role {
        id: "vp-gate".to_string(),
        name: "VP Gate".to_string(),
        pack_portrait_catalog: PortraitCatalogToggle { enabled: true },
        portrait_catalog: Some(PortraitCatalogFile {
            schema_version: 1,
            assets: vec![PortraitCatalogAsset {
                id: "happy_default".to_string(),
                path: "assets/images/happy.webp".to_string(),
                desc: "happy".to_string(),
                tags: vec!["happy".to_string()],
                kind: PortraitAssetKind::Image,
                cluster: None,
                context: None,
                resources: None,
            }],
        }),
        pack_visual_presentation_config: RolePackVisualPresentationConfig {
            enabled: true,
            backend: VisualPresentationBackendKind::Image,
        },
        ..Default::default()
    }
}

#[test]
fn host_profile_parses_visual_presentation_mode() {
    let dir = std::env::temp_dir().join(format!("oclive_vp_gate_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("theater.oclive.toml");
    std::fs::write(
        &path,
        r#"distro_id = "theater"
[visual_presentation]
mode = "stage_full"
"#,
    )
    .unwrap();
    let profile = load_host_profile_file(&path).expect("profile");
    assert_eq!(
        profile.visual_presentation_mode.as_deref(),
        Some("stage_full")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn distro_off_suppresses_directive() {
    let role = catalog_role();
    assert!(materialize_directive_gated(&role, "happy_default", Some("off")).is_none());
}

#[test]
fn image_only_allows_image_kind() {
    let role = catalog_role();
    let d = materialize_directive_gated(&role, "happy_default", Some("image_only"))
        .expect("image directive");
    assert_eq!(d.kind, "image");
}

#[test]
fn image_only_blocks_non_image_after_materialize() {
    let mut d = oclive_kernel_types::models::PerformanceDirective {
        visual_state_id: "x".into(),
        kind: "live2d".into(),
        path: Some("assets/images/happy.webp".into()),
        expression: None,
        motion: None,
        fallback_image: None,
        live2d_model: None,
        rig3d_model: None,
        context: None,
    };
    assert!(filter_directive_by_distro_mode(Some("image_only"), d.clone()).is_none());
    d.kind = "image".into();
    assert!(filter_directive_by_distro_mode(Some("image_only"), d).is_some());
}
