//! `oclive debug` — 启动内核并展示 `OCLIVE_DEBUG_TRACE` 步骤摘要。

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct DebugArgs {
    /// 内核工程根（含 Cargo.toml）
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// 仅展示指定步骤（如 load_recent_context、user_emotion_analyze）
    #[arg(long)]
    pub step: Option<String>,

    /// 输出完整 JSON 追踪到 stdout
    #[arg(long)]
    pub json: bool,

    /// HTTP API 端口
    #[arg(long, default_value_t = 8420)]
    pub port: u16,

    /// 测试消息
    #[arg(long, default_value = "你好")]
    pub message: String,
}

const TRACE_PREFIX: &str = "OCLIVE_DEBUG_TRACE ";

pub fn run(args: DebugArgs) -> Result<()> {
    let root = args
        .path
        .canonicalize()
        .with_context(|| format!("path {}", args.path.display()))?;
    if !root.join("Cargo.toml").is_file() {
        bail!("{} 缺少 Cargo.toml", root.display());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--release")
        .arg("--")
        .arg("--api")
        .arg("--port")
        .arg(args.port.to_string())
        .current_dir(&root)
        .env("OCLIVE_DEBUG_TRACE", "1")
        .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    eprintln!(
        "[oclive debug] 启动 {} (port {}, MOCK_LLM=1)…",
        root.display(),
        args.port
    );
    let mut child = cmd.spawn().context("spawn kernel")?;

    let stderr = child.stderr.take().context("stderr")?;
    let trace_lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let lines_cap = trace_lines.clone();
    let reader_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(TRACE_PREFIX) {
                lines_cap.lock().unwrap().push(line);
            } else {
                eprint!("[kernel] {line}");
            }
        }
    });

    std::thread::sleep(Duration::from_secs(8));

    let chat_url = format!("http://127.0.0.1:{}/chat", args.port);
    let body = serde_json::json!({
        "message": args.message,
        "role_id": "default",
        "scene_id": "default"
    });
    eprintln!("[oclive debug] POST {chat_url}");
    let resp = ureq::post(&chat_url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());
    match resp {
        Ok(r) => {
            let text = r.into_string().unwrap_or_default();
            if args.json {
                println!("{text}");
            } else {
                eprintln!("[oclive debug] 回复长度: {} 字符", text.len());
            }
        }
        Err(e) => eprintln!("[oclive debug] chat 请求失败（内核可能仍在编译）: {e}"),
    }

    let _ = child.kill();
    let _ = reader_thread.join();
    let lines = trace_lines.lock().unwrap().clone();
    print_traces(&lines, args.step.as_deref(), args.json)?;
    Ok(())
}

fn print_traces(lines: &[String], step_filter: Option<&str>, full_json: bool) -> Result<()> {
    if lines.is_empty() {
        eprintln!("[oclive debug] 未捕获到 OCLIVE_DEBUG_TRACE 行。请使用 `oclive init --kernel-source <oclivenewnew根>` 接入完整内核。");
        return Ok(());
    }
    println!("\n—— process_message 调试追踪 ——");
    for line in lines {
        let payload = line.trim_start_matches(TRACE_PREFIX);
        let v: Value = serde_json::from_str(payload).unwrap_or(Value::String(payload.into()));
        let step = v.get("step").and_then(|s| s.as_str()).unwrap_or("?");
        if let Some(f) = step_filter {
            if step != f {
                continue;
            }
        }
        if full_json {
            println!("{}", serde_json::to_string_pretty(&v)?);
        } else {
            let summary = summarize_step(&v);
            println!("  [{step}] {summary}");
        }
    }
    Ok(())
}

fn summarize_step(v: &Value) -> String {
    let inp = v.get("input").cloned().unwrap_or(Value::Null);
    let out = v.get("output").cloned().unwrap_or(Value::Null);
    let in_len = inp.to_string().len().min(120);
    let out_len = out.to_string().len().min(120);
    format!("in≈{in_len}B out≈{out_len}B")
}
