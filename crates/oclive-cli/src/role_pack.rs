//! 示例角色包生成（`--with-role-pack` / 模板默认）。

use crate::blueprint_v3_init::build_blueprint_v3_value;
use crate::generator::render_settings_json;
use crate::init::{ProjectConfig, RolePackKind, ChatStorageBackend};
use anyhow::{Context, Result};
use oclive_validation::PIPELINE_BLUEPRINT_FILENAME;
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

fn chat_storage_backend_token(b: ChatStorageBackend) -> &'static str {
    // Tokens must match ChatStorageBackendKind serde (`hybrid`/`file`/`sqlite`).
    match b {
        ChatStorageBackend::Hybrid => "hybrid",
        ChatStorageBackend::File => "file",
        ChatStorageBackend::Sqlite => "sqlite",
    }
}

fn write_role_config_json(cfg: &ProjectConfig, role_root: &Path) -> Result<()> {
    let mut chat_storage = serde_json::json!({
        "backend": chat_storage_backend_token(cfg.chat_storage_backend),
        "max_messages_per_session": 500,
        "replay_similarity_threshold": 0.6
    });
    if cfg.chat_storage_location == "role_pack" {
        chat_storage["location"] = serde_json::json!("role_pack");
    }
    let config = serde_json::json!({
        "chat_storage": chat_storage
    });
    fs::write(
        role_root.join("config.json"),
        serde_json::to_string_pretty(&config).context("config.json")?,
    )
    .context("write config.json")?;
    Ok(())
}

fn write_default_example(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    let role_root = out.join("roles").join("default");
    fs::create_dir_all(role_root.join("scenes").join("default"))
        .context("create roles/default/scenes/default")?;
    if cfg.dual_core_enabled {
        return write_dual_core_v3_role(cfg, &role_root, "default", "Example role (dual-core)");
    }
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
    write_role_config_json(cfg, &role_root)?;
    Ok(())
}

fn write_robot_soul_minimal(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    let role_root = out.join("roles").join("default");
    fs::create_dir_all(role_root.join("prompts")).context("create prompts")?;
    fs::create_dir_all(role_root.join("scenes").join("default"))
        .context("create scenes/default")?;
    if cfg.dual_core_enabled {
        write_dual_core_v3_role(
            cfg,
            &role_root,
            "default",
            "Robot Soul Minimal (dual-core)",
        )?;
        fs::write(
            role_root.join("prompts").join("system.md"),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/role_packs/robot_soul_system.md"
            )),
        )
        .context("write prompts/system.md")?;
        return Ok(());
    }

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
    write_role_config_json(cfg, &role_root)?;
    Ok(())
}

fn write_dual_core_v3_role(
    cfg: &ProjectConfig,
    role_root: &Path,
    role_id: &str,
    name: &str,
) -> Result<()> {
    let bp = build_blueprint_v3_value(cfg, role_id, name);
    fs::write(
        role_root.join(PIPELINE_BLUEPRINT_FILENAME),
        serde_json::to_string_pretty(&bp).context("serialize v3 blueprint")?,
    )
    .context("write pipeline.ocblueprint")?;
    fs::write(
        role_root.join("character.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/character.md"
        )),
    )
    .context("write character.md")?;
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
    write_role_config_json(cfg, role_root)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::build_settings_value;
    use crate::init::{preset_config, RolePackKind};
    use tempfile::tempdir;

    #[test]
    fn role_pack_writes_chat_storage_config() {
        let mut cfg = preset_config("t", "minimal");
        cfg.chat_storage_backend = ChatStorageBackend::File;
        let dir = tempdir().unwrap();
        write_default_example(&cfg, dir.path()).unwrap();
        let raw = fs::read_to_string(dir.path().join("roles/default/config.json")).unwrap();
        assert!(raw.contains("\"backend\": \"file\""));
    }

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
    fn dual_core_writes_v3_blueprint_without_manifest() {
        let mut cfg = preset_config("t", "full");
        cfg.dual_core_enabled = true;
        let dir = tempdir().unwrap();
        write_default_example(&cfg, dir.path()).unwrap();
        let bp = dir.path().join("roles/default/pipeline.ocblueprint");
        assert!(bp.is_file());
        assert!(!dir.path().join("roles/default/manifest.json").exists());
        let raw = fs::read_to_string(bp).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["schema_version"], 3);
        assert_eq!(v["runtime_config"]["dual_core"]["enabled"], true);
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
