//! 模板渲染与落盘。

use crate::init::{BackendImpl, ProjectConfig, ProjectType};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

fn slugify(raw: &str) -> String {
    let mut s: String = raw
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let t = s.trim_matches('-').to_string();
    if t.is_empty() {
        "my-oclive-kernel".into()
    } else {
        t
    }
}

fn template_context(cfg: &ProjectConfig) -> serde_json::Value {
    let safe_package_name = slugify(&cfg.project_name);
    let rust_lib_name = safe_package_name.replace('-', "_");
    serde_json::json!({
        "project_name": cfg.project_name,
        "safe_package_name": safe_package_name,
        "rust_lib_name": rust_lib_name,
        "project_type": format!("{:?}", cfg.project_type),
        "plugin_directory": cfg.plugins.directory_plugins,
        "plugin_kernel_server": cfg.plugins.kernel_server,
        "plugin_oocp": cfg.plugins.oocp,
        "feature_complex_emotion": cfg.features.use_complex_emotion,
        "with_example_role": cfg.with_example_role,
    })
}

const COMMENT_PLUGIN_BACKENDS: &str = "七条编排槽位（与 PLUGIN_V1 / plugin_host 对齐）。主应用当前反序列化 6 个标准槽；complex_emotion 为扩展键，宿主会忽略未知字段。可选值以各槽枚举为准（见 SETTINGS_REFERENCE.md）。";

const COMMENT_MEMORY: &str = "记忆检索 (memory.rank)。常用: builtin | builtin_v2 | remote | directory | local。选 none：不参与检索排序（主应用无 none 枚举；若复制进 oclive 请删除该键或改为 builtin）。";

const COMMENT_EMOTION: &str = "用户情绪分析 (emotion.analyze)。常用: builtin | builtin_v2 | remote | directory。选 none：跳过该子系统（主应用请删除键或改为 builtin）。";

const COMMENT_EVENT: &str = "事件影响估计 (event.estimate)。常用: builtin | builtin_v2 | remote | directory。选 none：跳过事件估计链（主应用请删除键或改为 builtin）。";

const COMMENT_PROMPT: &str = "Prompt 组装 (prompt.build_prompt)。常用: builtin | builtin_v2 | remote | directory。选 none：无有效 prompt 组装（主应用请删除键或改为 builtin）。";

const COMMENT_LLM: &str = "主对话 LLM (llm.generate)。主应用枚举: ollama（本机默认进程内客户端，需 Ollama）| remote（HTTP 侧车，需 OCLIVE_REMOTE_LLM_URL）| directory。若无本地模型，请改为 remote 并配置远端 URL。选 none：主生成链不可用（仅实验）。";

const COMMENT_AGENT: &str = "Agent / 工具编排 (ReAct)。常用: builtin | remote | directory。预设 minimal 在 JSON 中省略本键（语义「不额外声明」= 宿主默认 builtin）。选 none 且需写入 JSON 时: 主应用无 none，请省略 agent 键。";

const COMMENT_COMPLEX_EMOTION: &str = "复杂情感扩展（路线图）。可写 builtin | remote | directory | none 作团队约定；当前桌面 PluginBackends 不含此槽，宿主加载时会忽略。remote 时需侧车并实现协议（见 REMOTE_PLUGIN_PROTOCOL.md）。";

/// 构建 `roles/default/settings.json` 根对象（含 `_comment_*` 与完整 `plugin_backends`）。
pub fn build_settings_value(cfg: &ProjectConfig) -> Value {
    let mut root = Map::new();
    root.insert(
        "_comment_plugin_backends".to_string(),
        json!(COMMENT_PLUGIN_BACKENDS),
    );
    root.insert("_comment_memory".to_string(), json!(COMMENT_MEMORY));
    root.insert("_comment_emotion".to_string(), json!(COMMENT_EMOTION));
    root.insert("_comment_event".to_string(), json!(COMMENT_EVENT));
    root.insert("_comment_prompt".to_string(), json!(COMMENT_PROMPT));
    root.insert("_comment_llm".to_string(), json!(COMMENT_LLM));
    root.insert("_comment_agent".to_string(), json!(COMMENT_AGENT));
    root.insert(
        "_comment_complex_emotion".to_string(),
        json!(COMMENT_COMPLEX_EMOTION),
    );
    root.insert("schema_version".to_string(), json!(1));

    let mut pb = Map::new();
    pb.insert(
        "memory".to_string(),
        json!(line_memory(cfg.backends.memory)),
    );
    pb.insert(
        "emotion".to_string(),
        json!(line_standard(cfg.backends.emotion)),
    );
    pb.insert(
        "event".to_string(),
        json!(line_standard(cfg.backends.event)),
    );
    pb.insert(
        "prompt".to_string(),
        json!(line_standard(cfg.backends.prompt)),
    );
    pb.insert("llm".to_string(), json!(line_llm(cfg.backends.llm)));

    if cfg.backends.agent != BackendImpl::None {
        pb.insert(
            "agent".to_string(),
            json!(line_standard(cfg.backends.agent)),
        );
    }

    pb.insert(
        "complex_emotion".to_string(),
        json!(line_complex(cfg.backends.complex_emotion)),
    );

    let any_directory = matches!(cfg.backends.memory, BackendImpl::Directory)
        || matches!(cfg.backends.emotion, BackendImpl::Directory)
        || matches!(cfg.backends.event, BackendImpl::Directory)
        || matches!(cfg.backends.prompt, BackendImpl::Directory)
        || matches!(cfg.backends.llm, BackendImpl::Directory)
        || matches!(cfg.backends.agent, BackendImpl::Directory);

    if cfg.plugins.directory_plugins || any_directory {
        pb.insert("directory_plugins".to_string(), json!({}));
        pb.insert(
            "_comment_directory_plugins".to_string(),
            json!("任一槽为 directory 或启用目录插件说明时给出；各槽填插件 manifest id，见 DIRECTORY_PLUGINS.md"),
        );
    }

    root.insert("plugin_backends".to_string(), Value::Object(pb));

    let mut ocli = Map::new();
    ocli.insert("generator".to_string(), json!("oclive-cli"));
    ocli.insert(
        "plugins".to_string(),
        json!({
            "directory_plugins": cfg.plugins.directory_plugins,
            "kernel_server_doc": cfg.plugins.kernel_server,
            "oocp_doc": cfg.plugins.oocp,
        }),
    );
    ocli.insert(
        "features".to_string(),
        json!({
            "use_complex_emotion": cfg.features.use_complex_emotion,
        }),
    );
    root.insert("_oclive_cli".to_string(), Value::Object(ocli));

    Value::Object(root)
}

pub fn render_settings_json(cfg: &ProjectConfig) -> Result<String> {
    serde_json::to_string_pretty(&build_settings_value(cfg)).context("serialize settings.json")
}

fn line_standard(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Builtin => "builtin",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::Ollama => "builtin",
        BackendImpl::None => "none",
    }
}

fn line_memory(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Builtin => "builtin",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::Ollama => "builtin",
        BackendImpl::None => "none",
    }
}

fn line_llm(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Ollama | BackendImpl::Builtin => "ollama",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::None => "none",
    }
}

fn line_complex(b: BackendImpl) -> &'static str {
    match b {
        BackendImpl::Builtin => "builtin",
        BackendImpl::Remote => "remote",
        BackendImpl::Directory => "directory",
        BackendImpl::Ollama => "ollama",
        BackendImpl::None => "none",
    }
}

fn config_reference_markdown() -> &'static str {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/CONFIG_REFERENCE.md"
    ))
}

pub fn write_project(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    if out.exists() {
        let mut it = fs::read_dir(out).with_context(|| format!("read_dir {}", out.display()))?;
        if it.next().transpose()?.is_some() {
            anyhow::bail!(
                "输出目录非空：{}。请使用空目录或删除后再试。",
                out.display()
            );
        }
    } else {
        fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    }
    fs::create_dir_all(out.join("src")).context("create src")?;

    let reg = Handlebars::new();
    let ctx = template_context(cfg);

    let cargo_tmpl = match cfg.project_type {
        ProjectType::KernelServer => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/Cargo.kernel.toml.hbs"
            ))
        }
        ProjectType::Library => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/Cargo.library.toml.hbs"
            ))
        }
    };
    let cargo = reg
        .render_template(cargo_tmpl, &ctx)
        .context("render Cargo.toml")?;
    fs::write(out.join("Cargo.toml"), cargo).context("write Cargo.toml")?;

    match cfg.project_type {
        ProjectType::KernelServer => {
            let main_t = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/main.rs.hbs"
            ));
            let main = reg
                .render_template(main_t, &ctx)
                .context("render main.rs")?;
            fs::write(out.join("src").join("main.rs"), main).context("write main.rs")?;
        }
        ProjectType::Library => {
            let lib_t = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/lib.rs.hbs"
            ));
            let lib = reg.render_template(lib_t, &ctx).context("render lib.rs")?;
            fs::write(out.join("src").join("lib.rs"), lib).context("write lib.rs")?;
        }
    }

    let readme_t = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/README.generated.hbs"
    ));
    let readme = reg
        .render_template(readme_t, &ctx)
        .context("render README")?;
    fs::write(out.join("README.md"), readme).context("write README")?;

    fs::write(out.join("CONFIG_REFERENCE.md"), config_reference_markdown())
        .context("write CONFIG_REFERENCE.md")?;

    if cfg.with_example_role {
        fs::create_dir_all(out.join("roles").join("default")).context("create roles/default")?;
        let settings = render_settings_json(cfg).context("settings.json")?;
        fs::write(out.join("roles/default/settings.json"), settings)
            .context("write settings.json")?;
        fs::write(
            out.join("roles/default/character.md"),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/templates/character.md"
            )),
        )
        .context("write character.md")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init::preset_config;

    #[test]
    fn slugify_keeps_alnum() {
        assert_eq!(slugify("  My_Project "), "my-project");
    }

    #[test]
    fn minimal_preset_omits_agent_key_in_json() {
        let cfg = preset_config("t", "minimal");
        let v = build_settings_value(&cfg);
        let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
        assert!(!pb.contains_key("agent"));
        assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "ollama");
        assert_eq!(pb.get("memory").unwrap().as_str().unwrap(), "builtin");
    }

    #[test]
    fn full_preset_matches_matrix() {
        let cfg = preset_config("t", "full");
        let doc = build_settings_value(&cfg);
        let pb = doc.get("plugin_backends").unwrap().as_object().unwrap();
        assert_eq!(pb.get("memory").unwrap().as_str().unwrap(), "builtin");
        assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "remote");
        assert_eq!(pb.get("agent").unwrap().as_str().unwrap(), "builtin");
        assert_eq!(
            pb.get("complex_emotion").unwrap().as_str().unwrap(),
            "remote"
        );
    }

    #[test]
    fn mixed_preset_matches_matrix() {
        let cfg = preset_config("t", "mixed");
        let doc = build_settings_value(&cfg);
        let pb = doc.get("plugin_backends").unwrap().as_object().unwrap();
        assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "ollama");
        assert_eq!(pb.get("agent").unwrap().as_str().unwrap(), "builtin");
        assert_eq!(
            pb.get("complex_emotion").unwrap().as_str().unwrap(),
            "builtin"
        );
    }
}
