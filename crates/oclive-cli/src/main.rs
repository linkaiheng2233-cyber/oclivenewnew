//! oclive-cli — 官方内核项目脚手架入口。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used)
)]

mod bench_cmd;
mod bench_metrics;
mod bench_stress;
mod blueprint;
mod cargo_hints;
mod doctor_cmd;
mod blueprint_cmd;
mod build_cmd;
mod dashboard_cmd;
mod dev_cmd;
mod plugin_cmd;
mod plugin_ext;
mod generator;
mod init;
mod init_from_existing;
mod init_bench;
mod project_introspect;
mod interactive;
mod learn_cmd;
mod lint_cmd;
mod pipeline;
mod profile_cmd;
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
mod registry_remote;
mod market_index;
mod market_cmd;
mod collab_cmd;
mod config;
mod config_cmd;
mod ci_cmd;
mod template_cmd;
mod role_pack;
mod test_cmd;
mod test_ci_parity;
mod kernel_cmd;

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
    /// [experimental/legacy] 蓝图（pipeline.ocblueprint）校验
    Blueprint(blueprint_cmd::BlueprintCli),
    /// 环境诊断（Rust / 磁盘 / Ollama / 网络等）
    Doctor(doctor_cmd::DoctorArgs),
    /// 插件脚手架（directory / remote）
    Plugin(plugin_cmd::PluginCli),
    /// 本地内核工程注册表
    Registry(registry_cmd::RegistryCli),
    /// 多内核 compose 编排
    Compose(compose_cmd::ComposeCli),
    /// [deprecated] 发布模板包 — 请用 `oclive template pack`
    Publish(publish_cmd::PublishArgs),
    /// 逐步骤调试 process_message
    Debug(debug_cmd::DebugArgs),
    /// 本地 Web 仪表盘（默认 127.0.0.1:8420）
    Dashboard(dashboard_cmd::DashboardArgs),
    /// 新用户交互式教程
    Learn(learn_cmd::LearnArgs),
    /// 内核工程回归测试
    Test(test_cmd::TestArgs),
    /// 内核工程静态健康检查
    Lint(lint_cmd::LintArgs),
    /// 内核性能画像
    Profile(profile_cmd::ProfileArgs),
    /// 插件 / 模板市场浏览与安装
    Market(market_cmd::MarketCli),
    /// 角色包 Git 协作
    Collab(collab_cmd::CollabCli),
    /// 全局 / 工程级配置（~/.oclive/config.toml）
    Config(config_cmd::ConfigCli),
    /// 生成 GitHub Actions CI
    Ci(ci_cmd::CiCli),
    /// 模板打包与反向生成（`pack` / `create`）
    Template(template_cmd::TemplateCli),
    /// Kernel runtime dependency info
    Kernel(kernel_cmd::KernelCli),
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
        Commands::Dashboard(args) => dashboard_cmd::run(args),
        Commands::Learn(args) => learn_cmd::run(args),
        Commands::Test(args) => test_cmd::run(args),
        Commands::Lint(args) => lint_cmd::run(args),
        Commands::Profile(args) => profile_cmd::run(args),
        Commands::Market(cli) => market_cmd::run(cli),
        Commands::Collab(cli) => collab_cmd::run(cli),
        Commands::Config(cli) => config_cmd::run(cli),
        Commands::Ci(cli) => ci_cmd::run(cli),
        Commands::Template(cli) => template_cmd::run(cli),
        Commands::Kernel(cli) => kernel_cmd::run(cli),
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
