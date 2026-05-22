//! oclive-cli — 官方内核项目脚手架入口。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod commands;

pub use commands::bench as bench_cmd;
pub use commands::init;
pub use commands::lint as lint_cmd;
mod bench_cold_start;
mod bench_equivalence;
mod bench_metrics;
mod bench_soak;
mod bench_stress;
mod blueprint;
mod blueprint_v3_init;
mod blueprint_cmd;
mod build_cmd;
mod cargo_hints;
mod ci_cmd;
mod cli_english_init;
mod collab_cmd;
mod completions_cmd;
mod compose_cmd;
mod config;
mod config_cmd;
mod dashboard_cmd;
mod debug_cmd;
mod dev_cmd;
mod doctor_blueprint;
mod doctor_cmd;
mod doctor_kernel_contracts;
mod env_probe;
mod doctor_sbom;
mod explain_cmd;
mod explain_dual_core;
mod generator;
mod init_bench;
mod init_check;
mod init_from_existing;
mod init_plan;
mod init_tui;
mod interactive;
mod kernel_cmd;
mod learn_cmd;
mod lint_audit_ci;
mod lint_report;
mod market_cmd;
mod market_index;
mod monolith_codegen;
mod monolith_config;
mod pack_cmd;
mod pipeline;
mod plugin_cmd;
mod plugin_ext;
mod plugin_manage_cmd;
mod plugin_manage_tui;
mod profile_cmd;
mod project_introspect;
mod publish_cmd;
mod registry;
mod registry_cmd;
mod registry_remote;
mod role_pack;
mod template_catalog;
mod template_cmd;
mod test_ci_parity;
mod test_cmd;
mod test_coverage;
mod test_equivalence_check;
mod test_loom;
mod test_miri;
mod test_oocp;
mod test_json_report;
mod test_report;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oclive-cli",
    version,
    about = "Oclive official kernel project scaffolding"
)]
pub struct Cli {
    /// Verbosity: `-v` count or `RUST_LOG` (0=INFO, 1=DEBUG, 2+=TRACE)
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a kernel project (interactive or scripted)
    #[command(after_long_help = commands::init::PRESET_MATRIX_HELP)]
    Init(commands::init::InitArgs),
    /// Regenerate Monolith artifacts and build standard + monolith binaries
    Build(build_cmd::BuildArgs),
    /// Benchmark standard vs monolith binaries (JSON report)
    Bench(commands::bench::BenchArgs),
    /// Watch role pack manifest/settings changes
    Dev(dev_cmd::DevArgs),
    /// Role pack validate, create, publish (.oclivepack)
    Pack(pack_cmd::PackArgs),
    /// [experimental/legacy] Validate pipeline.ocblueprint JSON
    Blueprint(blueprint_cmd::BlueprintCli),
    /// Environment diagnostics (Rust, disk, Ollama, network)
    Doctor(doctor_cmd::DoctorArgs),
    /// Plugin scaffolds (directory / remote)
    Plugin(plugin_cmd::PluginCli),
    /// Local kernel project registry
    Registry(registry_cmd::RegistryCli),
    /// Multi-kernel compose orchestration
    Compose(compose_cmd::ComposeCli),
    /// Step trace debug for process_message
    Debug(debug_cmd::DebugArgs),
    /// Local web dashboard (default 127.0.0.1:8420)
    Dashboard(dashboard_cmd::DashboardArgs),
    /// Interactive onboarding tutorial
    Learn(learn_cmd::LearnArgs),
    /// Project regression checks
    Test(test_cmd::TestArgs),
    /// Static project health lint
    Lint(commands::lint::LintArgs),
    /// Build size and dependency profile
    Profile(profile_cmd::ProfileArgs),
    /// Browse and install from market index
    Market(market_cmd::MarketCli),
    /// Role pack Git collaboration helpers
    Collab(collab_cmd::CollabCli),
    /// Global / project config (~/.oclive/config.toml)
    Config(config_cmd::ConfigCli),
    /// Generate or check GitHub Actions CI
    Ci(ci_cmd::CiCli),
    /// Template pack / create from existing project
    Template(template_cmd::TemplateCli),
    /// Kernel runtime dependency info
    Kernel(kernel_cmd::KernelCli),
    /// Explain a kernel error code (from ERROR_CODES.md)
    Explain(explain_cmd::ExplainArgs),
    /// Shell completion scripts (install to your shell profile)
    Completions(completions_cmd::CompletionsArgs),
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
        Commands::Init(args) => commands::init::run(args),
        Commands::Build(args) => build_cmd::run(args),
        Commands::Bench(args) => commands::bench::run(args),
        Commands::Dev(args) => dev_cmd::run(args),
        Commands::Pack(args) => pack_cmd::run_pack(args),
        Commands::Blueprint(cli) => blueprint_cmd::run(cli),
        Commands::Doctor(args) => doctor_cmd::run(args),
        Commands::Plugin(cli) => plugin_cmd::run(cli),
        Commands::Registry(cli) => registry_cmd::run(cli),
        Commands::Compose(cli) => compose_cmd::run(cli),
        Commands::Debug(args) => debug_cmd::run(args),
        Commands::Dashboard(args) => dashboard_cmd::run(args),
        Commands::Learn(args) => learn_cmd::run(args),
        Commands::Test(args) => test_cmd::run(args),
        Commands::Lint(args) => commands::lint::run(args),
        Commands::Profile(args) => profile_cmd::run(args),
        Commands::Market(cli) => market_cmd::run(cli),
        Commands::Collab(cli) => collab_cmd::run(cli),
        Commands::Config(cli) => config_cmd::run(cli),
        Commands::Ci(cli) => ci_cmd::run(cli),
        Commands::Template(cli) => template_cmd::run(cli),
        Commands::Kernel(cli) => kernel_cmd::run(cli),
        Commands::Explain(args) => explain_cmd::run(args),
        Commands::Completions(args) => completions_cmd::run(args),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_after_long_help_mentions_monolith_rfc() {
        assert!(
            crate::commands::init::PRESET_MATRIX_HELP.contains("RFC_OCLIVE_MONOLITH_MODE.md"),
            "init --help footer should point to Monolith RFC"
        );
        assert!(
            crate::commands::init::PRESET_MATRIX_HELP.contains("--monolith"),
            "init --help footer should mention --monolith"
        );
        assert!(
            crate::commands::init::PRESET_MATRIX_HELP.contains("robot-soul"),
            "init --help footer should mention --template"
        );
        assert!(
            crate::init::PRESET_MATRIX_HELP.contains("--list-templates"),
            "init --help footer should mention --list-templates"
        );
        assert!(
            crate::commands::init::PRESET_MATRIX_HELP.contains("--monolith-bench-preset"),
            "init --help footer should mention --monolith-bench-preset"
        );
    }
}
