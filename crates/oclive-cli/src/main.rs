//! oclive-cli — 官方内核项目脚手架入口。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used)
)]

mod bench_cmd;
mod bench_metrics;
mod blueprint;
mod cargo_hints;
mod doctor_cmd;
mod blueprint_cmd;
mod build_cmd;
mod dev_cmd;
mod plugin_cmd;
mod generator;
mod init;
mod init_bench;
mod interactive;
mod template_catalog;
mod monolith_codegen;
mod monolith_config;
mod pack_cmd;
mod compose_cmd;
mod debug_cmd;
mod init_tui;
mod publish_cmd;
mod registry;
mod registry_cmd;
mod role_pack;

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
    /// 蓝图（pipeline.ocblueprint）读取与校验
    Blueprint(blueprint_cmd::BlueprintCli),
    /// 环境诊断（Rust / 磁盘 / Ollama / 网络等）
    Doctor(doctor_cmd::DoctorArgs),
    /// 插件脚手架（directory / remote）
    Plugin(plugin_cmd::PluginCli),
    /// 本地内核工程注册表
    Registry(registry_cmd::RegistryCli),
    /// 多内核 compose 编排
    Compose(compose_cmd::ComposeCli),
    /// 发布模板包
    Publish(publish_cmd::PublishArgs),
    /// 逐步骤调试 process_message
    Debug(debug_cmd::DebugArgs),
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
        Commands::Blueprint(cli) => blueprint_cmd::run(cli),
        Commands::Doctor(args) => doctor_cmd::run(args),
        Commands::Plugin(cli) => plugin_cmd::run(cli),
        Commands::Registry(cli) => registry_cmd::run(cli),
        Commands::Compose(cli) => compose_cmd::run(cli),
        Commands::Publish(args) => publish_cmd::run(args),
        Commands::Debug(args) => debug_cmd::run(args),
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
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("robot-soul"),
            "init --help footer should mention --template"
        );
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("--list-templates"),
            "init --help footer should mention --list-templates"
        );
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("--monolith-bench-preset"),
            "init --help footer should mention --monolith-bench-preset"
        );
    }
}
