//! oclive-cli — 官方内核项目脚手架入口。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used)
)]

mod bench_cmd;
mod build_cmd;
mod dev_cmd;
mod generator;
mod init;
mod interactive;
mod monolith_codegen;
mod monolith_config;
mod pack_cmd;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oclive-cli",
    version,
    about = "Oclive official kernel project scaffolding"
)]
struct Cli {
    /// 日志详细程度：累计 `-v` 提升（0=INFO, 1=DEBUG, 2+=TRACE）；可被 `RUST_LOG` 覆盖
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

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
    /// 监听角色包目录变更（开发用；生产路径不使用）
    Dev(dev_cmd::DevArgs),
    /// 角色包：校验、创建、打包（.oclivepack）
    Pack(pack_cmd::PackArgs),
}

fn init_tracing(verbose: u8) {
    use tracing_subscriber::EnvFilter;
    let default = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    match cli.command {
        Commands::Init(args) => init::run(args),
        Commands::Build(args) => build_cmd::run(args),
        Commands::Bench(args) => bench_cmd::run(args),
        Commands::Dev(args) => dev_cmd::run(args),
        Commands::Pack(args) => pack_cmd::run_pack(args),
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
