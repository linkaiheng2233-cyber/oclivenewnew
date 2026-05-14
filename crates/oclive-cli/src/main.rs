//! oclive-cli — 官方内核项目脚手架入口。

mod bench_cmd;
mod build_cmd;
mod generator;
mod init;
mod interactive;
mod monolith_codegen;
mod monolith_config;

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
    /// 读取 monolith.toml，写入 vendor 与 process_message_monolith.rs；默认执行标准 + Monolith 两次 cargo build
    Build(build_cmd::BuildArgs),
    /// 对比标准与 Monolith 二进制的子进程耗时（JSON 报告）
    Bench(bench_cmd::BenchArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => init::run(args),
        Commands::Build(args) => build_cmd::run(args),
        Commands::Bench(args) => bench_cmd::run(args),
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
