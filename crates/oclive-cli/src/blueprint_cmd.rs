//! `oclive blueprint` — 蓝图读取与校验。

use crate::blueprint::{load_blueprint, validate_blueprint};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    about = "[experimental/legacy] Validate pipeline.ocblueprint (desktop main path removed; kernel-only toolchain)",
    long_about = "Validate pipeline.ocblueprint JSON in a role pack or project.\n\
                  Desktop host orchestration uses process_message; this command does not change runtime.\n\
                  Prefer init --pipeline for generated Rust pipeline order docs on new projects."
)]
pub struct BlueprintCli {
    #[command(subcommand)]
    pub command: BlueprintCommand,
}

#[derive(Subcommand, Debug)]
pub enum BlueprintCommand {
    /// Validate `pipeline.ocblueprint` (or `.json`) format and step references
    Validate(BlueprintValidateArgs),
}

#[derive(Parser, Debug)]
pub struct BlueprintValidateArgs {
    /// Blueprint file path
    pub path: PathBuf,

    /// Machine-readable JSON (`{"ok":true}` or `{"ok":false,"errors":[...]}`)
    #[arg(long)]
    pub json: bool,
}

pub fn run(cli: BlueprintCli) -> Result<()> {
    match cli.command {
        BlueprintCommand::Validate(args) => run_validate(args),
    }
}

fn run_validate(args: BlueprintValidateArgs) -> Result<()> {
    let bp = load_blueprint(&args.path)?;
    let report = validate_blueprint(&bp);
    if args.json {
        let body = if report.ok {
            serde_json::json!({ "ok": true })
        } else {
            serde_json::json!({
                "ok": false,
                "errors": report.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
            })
        };
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else if report.ok {
        println!("OK: blueprint valid ({})", args.path.display());
    } else {
        eprintln!("FAIL: blueprint invalid ({})", args.path.display());
        for e in &report.errors {
            eprintln!("  - {}", e.message);
        }
        std::process::exit(1);
    }
    Ok(())
}
