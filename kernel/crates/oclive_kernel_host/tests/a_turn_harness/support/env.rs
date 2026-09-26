//! 子进程环境构造（补充约束 §2）：`env_clear` 后只设白名单；可写用户/临时路径重定向到本次树。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{ENV_CHILD, ENV_ROOT, ENV_RUN_ID, ENV_RUN_TOKEN, ENV_SCENARIO};
use std::path::Path;

/// 继承的**必要系统值**（不含任何密钥、不含 OCLIVE 配置）。
#[cfg(windows)]
const INHERIT_SYSTEM: &[&str] = &[
    "SystemRoot",
    "windir",
    "SystemDrive",
    "PATHEXT",
    "NUMBER_OF_PROCESSORS",
];
#[cfg(not(windows))]
const INHERIT_SYSTEM: &[&str] = &[];

/// 本 harness 显式设置的 OCLIVE_* 行为键（值全部由测试给出，不继承）。
pub const BEHAVIOR_KEYS: &[&str] = &[
    "OCLIVE_CHAT_STORAGE_ROOT",
    "OCLIVE_CHAT_STORAGE_BACKEND",
    "OCLIVE_OLLAMA_PRELOAD",
    "OCLIVE_EVENT_IMPACT_LLM",
    "OCLIVE_SKIP_LLM_STARTUP_PROBE",
    "OCLIVE_PORTRAIT_EMOTION_LLM",
];

/// 明确**不设置**的键（用于自审：不得继承、不得由测试补回）。
pub const DENIED_KEYS: &[&str] = &[
    "OCLIVE_LLM_BACKEND",
    "OCLIVE_MIGRATIONS_DIR",
    "OCLIVE_DISTRO_PROFILE",
    "OCLIVE_DISTRO_ID",
    "OCLIVE_THEATER_DIRECTOR_PLUGIN",
    "OCLIVE_SKIP_STARTUP_HEALTH",
    "OCLIVE_REMOTE_LLM_URL",
    "OCLIVE_REMOTE_LLM_TOKEN",
    "OCLIVE_REMOTE_FALLBACK_TO_BUILTIN",
    "OCLIVE_LLM_CLOUD_API_STYLE",
    "OCLIVE_LOCAL_LLM_MODEL_PATH",
    "OCLIVE_LOCAL_LLM_LORA_PATH",
    "OCLIVE_APP_DATA",
    "OCLIVE_LOG_DIR",
    "OLLAMA_BASE_URL",
    "OLLAMA_MODEL",
    "OCLIVE_OLLAMA_KEEP_ALIVE",
];

/// 构造子进程环境（返回键值对；调用方须 `env_clear()` 后逐项 `env()`）。
#[must_use]
pub fn build_child_env(
    scenario_root: &Path,
    scenario: &str,
    run_id: &str,
    run_token: &str,
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();

    // 1) 必要系统值（只继承值，不继承任何 OCLIVE/密钥键）。
    for key in INHERIT_SYSTEM {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                out.push(((*key).to_string(), v));
            }
        }
    }
    // 2) 只给系统目录的装载 PATH（不继承用户 PATH；本片不启动系统命令或 shell）。
    #[cfg(windows)]
    let system_path = {
        let system_root = out
            .iter()
            .find(|(k, _)| k == "SystemRoot")
            .map(|(_, v)| v.clone())
            .expect("[A-HARNESS] 父进程缺少 SystemRoot，无法构造安全的子环境");
        format!("{system_root}\\System32;{system_root}")
    };
    #[cfg(not(windows))]
    let system_path = "/usr/bin:/bin".to_string();
    out.push(("PATH".to_string(), system_path));

    // 3) 可写用户/临时路径重定向到本次树（不原值照搬）。
    let tmp = scenario_root.join("tmp");
    for (key, sub) in [
        ("TEMP", "temp"),
        ("TMP", "temp"),
        ("APPDATA", "appdata"),
        ("LOCALAPPDATA", "localappdata"),
        ("USERPROFILE", "profile"),
        ("ProgramData", "programdata"),
    ] {
        out.push((key.to_string(), tmp.join(sub).display().to_string()));
    }

    // 4) 诊断（测试自有值，非继承）。
    out.push(("RUST_BACKTRACE".to_string(), "1".to_string()));
    out.push(("RUST_LOG".to_string(), "info,oclive_chat=debug".to_string()));

    // 5) 控制项（父→子的身份/授权）。
    out.push((ENV_CHILD.to_string(), "1".to_string()));
    out.push((ENV_SCENARIO.to_string(), scenario.to_string()));
    out.push((ENV_RUN_ID.to_string(), run_id.to_string()));
    out.push((ENV_RUN_TOKEN.to_string(), run_token.to_string()));
    out.push((ENV_ROOT.to_string(), scenario_root.display().to_string()));

    // 6) 行为项（本片冻结的配置来源）。
    out.push((
        "OCLIVE_CHAT_STORAGE_ROOT".to_string(),
        scenario_root.join("chats").display().to_string(),
    ));
    // JSON mirror 的**有效**开关（首轮实测：仅关角色 config.json 的 mirror 无效）：
    // build 期只按 env 选 backend kind（`app_state_builder.rs:361-371` 传
    // `RolePackChatStorageConfig::default()` ⇒ mirror 字段为 None），`factory.rs:20-32` 中
    // kind=Sqlite ⇒ `pick_mirror_enabled` = false；SQLite 权威存储路径不变（factory.rs:16-18）。
    out.push((
        "OCLIVE_CHAT_STORAGE_BACKEND".to_string(),
        "sqlite".to_string(),
    ));
    out.push(("OCLIVE_OLLAMA_PRELOAD".to_string(), "0".to_string()));
    out.push(("OCLIVE_EVENT_IMPACT_LLM".to_string(), "0".to_string()));
    out.push(("OCLIVE_SKIP_LLM_STARTUP_PROBE".to_string(), "1".to_string()));
    // 肖像标签 LLM 缺省为真（portrait_emotion_engine.rs:17-27 / portrait_facility/director.rs:15-25）：
    // 不关闭会让每回合多出一次 generate_tag，破坏确定预算。
    out.push(("OCLIVE_PORTRAIT_EMOTION_LLM".to_string(), "0".to_string()));

    for (k, _) in &out {
        assert!(
            !DENIED_KEYS.contains(&k.as_str()),
            "[A-HARNESS] 子环境不得包含被拒键：{k}"
        );
    }
    // B uses the same isolated system environment, with only these explicit additions.
    // The proxy endpoint is chosen inside the owned child before the default builder runs.
    if scenario == super::B1 {
        out.push((
            super::ENV_B_APPROVAL.into(),
            crate::semantic_cases::approval_for_run(run_id)
                .expect("parent validated the frozen live mode before preparing child env")
                .into(),
        ));
        out.push(("OLLAMA_MODEL".into(), crate::live_proxy::MODEL.into()));
        out.push(("OCLIVE_OLLAMA_HTTP_TIMEOUT_SECS".into(), "60".into()));
    }
    out
}

pub fn apply(cmd: &mut std::process::Command, pairs: &[(String, String)]) {
    cmd.env_clear();
    for (k, v) in pairs {
        cmd.env(k, v);
    }
}

/// 非敏感环境事实（补充约束 §2）：只记**存在/缺失**，绝不记录值（尤其 token）。
#[must_use]
pub fn env_facts() -> serde_json::Value {
    let keys = [
        "OCLIVE_LLM_BACKEND",
        "OCLIVE_REMOTE_LLM_URL",
        "OCLIVE_REMOTE_LLM_TOKEN",
        "OCLIVE_LLM_CLOUD_API_STYLE",
        "OCLIVE_LOCAL_LLM_MODEL_PATH",
        "OCLIVE_LOCAL_LLM_LORA_PATH",
        "OLLAMA_BASE_URL",
        "OCLIVE_CHAT_STORAGE_ROOT",
        "OCLIVE_SKIP_LLM_STARTUP_PROBE",
        "OCLIVE_PORTRAIT_EMOTION_LLM",
        "OCLIVE_OLLAMA_PRELOAD",
        "OCLIVE_EVENT_IMPACT_LLM",
    ];
    let mut present = serde_json::Map::new();
    for k in keys {
        present.insert(
            k.to_string(),
            serde_json::Value::Bool(std::env::var(k).is_ok()),
        );
    }
    serde_json::json!({
        "presence_only": true,
        "values_recorded": false,
        "keys": serde_json::Value::Object(present),
    })
}

/// 非敏感、可安全记录的少数白名单值（仅本测试自己设置的键）。
#[must_use]
pub fn safe_set_values() -> serde_json::Value {
    let mut m = serde_json::Map::new();
    for k in BEHAVIOR_KEYS {
        if let Ok(v) = std::env::var(k) {
            m.insert((*k).to_string(), serde_json::Value::String(v));
        }
    }
    for k in ["TEMP", "TMP", "APPDATA", "LOCALAPPDATA", "USERPROFILE"] {
        if let Ok(v) = std::env::var(k) {
            m.insert(k.to_string(), serde_json::Value::String(v));
        }
    }
    serde_json::Value::Object(m)
}
