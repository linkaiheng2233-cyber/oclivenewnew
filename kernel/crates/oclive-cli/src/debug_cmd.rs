//! `oclive debug` — launch the kernel and display the `OCLIVE_DEBUG_TRACE` step summary.

use anyhow::{bail, Context, Result};
use clap::Parser;
use oclive_kernel_runtime::DEFAULT_API_PORT;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct DebugArgs {
    /// Kernel project root (contains Cargo.toml)
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,

    /// Show only the given step (e.g. load_recent_context, user_emotion_analyze)
    #[arg(long)]
    pub step: Option<String>,

    /// Emit full JSON trace to stdout
    #[arg(long)]
    pub json: bool,

    /// HTTP API port
    #[arg(long, default_value_t = DEFAULT_API_PORT)]
    pub port: u16,

    /// Test message
    #[arg(long, default_value = "hello")]
    pub message: String,
}

const TRACE_PREFIX: &str = "OCLIVE_DEBUG_TRACE ";

pub fn run(args: DebugArgs) -> Result<()> {
    let root = args
        .path
        .canonicalize()
        .with_context(|| format!("path {}", args.path.display()))?;
    if !root.join("Cargo.toml").is_file() {
        bail!("{} missing Cargo.toml", root.display());
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
        "[oclive debug] starting {} (port {}, MOCK_LLM=1)…",
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
    let resp = crate::http_client::post(&chat_url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());
    match resp {
        Ok(r) => {
            let text = r.into_string().unwrap_or_default();
            if args.json {
                println!("{text}");
            } else {
                eprintln!("[oclive debug] reply length: {} chars", text.len());
            }
        }
        Err(e) => {
            eprintln!("[oclive debug] chat request failed (kernel may still be compiling): {e}")
        }
    }

    let _ = child.kill();
    let _ = reader_thread.join();
    let lines = trace_lines.lock().unwrap().clone();
    print_traces(&lines, args.step.as_deref(), args.json)?;
    Ok(())
}

fn print_traces(lines: &[String], step_filter: Option<&str>, full_json: bool) -> Result<()> {
    if lines.is_empty() {
        eprintln!("[oclive debug] no OCLIVE_DEBUG_TRACE lines captured. Use `oclive init --kernel-source <oclivenewnew-root>` for the full kernel.");
        return Ok(());
    }
    println!("\n—— process_message debug trace ——");
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
