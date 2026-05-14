//! oclive-cli — 官方内核项目脚手架入口。

mod generator;
mod init;
mod interactive;
mod monolith_codegen;

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
    #[command(after_long_help = init::PRESET_MATRIX_HELP)]
    Init(init::InitArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init::run(args),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_after_long_help_mentions_monolith_rfc() {
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("RFC_OCLIVE_MONOLITH_MODE.md"),
            "init --help footer should point to Monolith RFC"
        );
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("--monolith"),
            "init --help footer should mention --monolith"
        );
    }
}
