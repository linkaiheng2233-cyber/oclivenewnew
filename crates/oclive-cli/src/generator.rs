//! 模板渲染与落盘。

use crate::init::{BackendImpl, InitTemplateArg, ProjectConfig, ProjectType};
use crate::monolith_codegen;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// 用于 `Cargo.toml` package 名、二进制名与终端提示。
pub fn project_slug(cfg: &ProjectConfig) -> String {
    slugify(&cfg.project_name)
}

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

fn template_context(cfg: &ProjectConfig, out: &Path) -> serde_json::Value {
    let safe_package_name = slugify(&cfg.project_name);
    let rust_lib_name = safe_package_name.replace('-', "_");
    let monolith_enabled = cfg.monolith_enabled && cfg.project_type == ProjectType::KernelServer;
    let mut ctx = serde_json::json!({
        "project_name": cfg.project_name,
        "safe_package_name": safe_package_name,
        "rust_lib_name": rust_lib_name,
        "project_type": format!("{:?}", cfg.project_type),
        "plugin_directory": cfg.plugins.directory_plugins,
        "plugin_kernel_server": cfg.plugins.kernel_server,
        "plugin_oocp": cfg.plugins.oocp,
        "feature_complex_emotion": cfg.features.use_complex_emotion,
        "has_role_pack": cfg.role_pack_kind != crate::init::RolePackKind::None,
        "monolith_enabled": monolith_enabled,
        "kernel_linked": false,
        "cargo_author": cfg.cargo_author.as_deref().unwrap_or(""),
        "cargo_license": cfg.cargo_license.as_deref().unwrap_or("MIT"),
        "cargo_description": cfg.cargo_description.as_deref().unwrap_or(""),
    });
    if let Some(ref root) = cfg.kernel_source {
        let path_tauri = relativize_path(out, &root.join("src-tauri"));
        let path_runtime = relativize_path(out, &root.join("crates/oclive_kernel_runtime"));
        let lib_demo = cfg.project_type == ProjectType::Library;
        let http_entry = cfg.project_type == ProjectType::KernelServer;
        if let Some(obj) = ctx.as_object_mut() {
            obj.insert("kernel_linked".into(), json!(true));
            obj.insert("path_tauri".into(), json!(path_tauri));
            obj.insert("path_runtime".into(), json!(path_runtime));
            obj.insert("library_kernel_demo".into(), json!(lib_demo));
            obj.insert("kernel_server_http_entry".into(), json!(http_entry));
        }
    }
    ctx
}

/// `--kernel-source` 须为 oclivenewnew 仓库根。
pub fn validate_kernel_source(root: &Path) -> Result<()> {
    let tauri = root.join("src-tauri").join("Cargo.toml");
    let runtime = root
        .join("crates")
        .join("oclive_kernel_runtime")
        .join("Cargo.toml");
    if !tauri.is_file() || !runtime.is_file() {
        anyhow::bail!(
            "--kernel-source must point to oclivenewnew repo root (needs src-tauri/ and crates/oclive_kernel_runtime/)"
        );
    }
    Ok(())
}

fn relativize_path(from: &Path, to: &Path) -> String {
    let from = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());
    let to = to.canonicalize().unwrap_or_else(|_| to.to_path_buf());
    let from_c: Vec<_> = from.components().collect();
    let to_c: Vec<_> = to.components().collect();
    let mut shared = 0usize;
    while shared < from_c.len() && shared < to_c.len() && from_c[shared] == to_c[shared] {
        shared += 1;
    }
    let mut rel = PathBuf::new();
    for _ in shared..from_c.len() {
        rel.push("..");
    }
    for c in &to_c[shared..] {
        rel.push(c.as_os_str());
    }
    rel.to_string_lossy().replace('\\', "/")
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

fn write_pipeline_artifacts(cfg: &ProjectConfig, out: &Path) -> Result<()> {
    let docs = out.join("docs");
    fs::create_dir_all(&docs).context("create docs")?;
    fs::write(
        docs.join("PIPELINE_CUSTOM.md"),
        cfg.pipeline.doc_markdown(),
    )
    .context("write PIPELINE_CUSTOM.md")?;
    let steps = cfg.pipeline.steps();
    let body: String = steps
        .iter()
        .map(|s| format!("    \"{s}\",\n"))
        .collect();
    let rs = format!(
        "//! 编排步骤顺序快照（`oclive init --pipeline {:?}`）。\n//! 完整宿主以 oclivenewnew `process_message` 为准。\n\npub const OCLIVE_PIPELINE_STEPS: &[&str] = &[\n{body}];\n",
        cfg.pipeline
    );
    fs::write(out.join("src/oclive_pipeline_order.rs"), rs).context("write oclive_pipeline_order.rs")?;
    Ok(())
}

fn write_kernel_dev_docs(out: &Path) -> Result<()> {
    let docs = out.join("docs");
    fs::create_dir_all(&docs).context("create docs")?;
    fs::write(
        docs.join("BLUEPRINT_REFERENCE.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/BLUEPRINT_REFERENCE.md"
        )),
    )
    .context("write BLUEPRINT_REFERENCE.md")?;
    fs::write(
        docs.join("ORCHESTRATION_REFERENCE.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/ORCHESTRATION_REFERENCE.md"
        )),
    )
    .context("write ORCHESTRATION_REFERENCE.md")?;
    fs::write(
        docs.join("ORCHESTRATION_REFERENCE.en.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/ORCHESTRATION_REFERENCE.en.md"
        )),
    )
    .context("write ORCHESTRATION_REFERENCE.en.md")?;
    fs::write(
        docs.join("WELD_BENCH_REPORT.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/WELD_BENCH_REPORT.md"
        )),
    )
    .context("write WELD_BENCH_REPORT.md")?;
    fs::write(
        docs.join("WELD_BENCH_REPORT.en.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/WELD_BENCH_REPORT.en.md"
        )),
    )
    .context("write WELD_BENCH_REPORT.en.md")?;
    fs::write(
        docs.join("DEBUG_REFERENCE.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/docs/DEBUG_REFERENCE.md"
        )),
    )
    .context("write DEBUG_REFERENCE.md")?;
    Ok(())
}

fn write_robot_gateway_extras(out: &Path) -> Result<()> {
    let mcp_dir = out.join("mcp_servers");
    fs::create_dir_all(&mcp_dir).context("create mcp_servers")?;
    fs::write(
        mcp_dir.join("README.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/mcp_servers/README.md"
        )),
    )
    .context("write mcp_servers/README.md")?;
    fs::write(
        mcp_dir.join("smart_home.example.json"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/mcp_servers/smart_home.example.json"
        )),
    )
    .context("write smart_home.example.json")?;

    let role_root = out.join("roles/gateway");
    fs::create_dir_all(role_root.join("scenes").join("default")).context("create roles/gateway")?;

    let mut settings: serde_json::Value =
        serde_json::from_str(&render_settings_json(&preset_gateway_config())?)
            .context("parse gateway settings")?;
    if let Some(obj) = settings.as_object_mut() {
        obj.insert(
            "agent_mcp".to_string(),
            json!({
                "_comment": "脚手架占位：建议将 mcp_servers/*.json 同步到宿主 {app_data}/mcp-servers/",
                "local_scan_dir": "mcp_servers",
                "server_ids": ["smart_home_stub"]
            }),
        );
    }
    fs::write(
        role_root.join("settings.json"),
        serde_json::to_string_pretty(&settings).context("gateway settings")?,
    )
    .context("write roles/gateway/settings.json")?;

    let manifest = json!({
        "id": "gateway",
        "name": "Smart gateway (OEM stub)",
        "version": "0.1.0",
        "author": "oclive-cli",
        "description": "Replace with vendor role pack; agent uses builtin + MCP.",
        "min_runtime_version": "0.2.0",
        "scenes": ["default"],
        "user_relations": {
            "household": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
        },
        "default_relation": "household"
    });
    fs::write(
        role_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("gateway manifest")?,
    )
    .context("write manifest.json")?;
    fs::write(
        role_root.join("character.md"),
        "# Gateway persona (OEM)\n\nCoordinate smart home devices via Agent + MCP.\n",
    )
    .context("write character.md")?;
    let scene = json!({
        "name": "Default",
        "time_windows": [],
        "keywords": [],
        "events": []
    });
    fs::write(
        role_root.join("scenes/default/scene.json"),
        serde_json::to_string_pretty(&scene).context("scene")?,
    )
    .context("write scene.json")?;
    Ok(())
}

fn preset_gateway_config() -> ProjectConfig {
    let mut cfg = crate::init::preset_config("gateway", "mixed");
    cfg.backends.agent = BackendImpl::Builtin;
    cfg.role_pack_kind = crate::init::RolePackKind::None;
    cfg
}

fn example_llamacpp_plugin_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/directory-plugin-llamacpp")
}

fn copy_example_llamacpp_plugin(out: &Path) -> Result<()> {
    let src = example_llamacpp_plugin_src();
    if !src.is_dir() {
        anyhow::bail!(
            "Example plugin source missing: {} (run oclive-cli from oclivenewnew repo)",
            src.display()
        );
    }
    let dst = out.join("plugins/com.oclive.example.llamacpp_llm");
    copy_dir_all(&src, &dst).with_context(|| format!("copy example plugin to {}", dst.display()))
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name();
        let to = dst.join(name);
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
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
                "Output directory not empty: {}. Use an empty directory or delete it first.",
                out.display()
            );
        }
    } else {
        fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    }
    fs::create_dir_all(out.join("src")).context("create src")?;

    let reg = Handlebars::new();
    let ctx = template_context(cfg, out);

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
            if cfg.monolith_enabled && matches!(cfg.project_type, ProjectType::KernelServer) {
                let main_mono_t = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/templates/main_monolith.rs.hbs"
                ));
                let main_mono = reg
                    .render_template(main_mono_t, &ctx)
                    .context("render main_monolith.rs")?;
                fs::write(out.join("src").join("main_monolith.rs"), main_mono)
                    .context("write main_monolith.rs")?;
                monolith_codegen::copy_monolith_vendor(out)
                    .context("copy vendor/oclive_monolith_builtin")?;
                let weld = crate::init::resolve_monolith_weld_modules(cfg);
                let weld_refs: Vec<&str> = weld.iter().map(|s| s.as_str()).collect();
                let (toml, plan) = monolith_codegen::monolith_toml_and_plan(&weld_refs);
                fs::write(
                    out.join("src").join("process_message_monolith.rs"),
                    monolith_codegen::generate_monolith_source(&plan),
                )
                .context("write process_message_monolith.rs")?;
                fs::write(out.join("monolith.toml"), toml).context("write monolith.toml")?;
            }
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

    write_kernel_dev_docs(out)?;
    write_pipeline_artifacts(cfg, out)?;

    fs::create_dir_all(out.join("plugins")).context("create plugins")?;
    fs::write(
        out.join("plugins").join("README.md"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/templates/plugins_README.md"
        )),
    )
    .context("write plugins/README.md")?;

    if cfg.with_example_plugin {
        copy_example_llamacpp_plugin(out)?;
    }

    if cfg.factory_template == Some(InitTemplateArg::RobotGateway) {
        write_robot_gateway_extras(out).context("robot-gateway MCP scaffold")?;
    }

    if !cfg.skip_role_pack && cfg.role_pack_kind != crate::init::RolePackKind::None {
        crate::role_pack::write_role_pack(cfg, out).context("write role pack")?;
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

    #[test]
    fn robot_gateway_writes_mcp_and_gateway_role() {
        use crate::init::{preset_config, InitTemplateArg};
        use tempfile::tempdir;

        let mut cfg = preset_config("gw", "mixed");
        cfg.factory_template = Some(InitTemplateArg::RobotGateway);
        cfg.monolith_enabled = true;
        let dir = tempdir().unwrap();
        write_project(&cfg, dir.path()).unwrap();
        assert!(dir.path().join("mcp_servers/README.md").is_file());
        assert!(dir.path().join("roles/gateway/settings.json").is_file());
    }

    #[test]
    fn kernel_linked_cargo_contains_path_deps() {
        use crate::init::{preset_config, ProjectType};
        use std::path::PathBuf;
        use tempfile::tempdir;

        let mut cfg = preset_config("linked-kernel", "minimal");
        cfg.project_type = ProjectType::KernelServer;
        cfg.kernel_source = Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
        let out = tempdir().unwrap();
        write_project(&cfg, out.path()).unwrap();
        let cargo = std::fs::read_to_string(out.path().join("Cargo.toml")).unwrap();
        assert!(cargo.contains("oclivenewnew-tauri"));
        assert!(cargo.contains("oclive_kernel_runtime"));
        let main_rs = std::fs::read_to_string(out.path().join("src/main.rs")).unwrap();
        assert!(main_rs.contains("run_api_server"));
    }
}
