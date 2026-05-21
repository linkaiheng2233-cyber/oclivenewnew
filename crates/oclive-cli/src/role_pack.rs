//! 示例角色包生成（`--with-role-pack` / 模板默认）。

use crate::generator::render_settings_json;
use crate::init::{ProjectConfig, RolePackKind};
use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn write_role_pack(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    match cfg.role_pack_kind {
        RolePackKind::None => Ok(()),
        RolePackKind::DefaultExample => write_default_example(cfg, out),
        RolePackKind::RobotSoulMinimal => write_robot_soul_minimal(cfg, out),
    }
}

fn write_default_example(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    let role_root = out.join("roles").join("default");
    fs::create_dir_all(role_root.join("scenes").join("default"))
        .context("create roles/default/scenes/default")?;
    let settings = render_settings_json(cfg).context("settings.json")?;
    fs::write(role_root.join("settings.json"), settings).context("write settings.json")?;
    fs::write(
        role_root.join("character.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/character.md"
        )),
    )
    .context("write character.md")?;
    let manifest = json!({
        "id": "default",
        "name": "Example role",
        "version": "0.1.0",
        "author": "oclive-cli",
        "description": "Scaffold example; replace with your role pack.",
        "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        "scenes": ["default"],
        "user_relations": {
            "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
        },
        "default_relation": "friend"
    });
    fs::write(
        role_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("manifest.json")?,
    )
    .context("write manifest.json")?;
    fs::write(
        role_root.join("core_personality.txt"),
        "# Core personality text (optional).\n",
    )
    .context("write core_personality.txt")?;
    let scene = json!({
        "name": "Default",
        "time_windows": [],
        "keywords": [],
        "events": []
    });
    fs::write(
        role_root.join("scenes").join("default").join("scene.json"),
        serde_json::to_string_pretty(&scene).context("scene.json")?,
    )
    .context("write scene.json")?;
    Ok(())
}

fn write_robot_soul_minimal(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    let role_root = out.join("roles").join("default");
    fs::create_dir_all(role_root.join("prompts")).context("create prompts")?;
    fs::create_dir_all(role_root.join("scenes").join("default"))
        .context("create scenes/default")?;

    let settings = render_settings_json(cfg).context("settings.json")?;
    fs::write(role_root.join("settings.json"), settings).context("write settings.json")?;

    let manifest = json!({
        "id": "default",
        "name": "Robot Soul Minimal",
        "version": "0.1.0",
        "author": "oclive-cli",
        "description": "Minimal soul pack for smart toy / embedded (kernel factory template).",
        "min_runtime_version": "0.2.0",
        "default_personality": [0.45, 0.35, 0.4, 0.25, 0.3, 0.2, 0.55],
        "scenes": ["default"],
        "user_relations": {
            "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
        },
        "default_relation": "friend"
    });
    fs::write(
        role_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("manifest.json")?,
    )
    .context("write manifest.json")?;

    fs::write(
        role_root.join("prompts").join("system.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/role_packs/robot_soul_system.md"
        )),
    )
    .context("write prompts/system.md")?;

    fs::write(
        role_root.join("core_personality.txt"),
        "核心人设（示例）：冷静、可靠、简短回应；面向设备侧可调参，不依赖桌面 UI。\n",
    )
    .context("write core_personality.txt")?;

    let scene = json!({
        "name": "Default",
        "time_windows": [],
        "keywords": [],
        "events": []
    });
    fs::write(
        role_root.join("scenes").join("default").join("scene.json"),
        serde_json::to_string_pretty(&scene).context("scene.json")?,
    )
    .context("write scene.json")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::build_settings_value;
    use crate::init::{preset_config, RolePackKind};
    use tempfile::tempdir;

    #[test]
    fn robot_soul_pack_has_system_prompt() {
        let mut cfg = preset_config("t", "minimal");
        cfg.role_pack_kind = RolePackKind::RobotSoulMinimal;
        let dir = tempdir().unwrap();
        write_robot_soul_minimal(&cfg, dir.path()).unwrap();
        let p = dir.path().join("roles/default/prompts/system.md");
        assert!(p.is_file());
        let m = fs::read_to_string(dir.path().join("roles/default/manifest.json")).unwrap();
        assert!(m.contains("default_personality"));
    }

    #[test]
    fn template_robot_soul_settings_match_minimal_matrix() {
        let cfg = preset_config("t", "minimal");
        let v = build_settings_value(&cfg);
        let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
        assert!(!pb.contains_key("agent"));
        assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "ollama");
    }
}
