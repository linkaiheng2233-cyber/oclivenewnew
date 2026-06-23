//! Kernel factory template catalog (`--list-templates` and interactive recipe selection).

use crate::init::{template_defaults, InitTemplateArg, ProjectType, RolePackKind};

pub struct TemplateCatalogEntry {
    pub id: &'static str,
    pub scene: &'static str,
    pub description: &'static str,
    pub preset: &'static str,
    pub monolith: &'static str,
    pub project_type: &'static str,
    pub default_role_pack: &'static str,
}

pub const CATALOG: &[TemplateCatalogEntry] = &[
    TemplateCatalogEntry {
        id: "robot-soul",
        scene: "Smart doll / embedded",
        description: "minimal preset + default Monolith + robot-soul-minimal role pack",
        preset: "minimal",
        monolith: "on",
        project_type: "kernel_server",
        default_role_pack: "robot-soul-minimal",
    },
    TemplateCatalogEntry {
        id: "robot-gateway",
        scene: "Smart gateway / home hub",
        description: "mixed preset + Monolith + Agent/MCP skeleton (roles/gateway + mcp_servers/)",
        preset: "mixed",
        monolith: "on",
        project_type: "kernel_server",
        default_role_pack: "gateway skeleton (not default example pack)",
    },
    TemplateCatalogEntry {
        id: "dialogue-only",
        scene: "Dialogue-only service",
        description: "full preset + default example role pack; Monolith off by default",
        preset: "full",
        monolith: "off",
        project_type: "kernel_server",
        default_role_pack: "default",
    },
    TemplateCatalogEntry {
        id: "headless-api",
        scene: "Headless HTTP API",
        description: "full preset, no example role pack; optional --monolith",
        preset: "full",
        monolith: "off",
        project_type: "kernel_server",
        default_role_pack: "none",
    },
    TemplateCatalogEntry {
        id: "library-embed",
        scene: "Library embedded in another Rust process",
        description: "minimal preset, library type; no monolith.toml generated",
        preset: "minimal",
        monolith: "off",
        project_type: "library",
        default_role_pack: "none",
    },
];

pub fn print_templates_table() {
    println!("oclive kernel factory templates (--template <id>)\n");
    println!(
        "{:<16} {:<22} {:<8} {:<8} {:<14} default role pack",
        "template", "scene", "preset", "Monolith", "project-type"
    );
    println!("{}", "-".repeat(96));
    for e in CATALOG {
        println!(
            "{:<16} {:<22} {:<8} {:<8} {:<14} {}",
            e.id, e.scene, e.preset, e.monolith, e.project_type, e.default_role_pack
        );
    }
    println!(
        "\nNote: explicit --preset / --monolith / --with-role-pack override template defaults."
    );
    println!("Vision and weld comparison: creator-docs/getting-started/KERNEL_FACTORY_VISION.md");
    for e in CATALOG {
        println!("  · {} — {}", e.id, e.description);
    }
}

pub fn template_from_id(id: &str) -> Option<InitTemplateArg> {
    match id {
        "robot-soul" => Some(InitTemplateArg::RobotSoul),
        "robot-gateway" => Some(InitTemplateArg::RobotGateway),
        "dialogue-only" => Some(InitTemplateArg::DialogueOnly),
        "headless-api" => Some(InitTemplateArg::HeadlessApi),
        "library-embed" => Some(InitTemplateArg::LibraryEmbed),
        _ => None,
    }
}

pub fn role_pack_label(kind: RolePackKind) -> &'static str {
    match kind {
        RolePackKind::None => "none",
        RolePackKind::DefaultExample => "default",
        RolePackKind::RobotSoulMinimal => "robot-soul-minimal",
    }
}

/// Build initial `ProjectConfig` baseline from a template (CLI overrides apply afterward).
pub fn project_config_from_template(name: &str, t: InitTemplateArg) -> crate::init::ProjectConfig {
    let td = template_defaults(t);
    let mut cfg = crate::init::preset_config(name, td.preset);
    cfg.project_type = td.project_type;
    cfg.monolith_enabled = td.monolith_default && td.project_type == ProjectType::KernelServer;
    cfg.role_pack_kind = td.role_pack;
    cfg.factory_template = Some(t);
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_five_templates() {
        assert_eq!(CATALOG.len(), 5);
        assert!(template_from_id("robot-gateway").is_some());
        assert!(template_from_id("unknown").is_none());
    }
}
