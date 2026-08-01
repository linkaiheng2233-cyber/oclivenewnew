//! `bench --stress` — HTTP /chat concurrency stress test.

use anyhow::{bail, Result};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::bench_cmd::BenchArgs;
use oclive_kernel_runtime::DEFAULT_API_PORT;

#[derive(Debug, Clone, Serialize)]
pub struct StressReport {
    pub schema_version: u32,
    pub duration_secs: f64,
    pub concurrency: u32,
    pub total_requests: u64,
    pub errors: u64,
    pub error_rate: f64,
    pub throughput_rps: f64,
    pub latency_ms: LatencyStats,
    pub peak_memory_mib: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyStats {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

pub fn run_stress(root: &std::path::Path, args: &BenchArgs) -> Result<()> {
    let concurrency = args.stress_concurrency.max(1);
    let duration = Duration::from_secs(args.stress_duration.max(1));
    let port = std::env::var("OCLIVE_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_API_PORT);
    let url = format!("http://127.0.0.1:{port}/chat");
    let agent = crate::http_client::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();
    let body = r#"{"message":"bench stress ping"}"#;

    let samples: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
    let errors = Arc::new(AtomicU64::new(0));
    let total = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));

    let _ = ctrlc::set_handler({
        let stop = stop.clone();
        move || stop.store(true, Ordering::SeqCst)
    });

    if !args.json {
        eprintln!(
            "oclive bench --stress — POST {url} (concurrency={concurrency}, duration={}s, Ctrl+C to stop)",
            duration.as_secs()
        );
    }

    let start = Instant::now();
    let deadline = start + duration;
    let mut handles = Vec::new();

    for _ in 0..concurrency {
        let agent = agent.clone();
        let url = url.clone();
        let body = body.to_string();
        let samples = samples.clone();
        let errors = errors.clone();
        let total = total.clone();
        let stop = stop.clone();
        handles.push(std::thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) && Instant::now() < deadline {
                let t0 = Instant::now();
                match agent
                    .post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&body)
                {
                    Ok(_) => {
                        let ms = t0.elapsed().as_secs_f64() * 1000.0;
                        total.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut s) = samples.lock() {
                            s.push(ms);
                        }
                    }
                    Err(_) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                        total.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    while Instant::now() < deadline && !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(500));
        if !args.json {
            let n = total.load(Ordering::Relaxed);
            let e = errors.load(Ordering::Relaxed);
            eprint!(
                "\r  requests={n} errors={e} elapsed={:.1}s   ",
                start.elapsed().as_secs_f64()
            );
        }
    }
    stop.store(true, Ordering::SeqCst);
    for h in handles {
        let _ = h.join();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let mut lat = samples.lock().unwrap().clone();
    lat.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = lat.len();
    let stats = if n == 0 {
        LatencyStats {
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            min: 0.0,
            max: 0.0,
            mean: 0.0,
        }
    } else {
        LatencyStats {
            p50: percentile(&lat, 0.50),
            p95: percentile(&lat, 0.95),
            p99: percentile(&lat, 0.99),
            min: lat[0],
            max: lat[n - 1],
            mean: lat.iter().sum::<f64>() / n as f64,
        }
    };

    let err_n = errors.load(Ordering::Relaxed);
    let tot = total.load(Ordering::Relaxed).max(1);
    let report = StressReport {
        schema_version: 1,
        duration_secs: elapsed,
        concurrency,
        total_requests: tot,
        errors: err_n,
        error_rate: err_n as f64 / tot as f64,
        throughput_rps: (tot as f64) / elapsed.max(0.001),
        latency_ms: stats,
        peak_memory_mib: probe_peak_memory_mib(root),
    };

    if !args.json && args.output == "-" {
        eprintln!();
        print_stress_human(&report, &url);
        if err_n > 0 && n == 0 {
            eprintln!("\nHint: start the kernel HTTP API first (e.g. cargo run --release in the project).");
        }
    } else {
        crate::bench_cmd::emit_json_report(args, &report)?;
    }

    if err_n > tot / 2 {
        bail!(
            "stress test error rate too high ({:.1}%)",
            report.error_rate * 100.0
        );
    }
    Ok(())
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn probe_peak_memory_mib(_root: &std::path::Path) -> Option<u64> {
    None
}

fn print_stress_human(r: &StressReport, url: &str) {
    println!("Stress report — {url}");
    println!("  duration:     {:.2}s", r.duration_secs);
    println!("  concurrency:  {}", r.concurrency);
    println!("  requests:     {}", r.total_requests);
    println!(
        "  errors:       {} ({:.2}%)",
        r.errors,
        r.error_rate * 100.0
    );
    println!("  throughput:   {:.2} req/s", r.throughput_rps);
    println!(
        "  latency (ms): p50={:.1} p95={:.1} p99={:.1} min={:.1} max={:.1} mean={:.1}",
        r.latency_ms.p50,
        r.latency_ms.p95,
        r.latency_ms.p99,
        r.latency_ms.min,
        r.latency_ms.max,
        r.latency_ms.mean,
    );
    if let Some(m) = r.peak_memory_mib {
        println!("  peak memory:  {m} MiB");
    }
}
