//! `oclive ci` — generate and check GitHub Actions CI.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct CiCli {
    #[command(subcommand)]
    pub command: CiCommands,
}

#[derive(Subcommand, Debug)]
pub enum CiCommands {
    /// Generate `.github/workflows/ci.yml`
    Init(CiInitArgs),
    /// Check whether CI matches the latest template
    Check(CiCheckArgs),
    /// Compute a deterministic domain-aware CI impact plan
    Plan(crate::ci_impact_cmd::CiPlanArgs),
    /// Explain an existing CI impact plan without recomputing it
    Explain(crate::ci_impact_cmd::CiExplainArgs),
}

#[derive(Parser, Debug)]
pub struct CiInitArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser, Debug)]
pub struct CiCheckArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ProjectCiKind {
    KernelServer,
    Library,
}

pub fn run(cli: CiCli) -> Result<()> {
    match cli.command {
        CiCommands::Init(a) => run_init(a),
        CiCommands::Check(a) => run_check(a),
        CiCommands::Plan(a) => crate::ci_impact_cmd::run_plan(a),
        CiCommands::Explain(a) => crate::ci_impact_cmd::run_explain(a),
    }
}

fn run_init(args: CiInitArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let kind = detect_project_kind(&root)?;
    let workflow = root.join(".github/workflows/ci.yml");
    if workflow.is_file() && !args.force {
        bail!(
            "{} already exists; use --force to overwrite",
            workflow.display()
        );
    }
    fs::create_dir_all(workflow.parent().unwrap()).context("mkdir .github/workflows")?;
    let content = render_ci_yaml(kind);
    fs::write(&workflow, content).context("write ci.yml")?;
    println!("Generated {} ({:?})", workflow.display(), kind);
    Ok(())
}

fn run_check(args: CiCheckArgs) -> Result<()> {
    let root = args.path.canonicalize().context("path")?;
    let kind = detect_project_kind(&root)?;
    let workflow = root.join(".github/workflows/ci.yml");
    if !workflow.is_file() {
        bail!("Missing {}; run oclive ci init", workflow.display());
    }
    let current = fs::read_to_string(&workflow)?;
    let expected = render_ci_yaml(kind);
    if current.trim() == expected.trim() {
        println!("✅ CI config matches latest oclive template");
        Ok(())
    } else {
        println!("⚠️ CI config differs from template; run `oclive ci init --force` to update");
        std::process::exit(1);
    }
}

fn detect_project_kind(root: &Path) -> Result<ProjectCiKind> {
    let cargo = root.join("Cargo.toml");
    if !cargo.is_file() {
        bail!("Missing Cargo.toml");
    }
    let raw = fs::read_to_string(&cargo)?;
    let v: toml::Value = toml::from_str(&raw)?;
    let has_main =
        root.join("src/main.rs").is_file() || root.join("src/main_monolith.rs").is_file();
    let bins = v
        .get("bin")
        .and_then(|b| b.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if has_main || bins {
        Ok(ProjectCiKind::KernelServer)
    } else {
        Ok(ProjectCiKind::Library)
    }
}

fn render_ci_yaml(kind: ProjectCiKind) -> String {
    let oocp_job = if kind == ProjectCiKind::KernelServer {
        r#"
  bench-regression:
    runs-on: ubuntu-latest
    needs: [build-test]
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: dtolnay/rust-toolchain@f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f # master (explicit stable)
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2
      - name: bench regression gate
        run: |
          cargo build -p oclive-cli --release 2>/dev/null || cargo build -p oclive-cli
          cargo run -p oclive-cli -- --experimental bench --release -o . --runs 3 --save || true
          cargo run -p oclive-cli -- --experimental bench --release -o . --regression --runs 5 || true
        continue-on-error: true

  oocp:
    runs-on: ubuntu-latest
    needs: [build-test]
    if: false
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4
        with:
          node-version: "22"
      - uses: dtolnay/rust-toolchain@f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f # master (explicit stable)
        with:
          toolchain: stable
      - name: OOCP test suite (enable when kernel linked to oclivenewnew)
        run: echo "skipped — link --kernel-source in scaffold to enable"
"#
    } else {
        ""
    };

    let audit_job = r#"
  cargo-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: dtolnay/rust-toolchain@f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f # master (explicit stable)
        with:
          toolchain: stable
      - name: rustup update
        run: rustup update stable
      - name: Install cargo-audit
        run: cargo install cargo-audit --version 0.22.1 --locked
      - name: cargo audit
        run: cargo audit

  cargo-deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: dtolnay/rust-toolchain@f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f # master (explicit stable)
        with:
          toolchain: stable
      - uses: EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25 # v2
        with:
          command: check
          arguments: licenses bans

"#;

    let extra_steps = if kind == ProjectCiKind::KernelServer {
        r#"
      - name: oclive test
        run: cargo run -p oclive-cli -- --experimental test -o . --skip-oocp
        continue-on-error: true
"#
    } else {
        ""
    };

    format!(
        r#"# Generated by oclive ci init — review before enabling OOCP / registry secrets.
name: CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]
  workflow_dispatch:

env:
  OCLIVE_REGISTRY_URL: ${{{{ vars.OCLIVE_REGISTRY_URL }}}}
  OCLIVE_MARKET_INDEX_URL: ${{{{ vars.OCLIVE_MARKET_INDEX_URL }}}}
  OCLIVE_PLUGIN_INDEX_URL: ${{{{ vars.OCLIVE_PLUGIN_INDEX_URL }}}}
  OCLIVE_HTTP_API_MOCK_LLM: "1"

jobs:
  build-test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{{{ matrix.os }}}}
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: dtolnay/rust-toolchain@f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f # master (explicit stable)
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2
      - name: cargo fmt
        run: cargo fmt --all -- --check
      - name: cargo clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: cargo build
        run: cargo build --release
      - name: cargo test
        run: cargo test
{extra_steps}
{audit_job}
{oocp_job}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::{render_ci_yaml, ProjectCiKind};
    use crate::lint_audit_ci::inspect_audit_ci;

    #[test]
    fn generated_ci_pins_actions_and_preserves_stable_toolchain() {
        for kind in [ProjectCiKind::Library, ProjectCiKind::KernelServer] {
            let workflow = render_ci_yaml(kind);
            let document: serde_yaml_ng::Value =
                serde_yaml_ng::from_str(&workflow).expect("generated workflow should parse");
            let mut action_count = 0;
            for job in document["jobs"].as_mapping().expect("jobs").values() {
                for step in job["steps"].as_sequence().expect("steps") {
                    let Some(reference) = step["uses"].as_str() else {
                        continue;
                    };
                    action_count += 1;
                    let (_, revision) =
                        reference.split_once('@').expect("external action revision");
                    assert_eq!(
                        revision.len(),
                        40,
                        "mutable action in {kind:?}: {reference}"
                    );
                    assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
                    if reference.starts_with("dtolnay/rust-toolchain@") {
                        assert_eq!(step["with"]["toolchain"].as_str(), Some("stable"));
                    }
                }
            }
            assert!(action_count > 0, "generated workflow must contain actions");
        }
    }

    #[test]
    fn generated_kernel_ci_uses_the_global_experimental_gate() {
        let workflow = render_ci_yaml(ProjectCiKind::KernelServer);
        assert!(workflow.contains("-- --experimental bench"));
        assert!(workflow.contains("-- --experimental test"));
        assert!(!workflow.contains("oclive-cli -- bench"));
        assert!(!workflow.contains("oclive-cli -- test"));
    }

    #[test]
    fn generated_kernel_ci_uses_current_node_and_required_audit_policy() {
        let workflow = render_ci_yaml(ProjectCiKind::KernelServer);
        assert!(workflow.contains("node-version: \"22\""));
        assert!(!workflow.contains("node-version: \"20\""));
        assert!(!workflow.contains("loom:"));
        assert!(!workflow.contains("loom_concurrency"));

        let audit = inspect_audit_ci(&workflow).expect("generated workflow should parse");
        assert_eq!(audit.owners, ["cargo-audit"]);
        assert!(audit.soft_owners.is_empty());
    }
}
