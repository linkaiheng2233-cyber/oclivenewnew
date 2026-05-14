//! oclive-cli — 官方内核项目脚手架入口。

mod generator;
mod init;
mod interactive;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oclive-cli",
    version,
    about = "Oclive official kernel project scaffolding"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 交互或非交互创建内核项目骨架
    Init(init::InitArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init::run(args),
    }
}
