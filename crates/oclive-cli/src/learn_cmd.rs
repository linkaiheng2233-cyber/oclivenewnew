//! `oclive learn` — 新用户交互式教程。

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
    println!("=== oclive 交互式教程（5 步）===\n");
    step(1, "环境检查", "将运行 `oclive doctor` 检查 Rust、磁盘与可选 Ollama。")?;
    run_oclive(&["doctor"])?;
    pause()?;

    step(2, "选择模板", "推荐新手使用 `dialogue-only`（full 预设、通用角色包）。")?;
    println!("  · robot-soul — 玩偶 / 嵌入式 + Monolith\n  · dialogue-only — 纯对话服务\n  · headless-api — 无角色包 API\n");
    pause()?;

    step(3, "生成工程", "将执行 `oclive init --non-interactive --template dialogue-only`。")?;
    if args.output.exists() {
        println!("输出目录已存在: {}", args.output.display());
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

    step(4, "编译", "在项目目录执行 `cargo build`（首次可能较慢）。")?;
    let st = Command::new("cargo")
        .arg("build")
        .current_dir(&args.output)
        .status();
    match st {
        Ok(s) if s.success() => println!("✅ cargo build 成功"),
        Ok(s) => {
            println!("❌ cargo build 退出码 {:?}", s.code());
            println!("建议: 运行 `oclive doctor`；或 `oclive init --kernel-source <oclivenewnew根>` 接入完整内核。");
        }
        Err(e) => println!("无法启动 cargo: {e}"),
    }
    pause()?;

    step(5, "第一条消息", "若已 `--kernel-source` 并启动 HTTP API，可用 curl 测试：")?;
    println!(
        r#"
  $env:OCLIVE_HTTP_API_MOCK_LLM = "1"
  cargo run --release -- --api --port 8421
  curl -X POST http://127.0.0.1:8421/chat -H "Content-Type: application/json" -d '{{"message":"你好","role_id":"default"}}'
"#
    );
    println!("\n🎉 教程完成。下一步: `oclive bench --release -o {}`（需 Monolith 时加 --monolith init）", args.output.display());
    Ok(())
}

fn step(n: u32, title: &str, detail: &str) -> Result<()> {
    println!("【步骤 {n}/5】{title}\n{detail}\n");
    Ok(())
}

fn pause() -> Result<()> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("按 Enter 继续")
        .default(true)
        .interact()?;
    println!();
    Ok(())
}

fn run_oclive(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    let st = Command::new(exe).args(args).status()?;
    if !st.success() {
        println!("⚠ 命令未成功（退出码 {:?}）；可单独重试: oclive {}", st.code(), args.join(" "));
    }
    Ok(())
}
