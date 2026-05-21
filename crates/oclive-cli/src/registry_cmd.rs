//! `oclive registry` 子命令。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::init::InitTemplateArg;
use crate::registry::{find_entry, load_registry, register_project, remove_entry, RegistryEntry};

#[derive(Parser, Debug)]
pub struct RegistryCli {
    #[command(subcommand)]
    pub command: RegistryCommands,
}

#[derive(Subcommand, Debug)]
pub enum RegistryCommands {
    /// List locally registered kernel projects
    List(RegistryListArgs),
    /// Register a project manually
    Add(RegistryAddArgs),
    /// Remove from registry (does not delete files on disk)
    Remove(RegistryRemoveArgs),
    /// Print command to switch working directory (Windows: cd /d; Unix: cd)
    Switch(RegistrySwitchArgs),
    /// [deprecated] Log in to cloud registry — use `oclive config set` (writes config.toml + auth.json)
    Login(crate::registry_remote::RegistryLoginArgs),
    /// Log out of cloud registry
    Logout,
    /// Push project as .oclive-template.tar.gz
    Push(crate::registry_remote::RegistryPushArgs),
    /// Pull project from cloud and register locally
    Pull(crate::registry_remote::RegistryPullArgs),
    /// Search cloud projects
    Search(crate::registry_remote::RegistrySearchArgs),
}

#[derive(Parser, Debug)]
pub struct RegistryListArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct RegistryAddArgs {
    pub name: String,
    pub path: PathBuf,
    #[arg(long)]
    pub template: Option<String>,
}

#[derive(Parser, Debug)]
pub struct RegistryRemoveArgs {
    pub name: String,
}

#[derive(Parser, Debug)]
pub struct RegistrySwitchArgs {
    pub name: String,
}

pub fn run(cli: RegistryCli) -> Result<()> {
    match cli.command {
        RegistryCommands::List(a) => run_list(a),
        RegistryCommands::Add(a) => run_add(a),
        RegistryCommands::Remove(a) => run_remove(a),
        RegistryCommands::Switch(a) => run_switch(a),
        RegistryCommands::Login(a) => crate::registry_remote::run_login(a),
        RegistryCommands::Logout => crate::registry_remote::run_logout(),
        RegistryCommands::Push(a) => crate::registry_remote::run_push(a),
        RegistryCommands::Pull(a) => crate::registry_remote::run_pull(a),
        RegistryCommands::Search(a) => crate::registry_remote::run_search(a),
    }
}

fn run_list(args: RegistryListArgs) -> Result<()> {
    let file = load_registry()?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&file.projects)?);
        return Ok(());
    }
    if file.projects.is_empty() {
        println!("(registry empty; successful `oclive init` auto-registers)");
        println!("Path: {}", crate::registry::registry_path().display());
        return Ok(());
    }
    println!("{:<24} {:<12} {:<10} path", "name", "template", "created");
    println!("{}", "-".repeat(96));
    for p in &file.projects {
        let tpl = p.template.as_deref().unwrap_or("—");
        let date = format_date(p.created_at);
        println!("{:<24} {:<12} {:<10} {}", p.name, tpl, date, p.path);
    }
    Ok(())
}

fn run_add(args: RegistryAddArgs) -> Result<()> {
    let path = args
        .path
        .canonicalize()
        .with_context(|| format!("path {}", args.path.display()))?;
    if !path.join("Cargo.toml").is_file() {
        bail!(
            "{} is not a valid Cargo project root (missing Cargo.toml)",
            path.display()
        );
    }
    let template = args.template.as_deref().and_then(template_from_str);
    register_project(&args.name, &path, template)?;
    println!("Registered: {} → {}", args.name, path.display());
    Ok(())
}

fn run_remove(args: RegistryRemoveArgs) -> Result<()> {
    if remove_entry(&args.name)? {
        println!(
            "Removed from registry: {} (project directory not deleted)",
            args.name
        );
    } else {
        bail!("No project in registry: {}", args.name);
    }
    Ok(())
}

fn run_switch(args: RegistrySwitchArgs) -> Result<()> {
    let entry = find_entry(&args.name)?
        .ok_or_else(|| anyhow::anyhow!("No project in registry: {}", args.name))?;
    print_switch_hint(&entry);
    Ok(())
}

pub fn print_switch_hint(entry: &RegistryEntry) {
    if cfg!(windows) {
        println!("cd /d \"{}\"", entry.path);
    } else {
        println!("cd \"{}\"", entry.path);
    }
}

fn template_from_str(s: &str) -> Option<InitTemplateArg> {
    match s {
        "robot-soul" => Some(InitTemplateArg::RobotSoul),
        "robot-gateway" => Some(InitTemplateArg::RobotGateway),
        "dialogue-only" => Some(InitTemplateArg::DialogueOnly),
        "headless-api" => Some(InitTemplateArg::HeadlessApi),
        "library-embed" => Some(InitTemplateArg::LibraryEmbed),
        _ => None,
    }
}

fn format_date(ts: u64) -> String {
    let days = ts / 86_400;
    let y = 1970 + days / 365;
    format!("{y}-{:02}", (days % 365 / 30 + 1).min(12))
}
