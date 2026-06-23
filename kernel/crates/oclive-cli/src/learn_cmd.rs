//! `oclive learn` — interactive tutorial for new users.

use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
pub struct LearnArgs {
    #[arg(short = 'o', long, default_value = "./oclive-learn-demo")]
    pub output: PathBuf,
}

pub fn run(args: LearnArgs) -> Result<()> {
    println!("=== oclive interactive tutorial (5 steps) ===\n");
    step(
        1,
        "Environment check",
        "Runs `oclive doctor` for Rust, disk, and optional Ollama.",
    )?;
    run_oclive(&["doctor"])?;
    pause()?;

    step(
        2,
        "Choose a template",
        "Recommended for beginners: `dialogue-only` (full preset, default role pack).",
    )?;
    println!(
        "  · robot-soul — doll / embedded + Monolith\n  · dialogue-only — dialogue service\n  · headless-api — API without role pack\n"
    );
    pause()?;

    step(
        3,
        "Generate project",
        "Runs `oclive init --non-interactive --template dialogue-only`.",
    )?;
    if args.output.exists() {
        println!("Output directory already exists: {}", args.output.display());
    } else {
        let out = args.output.to_string_lossy();
        run_oclive(&[
            "init",
            "--non-interactive",
            "--quiet",
            "--template",
            "dialogue-only",
            "-o",
            &out,
            "--project-name",
            "learn-demo",
        ])?;
    }
    pause()?;

    step(
        4,
        "Build",
        "Runs `cargo build` in the project directory (first build may be slow).",
    )?;
    let st = Command::new("cargo")
        .arg("build")
        .current_dir(&args.output)
        .status();
    match st {
        Ok(s) if s.success() => println!("✅ cargo build succeeded"),
        Ok(s) => {
            println!("❌ cargo build exit code {:?}", s.code());
            println!(
                "Try: `oclive doctor`; or `oclive init --kernel-source <oclivenewnew-root>` for the full kernel."
            );
        }
        Err(e) => println!("Cannot start cargo: {e}"),
    }
    pause()?;

    step(
        5,
        "First message",
        "With `--kernel-source` and HTTP API running, test with curl:",
    )?;
    println!(
        r#"
  $env:OCLIVE_HTTP_API_MOCK_LLM = "1"
  cargo run --release -- --api --port 8421
  curl -X POST http://127.0.0.1:8421/chat -H "Content-Type: application/json" -d '{{"message":"hello","role_id":"default"}}'
"#
    );
    println!(
        "\n🎉 Tutorial complete. Next: `oclive bench --release -o {}` (add Monolith via init --monolith when needed)",
        args.output.display()
    );
    Ok(())
}

fn step(n: u32, title: &str, detail: &str) -> Result<()> {
    println!("【Step {n}/5】{title}\n{detail}\n");
    Ok(())
}

fn pause() -> Result<()> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Press Enter to continue")
        .default(true)
        .interact()?;
    println!();
    Ok(())
}

fn run_oclive(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    let st = Command::new(exe).args(args).status()?;
    if !st.success() {
        println!(
            "⚠ Command did not succeed (exit {:?}); retry: oclive {}",
            st.code(),
            args.join(" ")
        );
    }
    Ok(())
}
