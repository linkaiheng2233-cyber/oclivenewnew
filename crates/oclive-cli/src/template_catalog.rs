//! 内核工厂模板目录（`--list-templates` 与交互式配方选择）。

use crate::init::{InitTemplateArg, RolePackKind, ProjectType, template_defaults};

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
        scene: "智能玩偶 / 嵌入式",
        description: "minimal 预设 + 默认 Monolith + robot-soul-minimal 角色包",
        preset: "minimal",
        monolith: "启用",
        project_type: "kernel_server",
        default_role_pack: "robot-soul-minimal",
    },
    TemplateCatalogEntry {
        id: "robot-gateway",
        scene: "智能网关 / 家庭中枢",
        description: "mixed 预设 + Monolith + Agent/MCP 骨架（roles/gateway + mcp_servers/）",
        preset: "mixed",
        monolith: "启用",
        project_type: "kernel_server",
        default_role_pack: "gateway 骨架（非 default 示例包）",
    },
    TemplateCatalogEntry {
        id: "dialogue-only",
        scene: "纯对话服务",
        description: "full 预设 + 通用 default 角色包，Monolith 默认关闭",
        preset: "full",
        monolith: "关闭",
        project_type: "kernel_server",
        default_role_pack: "default",
    },
    TemplateCatalogEntry {
        id: "headless-api",
        scene: "纯 HTTP API",
        description: "full 预设、无示例角色包，可按需 --monolith",
        preset: "full",
        monolith: "关闭",
        project_type: "kernel_server",
        default_role_pack: "无",
    },
    TemplateCatalogEntry {
        id: "library-embed",
        scene: "库嵌入其它 Rust 进程",
        description: "minimal 预设、library 类型，不生成 monolith.toml",
        preset: "minimal",
        monolith: "关闭",
        project_type: "library",
        default_role_pack: "无",
    },
];

pub fn print_templates_table() {
    println!("oclive 内核工厂模板（--template <id>）\n");
    println!(
        "{:<16} {:<22} {:<8} {:<8} {:<14} {}",
        "template", "场景", "preset", "Monolith", "project-type", "默认角色包"
    );
    println!("{}", "-".repeat(96));
    for e in CATALOG {
        println!(
            "{:<16} {:<22} {:<8} {:<8} {:<14} {}",
            e.id, e.scene, e.preset, e.monolith, e.project_type, e.default_role_pack
        );
    }
    println!("\n说明：显式 --preset / --monolith / --with-role-pack 等参数优先于模板默认值。");
    println!("愿景与焊接对比：creator-docs/getting-started/KERNEL_FACTORY_VISION.md");
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
        RolePackKind::None => "无",
        RolePackKind::DefaultExample => "default",
        RolePackKind::RobotSoulMinimal => "robot-soul-minimal",
    }
}

/// 由模板构建初始 `ProjectConfig` 基线（不含 project_name 以外的 CLI 覆盖）。
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
