//! 合成角色包夹具（补充约束 §2/§3）：单角色、无 blueprint、无目录插件、无画像目录。
//! 逐键来源见 `.cursor/plans/chatpro-verification-a-execution-draft.plan.md` §3 与 A-P2 修订 §0。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{MODEL_ID, ROLE_ID, ROLE_NAME};
use std::path::Path;

/// legacy six-slot manifest（`DiskRoleManifest` 为 `deny_unknown_fields`，只给必要键）。
pub const MANIFEST_JSON: &str = r#"{
  "id": "a-probe-role",
  "name": "A Probe Role",
  "version": "0.0.1",
  "author": "harness",
  "description": "candidate A synthetic role (non-production, isolated)",
  "ollama_model": "a-probe-model:latest",
  "default_personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
  "evolution": { "personality_source": "vector" },
  "scenes": [],
  "user_relations": { "friend": { "prompt_hint": "harness" } },
  "default_relation": "friend",
  "dev_only": false
}
"#;

/// 引擎侧 settings（`DiskRoleSettings`）：六槽全内建；Agent 关闭。
pub const SETTINGS_JSON: &str = r#"{
  "schema_version": 1,
  "plugin_backends": {
    "llm": "ollama",
    "agent": "none",
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin"
  }
}
"#;

/// 角色包 config：JSON mirror 的第二重记录（运行期有效开关是 env，见 env.rs）。
pub const CONFIG_JSON: &str = r#"{
  "chat_storage": { "mirror": false }
}
"#;

pub fn role_dir(scenario_root: &Path) -> std::path::PathBuf {
    scenario_root.join("roles").join(ROLE_ID)
}

/// # Errors 目录不存在或写入失败（调用方须先经路径先验确认 roles 目录）。
pub fn write_role_pack(scenario_root: &Path) -> Result<(), String> {
    let dir = role_dir(scenario_root);
    if !dir.is_dir() {
        return Err(format!("夹具目录不存在（须先建后写）：{}", dir.display()));
    }
    for (name, body) in [
        ("manifest.json", MANIFEST_JSON),
        ("settings.json", SETTINGS_JSON),
        ("config.json", CONFIG_JSON),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, body).map_err(|e| format!("写 {} 失败: {e}", path.display()))?;
    }
    Ok(())
}

/// 通过**公开加载链**装载夹具并断言有效配置（不使用私有字段、不引用被测格式化函数）。
/// 返回 `Err` 而不是 panic，便于子进程把闸门失败转成报告错误而不继续 build。
pub fn assert_role_pack_loads(scenario_root: &Path) -> Result<serde_json::Value, String> {
    let roles_dir = scenario_root.join("roles");
    let storage = oclive_kernel_host::infrastructure::RoleStorage::new(&roles_dir);
    let role = storage
        .load_role(ROLE_ID)
        .map_err(|e| format!("夹具无法经 RoleStorage::load_role 装载: {e}"))?;

    let mut problems: Vec<String> = Vec::new();
    if role.id != ROLE_ID {
        problems.push(format!("role.id 期望 {ROLE_ID}，实际 {}", role.id));
    }
    if role.name != ROLE_NAME {
        problems.push(format!("role.name 期望 {ROLE_NAME}，实际 {}", role.name));
    }
    if role.ollama_model.as_deref() != Some(MODEL_ID) {
        problems.push(format!(
            "role.ollama_model 期望 {MODEL_ID}，实际 {:?}",
            role.ollama_model
        ));
    }
    if role.pack_chat_storage_config.mirror != Some(false) {
        problems.push(format!(
            "chat_storage.mirror 期望 Some(false)（第二重记录），实际 {:?}",
            role.pack_chat_storage_config.mirror
        ));
    }
    if role.pack_chat_storage_config.auto_cleanup_days.is_some()
        || role
            .pack_chat_storage_config
            .auto_cleanup_max_sessions
            .is_some()
    {
        problems.push("auto_cleanup 两项必须未设（cleanup.rs:45-46 ⇒ is_enabled()=false）".into());
    }
    if format!("{:?}", role.plugin_backends.llm) != "Ollama" {
        problems.push(format!(
            "llm backend 期望 Ollama（注入替身所在），实际 {:?}",
            role.plugin_backends.llm
        ));
    }
    if format!("{:?}", role.plugin_backends.agent) != "None" {
        problems.push(format!(
            "agent backend 期望 None，实际 {:?}",
            role.plugin_backends.agent
        ));
    }
    if !format!("{:?}", role.evolution_config.personality_source).contains("Vector") {
        problems.push("personality_source 期望 Vector".into());
    }
    if role.scene_ids.len() > 1 {
        problems.push(format!(
            "scene_ids 期望 ≤1（scene.rs:119-121 早退门控），实际 {}",
            role.scene_ids.len()
        ));
    }
    if !problems.is_empty() {
        return Err(format!("夹具配置断言失败：{}", problems.join(" | ")));
    }

    Ok(serde_json::json!({
        "role_id": role.id,
        "role_name": role.name,
        "ollama_model": role.ollama_model,
        "scene_ids": role.scene_ids.iter().cloned().collect::<Vec<_>>(),
        "chat_storage_mirror": role.pack_chat_storage_config.mirror,
        "chat_storage_auto_cleanup_days": role.pack_chat_storage_config.auto_cleanup_days,
        "chat_storage_auto_cleanup_max_sessions": role.pack_chat_storage_config.auto_cleanup_max_sessions,
        "plugin_backends_debug": format!("{:?}", role.plugin_backends),
        "personality_source_debug": format!("{:?}", role.evolution_config.personality_source),
        "mirror_control_note": "运行期有效开关为 env OCLIVE_CHAT_STORAGE_BACKEND（见 env.rs）；config.json 的 mirror 仅作第二重记录",
    }))
}
