//! `oclive plugin create` — 目录 / Remote 插件脚手架。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use oclive_validation::validate_directory_plugin_manifest_permissions;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

type PluginScaffoldBundle = (String, (&'static str, String), (&'static str, String));

#[derive(Parser, Debug)]
pub struct PluginCli {
    #[command(subcommand)]
    pub command: PluginCommands,
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// Scaffold directory or Remote plugin (manifest + RPC stubs + README)
    Create(PluginCreateArgs),
    /// Install plugin and resolve plugin_dependencies
    Install(crate::plugin_ext::PluginInstallArgs),
    /// Uninstall plugin
    Uninstall(crate::plugin_ext::PluginUninstallArgs),
    /// RPC contract smoke test
    Test(crate::plugin_ext::PluginTestArgs),
    /// Advanced slot / blueprint management (list, link, TUI)
    Manage(crate::plugin_manage_cmd::PluginManageCli),
}

#[derive(Parser, Debug, Clone)]
pub struct PluginCreateArgs {
    /// Plugin name (used for id and display name)
    pub name: String,

    /// Plugin type: directory (subprocess RPC) | remote (HTTP sidecar)
    #[arg(long, value_enum)]
    pub r#type: Option<PluginTypeArg>,

    /// Slots provided (repeat: --provides llm --provides memory)
    #[arg(long = "provides", value_enum)]
    pub provides: Vec<PluginSlotArg>,

    /// Output root (default ./plugins/; final dir is <output>/<plugin_id>/)
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub output: PathBuf,

    /// Non-interactive (requires --type and at least one --provides)
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
        PluginCommands::Manage(args) => crate::plugin_manage_cmd::run_manage(args),
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
        bail!("Plugin name cannot be empty");
    } else {
        args.name.trim().to_string()
    };

    let plugin_type = if let Some(t) = args.r#type {
        t
    } else if args.non_interactive {
        bail!("Non-interactive mode requires --type directory or --type remote");
    } else {
        let items = [
            "directory (directory plugin, Node RPC subprocess)",
            "remote (Remote HTTP sidecar, Python)",
        ];
        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Plugin type")
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
        bail!("Non-interactive mode requires at least one --provides (e.g. --provides llm)");
    } else {
        let labels = [
            "llm (main dialogue)",
            "memory",
            "emotion",
            "event",
            "prompt",
            "agent",
            "complex_emotion",
        ];
        let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Which slots to provide (RPC method stubs)")
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
        bail!("Select at least one provides slot");
    }

    let plugin_id = slug_to_plugin_id(&display_name);
    let out_root = args.output.canonicalize().unwrap_or(args.output.clone());
    fs::create_dir_all(&out_root).context("create output")?;
    let plugin_dir = out_root.join(&plugin_id);
    if plugin_dir.exists() {
        bail!("Target directory already exists: {}", plugin_dir.display());
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
        PluginTypeArg::Directory => {
            build_directory_plugin(&plugin_id, &display_name, &provides, &rpc_methods, &slots)?
        }
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
            .map_err(|e| anyhow::anyhow!("manifest permission validation failed: {e}"))?;
    }

    println!("✓ Plugin scaffold generated: {}", plugin_dir.display());
    println!("  manifest.json · {} · README.md", rpc_file.0);
    Ok(())
}

fn build_directory_plugin(
    plugin_id: &str,
    name: &str,
    provides: &[&str],
    rpc_methods: &[&'static str],
    slots: &[PluginSlotArg],
) -> Result<PluginScaffoldBundle> {
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
) -> Result<PluginScaffoldBundle> {
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
