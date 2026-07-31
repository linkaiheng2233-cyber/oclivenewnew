//! `bench --soak` — long-run stability sampling (RSS / handles proxy via sysinfo).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::bench_cmd::BenchArgs;

#[derive(Debug, Clone, Serialize)]
pub struct SoakReport {
    pub schema_version: u32,
    pub hours: f64,
    pub samples: Vec<SoakSample>,
    pub initial_rss_mib: f64,
    pub final_rss_mib: f64,
    pub growth_warn: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SoakSample {
    pub hour: f64,
    pub rss_mib: f64,
    pub chats: u64,
}

pub fn run_soak(root: &Path, args: &BenchArgs) -> Result<()> {
    if !root.join("Cargo.toml").is_file() {
        bail!("missing Cargo.toml");
    }
    let hours = args.soak_duration.max(1) as f64;
    // Accelerated wall clock: ~2s per nominal hour (cap 120s) so local/CI can finish; see PERFORMANCE.md soak section.
    let wall_secs = (hours * 2.0).clamp(8.0, 120.0) as u64;
    let wall_duration = Duration::from_secs(wall_secs);
    let n_samples = hours as u32;
    let sample_interval = wall_duration / n_samples.max(1);

    let pkg = crate::bench_cmd::read_package_name(root)?;
    let port = 18500u16;
    let mut child = Command::new("cargo");
    child
        .args([
            "run",
            "--release",
            "--bin",
            &pkg,
            "--",
            "--api",
            "--port",
            &port.to_string(),
        ])
        .current_dir(root)
        .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
        .env("OCLIVE_API_TOKEN", crate::http_client::api_token())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let spawn_at = Instant::now();
    let mut proc = child.spawn().context("spawn soak kernel")?;
    if let Some(stderr) = proc.stderr.take() {
        std::thread::spawn(move || {
            let r = BufReader::new(stderr);
            for line in r.lines().map_while(Result::ok) {
                eprintln!("[soak] {line}");
            }
        });
    }
    wait_tcp(port, spawn_at, Duration::from_secs(300))?;

    let mut samples = Vec::new();
    let mut chats = 0u64;
    let mut initial_rss = 0f64;
    for h in 0..n_samples {
        let tick_deadline = spawn_at + sample_interval * (h + 1);
        while Instant::now() < tick_deadline {
            let _ = post_chat(port, &format!("soak tick {chats}"));
            chats += 1;
            std::thread::sleep(Duration::from_millis(500));
        }
        let rss = proc_rss_mib(proc.id()).unwrap_or(0.0);
        if samples.is_empty() {
            initial_rss = rss;
        }
        samples.push(SoakSample {
            hour: h as f64,
            rss_mib: rss,
            chats,
        });
    }
    let final_rss = proc_rss_mib(proc.id()).unwrap_or(0.0);
    let _ = proc.kill();
    let _ = proc.wait();

    let growth_warn = initial_rss > 0.0 && final_rss > initial_rss * 1.2;
    let report = SoakReport {
        schema_version: 1,
        hours,
        samples,
        initial_rss_mib: initial_rss,
        final_rss_mib: final_rss,
        growth_warn,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "oclive bench --soak (simulated {hours}h wall window {:?})",
            wall_duration
        );
        println!("  initial RSS: {initial_rss:.1} MiB  final: {final_rss:.1} MiB  chats: {chats}");
        if growth_warn {
            println!("  ⚠️  RSS grew more than 20% vs first sample");
        }
        for s in &report.samples {
            println!(
                "  hour {:.0}: RSS {:.1} MiB (chats={})",
                s.hour, s.rss_mib, s.chats
            );
        }
    }
    Ok(())
}

fn wait_tcp(port: u16, since: Instant, timeout: Duration) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    while since.elapsed() < timeout {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    bail!("soak: API port {port} not ready");
}

fn post_chat(port: u16, message: &str) -> Result<()> {
    let url = format!("http://127.0.0.1:{port}/chat");
    let body =
        serde_json::json!({ "message": message, "role_id": "default", "scene_id": "default" });
    let resp = crate::http_client::post(&url)
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(60))
        .send_string(&body.to_string())?;
    if resp.status() >= 400 {
        bail!("chat HTTP {}", resp.status());
    }
    Ok(())
}

fn proc_rss_mib(pid: u32) -> Result<f64> {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let mut sys = System::new();
    let pid = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    let mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    Ok(mem as f64 / (1024.0 * 1024.0))
}
