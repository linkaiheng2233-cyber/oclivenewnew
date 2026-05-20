//! `oclive plugin create` — 目录 / Remote 插件脚手架。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use oclive_validation::validate_directory_plugin_manifest_permissions;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct PluginCli {
    #[command(subcommand)]
    pub command: PluginCommands,
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// 生成目录或 Remote 插件骨架（manifest + RPC 桩 + README）
    Create(PluginCreateArgs),
    /// 安装插件并解析 plugin_dependencies
    Install(crate::plugin_ext::PluginInstallArgs),
    /// 卸载插件
    Uninstall(crate::plugin_ext::PluginUninstallArgs),
    /// RPC 契约烟测
    Test(crate::plugin_ext::PluginTestArgs),
    /// [deprecated] 搜索插件 — 请用 `oclive market search`
    Search(crate::plugin_ext::PluginSearchArgs),
    /// [deprecated] 检查更新 — 请用 `oclive market install`
    Update(crate::plugin_ext::PluginUpdateArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct PluginCreateArgs {
    /// 插件名称（用于生成 id 与显示名）
    pub name: String,

    /// 插件类型：directory（子进程 RPC）| remote（HTTP 侧车）
    #[arg(long, value_enum)]
    pub r#type: Option<PluginTypeArg>,

    /// 提供的槽位（可重复：--provides llm --provides memory）
    #[arg(long = "provides", value_enum)]
    pub provides: Vec<PluginSlotArg>,

    /// 输出根目录（默认 ./plugins/；最终目录为 <output>/<plugin_id>/）
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub output: PathBuf,

    /// 非交互（须同时提供 --type 与至少一个 --provides）
    #[arg(long)]
    pub non_interactive: bool,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
#[clap(rename_all = "kebab-case")]
pub enum PluginTypeArg {
    Directory,
    Remote,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[clap(rename_all = "kebab-case")]
pub enum PluginSlotArg {
    Llm,
    Memory,
    Emotion,
    Event,
    Prompt,
    Agent,
    ComplexEmotion,
}

impl PluginSlotArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Llm => "llm",
            Self::Memory => "memory",
            Self::Emotion => "emotion",
            Self::Event => "event",
            Self::Prompt => "prompt",
            Self::Agent => "agent",
            Self::ComplexEmotion => "complex_emotion",
        }
    }

    fn rpc_methods_for_slot(self) -> &'static [&'static str] {
        match self {
            Self::Llm => &["llm.generate", "llm.generate_tag"],
            Self::Memory => &["memory.rank"],
            Self::Emotion => &["emotion.analyze"],
            Self::Event => &["event.estimate"],
            Self::Prompt => &["prompt.build_prompt"],
            Self::Agent => &["agent.run_turn"],
            Self::ComplexEmotion => &["complex_emotion.resolve_turn"],
        }
    }
}

pub fn run(cli: PluginCli) -> Result<()> {
    match cli.command {
        PluginCommands::Create(args) => run_create(args),
        PluginCommands::Install(args) => crate::plugin_ext::run_install(args),
        PluginCommands::Uninstall(args) => crate::plugin_ext::run_uninstall(args),
        PluginCommands::Test(args) => crate::plugin_ext::run_test(args),
        PluginCommands::Search(args) => crate::plugin_ext::run_search(args),
        PluginCommands::Update(args) => crate::plugin_ext::run_update(args),
    }
}

fn slug_to_plugin_id(name: &str) -> String {
    let slug: String = name
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "com.oclive.plugin.unnamed".into()
    } else if slug.contains('.') {
        slug.to_string()
    } else {
        format!("com.oclive.plugin.{slug}")
    }
}

fn run_create(args: PluginCreateArgs) -> Result<()> {
    let display_name = if args.name.trim().is_empty() {
        bail!("插件名称不能为空");
    } else {
        args.name.trim().to_string()
    };

    let plugin_type = if let Some(t) = args.r#type {
        t
    } else if args.non_interactive {
        bail!("非交互模式须指定 --type directory 或 --type remote");
    } else {
        let items = ["directory（目录插件，Node RPC 子进程）", "remote（Remote HTTP 侧车，Python）"];
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("插件类型")
            .items(&items)
            .default(0)
            .interact()
            .context("plugin type")?;
        if idx == 0 {
            PluginTypeArg::Directory
        } else {
            PluginTypeArg::Remote
        }
    };

    let slots: Vec<PluginSlotArg> = if !args.provides.is_empty() {
        let mut v = args.provides.clone();
        v.sort();
        v.dedup();
        v
    } else if args.non_interactive {
        bail!("非交互模式须至少指定一个 --provides（如 --provides llm）");
    } else {
        let labels = [
            "llm（主对话）",
            "memory（记忆）",
            "emotion（情绪）",
            "event（事件）",
            "prompt（Prompt）",
            "agent（Agent）",
            "complex_emotion（复杂情感）",
        ];
        let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("提供哪些槽位（RPC 方法桩）")
            .items(&labels)
            .defaults(&[true, false, false, false, false, false, false])
            .interact()
            .context("provides")?;
        chosen
            .into_iter()
            .map(|i| {
                [
                    PluginSlotArg::Llm,
                    PluginSlotArg::Memory,
                    PluginSlotArg::Emotion,
                    PluginSlotArg::Event,
                    PluginSlotArg::Prompt,
                    PluginSlotArg::Agent,
                    PluginSlotArg::ComplexEmotion,
                ][i]
            })
            .collect()
    };

    if slots.is_empty() {
        bail!("至少选择一个 provides 槽位");
    }

    let plugin_id = slug_to_plugin_id(&display_name);
    let out_root = args.output.canonicalize().unwrap_or(args.output.clone());
    fs::create_dir_all(&out_root).context("create output")?;
    let plugin_dir = out_root.join(&plugin_id);
    if plugin_dir.exists() {
        bail!("目标目录已存在: {}", plugin_dir.display());
    }
    fs::create_dir_all(&plugin_dir).context("create plugin dir")?;

    let mut rpc_methods: Vec<&'static str> = Vec::new();
    let mut provides: Vec<&str> = Vec::new();
    for s in &slots {
        provides.push(s.as_str());
        rpc_methods.extend(s.rpc_methods_for_slot());
    }
    rpc_methods.sort();
    rpc_methods.dedup();

    let (manifest, readme, rpc_file) = match plugin_type {
        PluginTypeArg::Directory => build_directory_plugin(
            &plugin_id,
            &display_name,
            &provides,
            &rpc_methods,
            &slots,
        )?,
        PluginTypeArg::Remote => {
            build_remote_plugin(&plugin_id, &display_name, &provides, &rpc_methods, &slots)?
        }
    };

    fs::write(plugin_dir.join("manifest.json"), manifest).context("write manifest.json")?;
    fs::write(plugin_dir.join(readme.0), readme.1).context("write README.md")?;
    fs::write(plugin_dir.join(rpc_file.0), rpc_file.1).context("write rpc server")?;

    let manifest_str = fs::read_to_string(plugin_dir.join("manifest.json"))?;
    if plugin_type == PluginTypeArg::Directory {
        validate_directory_plugin_manifest_permissions(&manifest_str)
            .map_err(|e| anyhow::anyhow!("manifest 权限校验失败: {e}"))?;
    }

    println!("✓ 插件骨架已生成: {}", plugin_dir.display());
    println!("  manifest.json · {} · README.md", rpc_file.0);
    Ok(())
}

fn build_directory_plugin(
    plugin_id: &str,
    name: &str,
    provides: &[&str],
    rpc_methods: &[&'static str],
    slots: &[PluginSlotArg],
) -> Result<(String, (&'static str, String), (&'static str, String))> {
    let provides_json: Vec<&str> = provides.to_vec();
    let rpc_json: Vec<&str> = rpc_methods.to_vec();
    let manifest = json!({
        "schema_version": 1,
        "id": plugin_id,
        "name": name,
        "version": "0.1.0",
        "description": format!("{name} — directory plugin scaffold (oclive plugin create)"),
        "author": "oclive-cli",
        "provides": provides_json,
        "rpcMethods": rpc_json,
        "process": {
            "command": "node",
            "args": ["rpc_server.mjs"]
        },
        "permissions": ["process:spawn", "network:*"],
        "plugin_dependencies": []
    });
    let readme = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/plugins/README.directory.md"
    ))
    .replace("{{PLUGIN_ID}}", plugin_id)
    .replace("{{PLUGIN_NAME}}", name);
    let rpc = generate_directory_rpc_mjs(slots);
    Ok((
        serde_json::to_string_pretty(&manifest)?,
        ("README.md", readme),
        ("rpc_server.mjs", rpc),
    ))
}

fn build_remote_plugin(
    plugin_id: &str,
    name: &str,
    provides: &[&str],
    rpc_methods: &[&'static str],
    slots: &[PluginSlotArg],
) -> Result<(String, (&'static str, String), (&'static str, String))> {
    let manifest = json!({
        "schema_version": 1,
        "id": plugin_id,
        "name": name,
        "version": "0.1.0",
        "description": format!("{name} — remote HTTP plugin scaffold (oclive plugin create)"),
        "author": "oclive-cli",
        "provides": provides,
        "rpcMethods": rpc_methods,
        "permissions": ["network:*"],
        "remote": {
            "hint": "Set OCLIVE_REMOTE_PLUGIN_URL or OCLIVE_REMOTE_LLM_URL to http://127.0.0.1:PORT/rpc after starting rpc_server.py"
        }
    });
    let readme = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/plugins/README.remote.md"
    ))
    .replace("{{PLUGIN_ID}}", plugin_id)
    .replace("{{PLUGIN_NAME}}", name);
    let rpc = generate_remote_rpc_py(slots);
    Ok((
        serde_json::to_string_pretty(&manifest)?,
        ("README.md", readme),
        ("rpc_server.py", rpc),
    ))
}

fn generate_directory_rpc_mjs(slots: &[PluginSlotArg]) -> String {
    let mut stubs = String::new();
    for slot in slots {
        for method in slot.rpc_methods_for_slot() {
            stubs.push_str(&format!(
                "  \"{method}\": async (params) => ({{ ok: true, slot: \"{}\", method: \"{method}\", params }}),\n",
                slot.as_str()
            ));
        }
    }
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/plugins/rpc_server.mjs.hbs"
    ))
    .replace("{{METHOD_STUBS}}", &stubs)
}

fn generate_remote_rpc_py(slots: &[PluginSlotArg]) -> String {
    let mut arms = String::new();
    for slot in slots {
        for method in slot.rpc_methods_for_slot() {
            arms.push_str(&format!(
                "        if method == \"{method}\":\n            return self._send_json(req_id, result=handle_stub(\"{}\", \"{method}\", params))\n",
                slot.as_str()
            ));
        }
    }
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/templates/plugins/rpc_server.py.hbs"
    ))
    .replace("{{METHOD_ARMS}}", &arms)
}
