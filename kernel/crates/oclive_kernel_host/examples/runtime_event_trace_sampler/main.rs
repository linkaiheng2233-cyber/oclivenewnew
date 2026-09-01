//! Generate synthetic, privacy-minimized Runtime Event trace shadow samples for design review.
//!
//! This developer utility writes artifacts under `target/`. It does not expose a production read
//! API and never feeds trace records back into Event Ring, Prompt, memory, or turn orchestration.

mod contract;
mod events;
mod evidence;
mod faults;
mod loads;
mod probe;
mod soak;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context};
use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::{OcliveKernel, OcliveKernelConfig};

use contract::{parse_and_validate, AFTER_RESTART, BEFORE_RESTART};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mode = argument_value_or("--mode", "structural")?;
    let contract_path = PathBuf::from(argument_value("--contract")?);
    let output_dir = resolve_output_dir(&argument_value("--output-dir")?)?;
    let source_commit = argument_value("--source-commit")?;
    let source_worktree_dirty = argument_value("--source-worktree-dirty")?
        .parse::<bool>()
        .context("--source-worktree-dirty must be true or false")?;

    let contract_text = std::fs::read_to_string(&contract_path)
        .with_context(|| format!("read sample contract {}", contract_path.display()))?;
    let scenario_contract = repository_relative_path(&contract_path)?;

    match mode.as_str() {
        "structural" => {
            run_structural(
                &contract_text,
                scenario_contract,
                &output_dir,
                source_commit,
                source_worktree_dirty,
            )
            .await
        }
        "fault" => {
            faults::run(
                &contract_text,
                scenario_contract,
                &output_dir,
                source_commit,
                source_worktree_dirty,
            )
            .await
        }
        "load" => {
            loads::run(
                &contract_text,
                scenario_contract,
                &output_dir,
                source_commit,
                source_worktree_dirty,
            )
            .await
        }
        "soak" => {
            soak::run(
                &contract_text,
                scenario_contract,
                &output_dir,
                source_commit,
                source_worktree_dirty,
            )
            .await
        }
        other => bail!("unsupported sampler mode {other}"),
    }
}

async fn run_structural(
    contract_text: &str,
    scenario_contract: String,
    output_dir: &Path,
    source_commit: String,
    source_worktree_dirty: bool,
) -> anyhow::Result<()> {
    let contract = parse_and_validate(contract_text)?;

    let temp = tempfile::tempdir().context("create sample kernel directory")?;
    let trace_path = temp.path().join("runtime-event-trace.sqlite3");
    let config = sample_config(temp.path(), &trace_path)?;

    let first_kernel = build_kernel(config.clone()).await?;
    events::execute_phase(&first_kernel, &contract, BEFORE_RESTART).await?;
    first_kernel.shutdown().await;

    let second_kernel = build_kernel(config).await?;
    events::execute_phase(&second_kernel, &contract, AFTER_RESTART).await?;
    second_kernel.shutdown().await;

    let counts = evidence::collect_and_write(
        &contract,
        scenario_contract,
        contract_text,
        &trace_path,
        output_dir,
        source_commit,
        source_worktree_dirty,
    )
    .await?;
    println!(
        "sampler: PASS ({} scenarios; {} records; synthetic-only)",
        counts.scenarios, counts.records
    );
    Ok(())
}

pub(crate) fn sample_config(
    app_data_dir: &Path,
    trace_path: &Path,
) -> anyhow::Result<OcliveKernelConfig> {
    let roles_dir =
        oclive_kernel_runtime::chat_pro_roles_dir(&[PathBuf::from(env!("CARGO_MANIFEST_DIR"))])
            .context("locate Chat Pro role fixtures")?;
    Ok(OcliveKernelConfig::new(app_data_dir, roles_dir).with_runtime_event_trace_path(trace_path))
}

pub(crate) async fn build_kernel(config: OcliveKernelConfig) -> anyhow::Result<OcliveKernel> {
    OcliveKernel::builder(config)
        .with_host_profile(HostProfile::default())
        .with_llm_client(Arc::new(MockLlmClient {
            reply: "unused synthetic sampler reply".into(),
        }))
        .build()
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn argument_value_or(name: &str, fallback: &str) -> anyhow::Result<String> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let Some(index) = arguments.iter().position(|argument| argument == name) else {
        return Ok(fallback.into());
    };
    arguments
        .get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .with_context(|| format!("{name} requires a value"))
}

fn argument_value(name: &str) -> anyhow::Result<String> {
    let arguments = std::env::args().collect::<Vec<_>>();
    let Some(index) = arguments.iter().position(|argument| argument == name) else {
        bail!("missing required argument {name}");
    };
    arguments
        .get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .with_context(|| format!("{name} requires a value"))
}

fn resolve_output_dir(value: &str) -> anyhow::Result<PathBuf> {
    let repo_root = std::env::current_dir().context("resolve repository root")?;
    let target_root = repo_root.join("target");
    std::fs::create_dir_all(&target_root).context("create repository target directory")?;
    let output = if Path::new(value).is_absolute() {
        PathBuf::from(value)
    } else {
        repo_root.join(value)
    };
    std::fs::create_dir_all(&output).context("create requested sample output directory")?;
    let target_root = std::fs::canonicalize(target_root).context("canonicalize target root")?;
    let output = std::fs::canonicalize(output).context("canonicalize sample output")?;
    if output != target_root && !output.starts_with(&target_root) {
        bail!("sample output must stay under repository target/");
    }
    Ok(output)
}

fn repository_relative_path(path: &Path) -> anyhow::Result<String> {
    let repo_root = std::fs::canonicalize(
        std::env::current_dir().context("resolve repository root for sample contract")?,
    )
    .context("canonicalize repository root for sample contract")?;
    let path = std::fs::canonicalize(path)
        .with_context(|| format!("canonicalize sample contract {}", path.display()))?;
    let relative = path
        .strip_prefix(&repo_root)
        .context("sample contract must stay inside the repository")?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
