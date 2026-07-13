//! `bench --equivalence` — compare standard vs Monolith `/chat` replies (MOCK_LLM).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::bench_cmd::BenchArgs;
use crate::build_cmd::{regenerate_monolith_from_disk, run_timed_dual_build};

const DEFAULT_MESSAGES: &[&str] = &[
    "equivalence probe one",
    "equivalence probe two",
    "equivalence probe three",
];

#[derive(Debug, Clone, Serialize)]
pub struct EquivalenceReport {
    pub schema_version: u32,
    pub messages: u32,
    pub exact_matches: u32,
    pub mismatches: u32,
    pub diffs: Vec<EquivalenceDiff>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquivalenceDiff {
    pub index: u32,
    pub message: String,
    pub standard_reply: String,
    pub monolith_reply: String,
}

pub fn run_equivalence(root: &Path, args: &BenchArgs) -> Result<()> {
    let mt = root.join("monolith.toml");
    if !mt.is_file() {
        bail!("missing monolith.toml (equivalence requires Monolith project)");
    }
    let file = regenerate_monolith_from_disk(root)?;
    if !file.monolith.enabled {
        bail!("monolith.toml: enabled = false");
    }
    if args.release {
        eprintln!("cargo build --release (standard + Monolith)…");
        crate::build_cmd::run_timed_dual_build(root, true, &args.cargo_extra, true)?;
    } else {
        run_timed_dual_build(root, false, &args.cargo_extra, true)?;
    }
    let pkg = crate::bench_cmd::read_package_name(root)?;
    let std_bin = crate::bench_cmd::release_bin_path(root, &pkg, args.release);
    let mono_bin =
        crate::bench_cmd::release_bin_path(root, &format!("{pkg}-monolith"), args.release);

    let mut exact = 0u32;
    let mut diffs = Vec::new();
    for (i, msg) in DEFAULT_MESSAGES.iter().enumerate() {
        let std_reply = chat_once(&std_bin, 18420u16.wrapping_add(i as u16), msg)?;
        let mono_reply = chat_once(&mono_bin, 19420u16.wrapping_add(i as u16), msg)?;
        if std_reply == mono_reply {
            exact += 1;
        } else {
            diffs.push(EquivalenceDiff {
                index: i as u32,
                message: (*msg).to_string(),
                standard_reply: std_reply,
                monolith_reply: mono_reply,
            });
        }
    }

    let report = EquivalenceReport {
        schema_version: 1,
        messages: DEFAULT_MESSAGES.len() as u32,
        exact_matches: exact,
        mismatches: diffs.len() as u32,
        diffs,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }
    if report.mismatches > 0 {
        bail!(
            "{} message(s) differ between standard and Monolith",
            report.mismatches
        );
    }
    Ok(())
}

fn chat_once(bin: &Path, port: u16, message: &str) -> Result<String> {
    let mut child = Command::new(bin);
    child
        .args(["--api", "--port", &port.to_string()])
        .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let spawn_at = Instant::now();
    let mut proc = child
        .spawn()
        .with_context(|| format!("spawn {}", bin.display()))?;
    if let Some(stderr) = proc.stderr.take() {
        std::thread::spawn(move || {
            let r = BufReader::new(stderr);
            for line in r.lines().map_while(Result::ok) {
                eprintln!("[kernel] {line}");
            }
        });
    }
    wait_tcp(port, spawn_at, Duration::from_secs(180))?;
    let reply = post_chat_extract(port, message)?;
    let _ = proc.kill();
    let _ = proc.wait();
    Ok(reply)
}

fn wait_tcp(port: u16, since: Instant, timeout: Duration) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    while since.elapsed() < timeout {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    bail!("port {port} not ready");
}

fn post_chat_extract(port: u16, message: &str) -> Result<String> {
    let url = format!("http://127.0.0.1:{port}/chat");
    let body = serde_json::json!({
        "message": message,
        "role_id": "default",
        "scene_id": "default"
    });
    let resp = crate::http_client::post(&url)
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(120))
        .send_string(&body.to_string())?;
    let text = resp.into_string().unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(r) = v.get("reply").and_then(|x| x.as_str()) {
            return Ok(r.to_string());
        }
    }
    Ok(text)
}

fn print_human(r: &EquivalenceReport) {
    println!("Monolith equivalence report");
    println!(
        "  messages: {}  exact matches: {}  mismatches: {}",
        r.messages, r.exact_matches, r.mismatches
    );
    for d in &r.diffs {
        println!(
            "  diff #{} {:?}\n    standard: {}\n    monolith: {}",
            d.index, d.message, d.standard_reply, d.monolith_reply
        );
    }
}
