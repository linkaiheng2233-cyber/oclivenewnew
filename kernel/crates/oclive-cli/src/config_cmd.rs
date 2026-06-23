//! `oclive config` — global and project-level configuration.

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use std::env;
use std::path::PathBuf;

use crate::config::{self, KNOWN_KEYS};

#[derive(Parser, Debug)]
pub struct ConfigCli {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    Set(ConfigSetArgs),
    Get(ConfigGetArgs),
    List(ConfigListArgs),
    Unset(ConfigUnsetArgs),
    /// Interactive config wizard (imports unset OCLIVE_* from the environment)
    Init(ConfigInitArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigSetArgs {
    pub key: String,
    pub value: String,
    #[arg(long, conflicts_with = "local")]
    pub global: bool,
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
}

#[derive(Parser, Debug)]
pub struct ConfigGetArgs {
    pub key: String,
    #[arg(long, conflicts_with = "local")]
    pub global: bool,
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
}

#[derive(Parser, Debug)]
pub struct ConfigListArgs {
    #[arg(long, conflicts_with = "local")]
    pub global: bool,
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct ConfigUnsetArgs {
    pub key: String,
    #[arg(long, conflicts_with = "local")]
    pub global: bool,
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
}

#[derive(Parser, Debug)]
pub struct ConfigInitArgs {
    #[arg(long)]
    pub global: bool,
}

pub fn run(cli: ConfigCli) -> Result<()> {
    match cli.command {
        ConfigCommands::Set(a) => run_set(a),
        ConfigCommands::Get(a) => run_get(a),
        ConfigCommands::List(a) => run_list(a),
        ConfigCommands::Unset(a) => run_unset(a),
        ConfigCommands::Init(a) => run_init(a),
    }
}

fn project_root() -> Option<PathBuf> {
    env::current_dir().ok()
}

fn scope_global(args_global: bool, args_local: bool) -> bool {
    if args_local {
        false
    } else {
        args_global || !args_local
    }
}

fn run_set(args: ConfigSetArgs) -> Result<()> {
    let global = scope_global(args.global, args.local);
    let path = config::set_key(&args.key, &args.value, global, project_root().as_deref())?;
    println!("Wrote {}: {}={}", path.display(), args.key, args.value);
    Ok(())
}

fn run_get(args: ConfigGetArgs) -> Result<()> {
    let global = scope_global(args.global, args.local);
    let root = project_root();
    let effective = config::resolve(&args.key, root.as_deref());
    if let Some(v) = if global {
        config::get_key(&args.key, true, None)?
    } else {
        config::get_key(&args.key, false, root.as_deref())?
    } {
        println!("{v}");
        return Ok(());
    }
    if let Some(v) = effective {
        println!("{v}");
        return Ok(());
    }
    bail!("Not configured: {}", args.key);
}

fn run_list(args: ConfigListArgs) -> Result<()> {
    let global = scope_global(args.global, args.local);
    let map = config::list_keys(global, project_root().as_deref())?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&map)?);
        return Ok(());
    }
    let path = if global {
        config::global_config_path()
    } else {
        project_root()
            .map(|r| config::local_config_path(&r))
            .unwrap_or_else(config::global_config_path)
    };
    println!("oclive config list — {}\n", path.display());
    for (k, v) in &map {
        println!("  {k} = {v}");
    }
    Ok(())
}

fn run_unset(args: ConfigUnsetArgs) -> Result<()> {
    let global = scope_global(args.global, args.local);
    let path = config::unset_key(&args.key, global, project_root().as_deref())?;
    println!("Removed {} from {}", args.key, path.display());
    Ok(())
}

fn run_init(_args: ConfigInitArgs) -> Result<()> {
    let n = config::import_env_to_global()?;
    if n > 0 {
        println!(
            "Imported {n} key(s) from environment into {}",
            config::global_config_path().display()
        );
    }
    let theme = ColorfulTheme::default();
    for key in KNOWN_KEYS {
        let current = config::resolve(key, None).unwrap_or_default();
        let label = format!(
            "{key} (current: {})",
            if current.is_empty() { "—" } else { &current }
        );
        let val: String = Input::with_theme(&theme)
            .with_prompt(&label)
            .default(current)
            .allow_empty(true)
            .interact_text()?;
        if !val.trim().is_empty() {
            config::set_key(key, val.trim(), true, None)?;
        }
    }
    let more = Confirm::with_theme(&theme)
        .with_prompt("Add more OCLIVE_* keys?")
        .default(false)
        .interact()?;
    if more {
        loop {
            let key: String = Input::with_theme(&theme)
                .with_prompt("Key name (empty to finish)")
                .allow_empty(true)
                .interact_text()?;
            if key.trim().is_empty() {
                break;
            }
            let val: String = Input::with_theme(&theme)
                .with_prompt("Value")
                .interact_text()?;
            config::set_key(key.trim(), val.trim(), true, None)?;
        }
    }
    println!("Config saved to {}", config::global_config_path().display());
    Ok(())
}
