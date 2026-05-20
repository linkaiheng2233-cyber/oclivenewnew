//! `oclive blueprint` — 蓝图读取与校验。

use crate::blueprint::{load_blueprint, validate_blueprint};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct BlueprintCli {
    #[command(subcommand)]
    pub command: BlueprintCommand,
}

#[derive(Subcommand, Debug)]
pub enum BlueprintCommand {
    /// 校验 `pipeline.ocblueprint`（或 `.json`）格式与步骤引用
    Validate(BlueprintValidateArgs),
}

#[derive(Parser, Debug)]
pub struct BlueprintValidateArgs {
    /// 蓝图文件路径
    pub path: PathBuf,

    /// 机器可读 JSON 输出（`{"ok":true}` 或 `{"ok":false,"errors":[...]}`）
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
