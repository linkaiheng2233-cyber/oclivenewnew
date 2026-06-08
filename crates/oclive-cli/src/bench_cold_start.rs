//! `bench --cold-start` — measure first-response latency after process spawn.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::bench_cmd::BenchArgs;
use oclive_kernel_runtime::DEFAULT_API_PORT;

#[derive(Debug, Clone, Serialize)]
pub struct ColdStartReport {
    pub schema_version: u32,
    pub runs: u32,
    pub port: u16,
    pub cold_start_ms: Vec<f64>,
    pub warm_avg_ms: f64,
    pub warm_samples: Vec<f64>,
    pub warmup_ms: Option<f64>,
}

pub fn run_cold_start(root: &Path, args: &BenchArgs) -> Result<()> {
    if !root.join("Cargo.toml").is_file() {
        bail!("missing Cargo.toml at {}", root.display());
    }
    let runs = args.cold_start_runs.max(1);
    let port = std::env::var("OCLIVE_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_API_PORT);
    let pkg = crate::bench_cmd::read_package_name(root)?;
    let warm_n = args.cold_start_warm_messages.max(1);
    let mut cold_samples = Vec::new();
    let mut all_warm = Vec::new();
    let mut last_warmup: Option<f64> = None;

    if !args.json {
        eprintln!(
            "oclive bench --cold-start — {} (runs={runs}, port={port}, MOCK_LLM=1)",
            root.display()
        );
    }

    for run_idx in 0..runs {
        if runs > 1 && !args.json {
            eprintln!("\n--- cold-start run {}/{} ---", run_idx + 1, runs);
        }
        let (cold_ms, warmup_ms, warm_ms) = one_cold_start_round(root, &pkg, port, warm_n)?;
        cold_samples.push(cold_ms);
        last_warmup = Some(warmup_ms);
        all_warm.extend(warm_ms);
    }

    let warm_avg = if all_warm.is_empty() {
        0.0
    } else {
        all_warm.iter().sum::<f64>() / all_warm.len() as f64
    };

    let report = ColdStartReport {
        schema_version: 1,
        runs,
        port,
        cold_start_ms: cold_samples.clone(),
        warm_avg_ms: warm_avg,
        warm_samples: all_warm.clone(),
        warmup_ms: last_warmup,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }
    Ok(())
}

fn one_cold_start_round(
    root: &Path,
    bin_name: &str,
    port: u16,
    warm_n: u32,
) -> Result<(f64, f64, Vec<f64>)> {
    let mut child = Command::new("cargo");
    child
        .args([
            "run",
            "--release",
            "--bin",
            bin_name,
            "--",
            "--api",
            "--port",
            &port.to_string(),
        ])
        .current_dir(root)
        .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let spawn_at = Instant::now();
    let mut proc = child
        .spawn()
        .context("spawn kernel (cargo run --release -- --api)")?;
    let stderr = proc.stderr.take();
    let _reader = stderr.map(|s| {
        std::thread::spawn(move || {
            let r = BufReader::new(s);
            for line in r.lines().map_while(Result::ok) {
                eprintln!("[kernel] {line}");
            }
        })
    });

    let warmup_ms = wait_tcp_listen(port, spawn_at, Duration::from_secs(180))?;
    let cold_elapsed = loop {
        if spawn_at.elapsed() > Duration::from_secs(180) {
            bail!("kernel did not respond to /chat within 180s (is --kernel-source linked?)");
        }
        match post_chat(port, "cold-start probe message") {
            Ok(()) => break spawn_at.elapsed().as_secs_f64() * 1000.0,
            Err(_) => std::thread::sleep(Duration::from_millis(500)),
        }
    };

    let mut warm = Vec::new();
    for i in 0..warm_n {
        let t0 = Instant::now();
        post_chat(port, &format!("warm message {i}"))?;
        warm.push(t0.elapsed().as_secs_f64() * 1000.0);
    }

    let _ = proc.kill();
    let _ = proc.wait();

    Ok((cold_elapsed, warmup_ms, warm))
}

fn wait_tcp_listen(port: u16, since: Instant, timeout: Duration) -> Result<f64> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    while since.elapsed() < timeout {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return Ok(since.elapsed().as_secs_f64() * 1000.0);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    bail!(
        "kernel API port {port} did not accept connections within {:?}",
        timeout
    );
}

fn post_chat(port: u16, message: &str) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/chat");
    let body = serde_json::json!({
        "message": message,
        "role_id": "default",
        "scene_id": "default"
    });
    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(120))
        .send_string(&body.to_string())
        .with_context(|| format!("POST {url}"))?;
    if resp.status() >= 400 {
        bail!("chat HTTP {}", resp.status());
    }
    let _text = resp.into_string().unwrap_or_default();
    Ok(())
}

fn print_human(r: &ColdStartReport) {
    println!("Cold-start report (port {})", r.port);
    if r.runs == 1 {
        println!("  cold start (first reply): {:.1} ms", r.cold_start_ms[0]);
    } else {
        let mean: f64 = r.cold_start_ms.iter().sum::<f64>() / r.cold_start_ms.len() as f64;
        println!(
            "  cold start ({} runs): {:?} ms (mean {:.1} ms)",
            r.runs, r.cold_start_ms, mean
        );
    }
    if let Some(w) = r.warmup_ms {
        println!("  warmup (spawn → API port listen): {:.1} ms", w);
    }
    println!(
        "  warm average (after first message): {:.1} ms",
        r.warm_avg_ms
    );
    if !r.warm_samples.is_empty() {
        println!(
            "  warm samples: min={:.1} max={:.1} (n={})",
            r.warm_samples.iter().cloned().fold(f64::INFINITY, f64::min),
            r.warm_samples.iter().cloned().fold(0.0, f64::max),
            r.warm_samples.len()
        );
    }
}
