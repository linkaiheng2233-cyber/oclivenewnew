//! `bench --soak` — long-run kernel stability sampling.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::bench_cmd::BenchArgs;

const ACCELERATED_SECONDS_PER_HOUR: f64 = 2.0;
const ACCELERATED_MIN_SECONDS: f64 = 8.0;
const ACCELERATED_MAX_SECONDS: f64 = 120.0;
const ACCELERATED_CHAT_INTERVAL: Duration = Duration::from_millis(500);
const REAL_TIME_CHAT_INTERVAL: Duration = Duration::from_secs(5);
const LOOP_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SoakMode {
    Accelerated,
    RealTime,
}

#[derive(Debug, Clone, Copy)]
struct SoakSchedule {
    mode: SoakMode,
    requested_hours: f64,
    wall_duration: Duration,
    sample_interval: Duration,
    chat_interval: Duration,
}

impl SoakSchedule {
    fn from_args(args: &BenchArgs) -> Result<Self> {
        let requested_hours = args.soak_duration;
        if !requested_hours.is_finite() || requested_hours <= 0.0 {
            bail!("--soak-duration must be a finite number greater than zero");
        }

        if args.soak_real_time {
            let wall_seconds = requested_hours * 60.0 * 60.0;
            if !wall_seconds.is_finite() || wall_seconds > Duration::MAX.as_secs_f64() {
                bail!("--soak-duration is too large for the platform clock");
            }
            return Ok(Self {
                mode: SoakMode::RealTime,
                requested_hours,
                wall_duration: Duration::from_secs_f64(wall_seconds),
                sample_interval: Duration::from_secs(args.soak_sample_interval.max(1)),
                chat_interval: REAL_TIME_CHAT_INTERVAL,
            });
        }

        let wall_seconds = (requested_hours * ACCELERATED_SECONDS_PER_HOUR)
            .clamp(ACCELERATED_MIN_SECONDS, ACCELERATED_MAX_SECONDS);
        let sample_count = requested_hours.ceil().clamp(1.0, 10_000.0) as u32;
        let wall_duration = Duration::from_secs_f64(wall_seconds);
        Ok(Self {
            mode: SoakMode::Accelerated,
            requested_hours,
            wall_duration,
            sample_interval: wall_duration / sample_count,
            chat_interval: ACCELERATED_CHAT_INTERVAL,
        })
    }

    fn nominal_hour(self, elapsed: Duration) -> f64 {
        if self.mode == SoakMode::RealTime {
            return elapsed.as_secs_f64() / 3600.0;
        }
        let progress = (elapsed.as_secs_f64() / self.wall_duration.as_secs_f64()).clamp(0.0, 1.0);
        self.requested_hours * progress
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SoakReport {
    pub schema_version: u32,
    mode: SoakMode,
    pub hours: f64,
    pub wall_duration_secs: f64,
    pub sample_interval_secs: f64,
    pub chat_interval_secs: f64,
    pub port: u16,
    pub process_id: u32,
    pub samples: Vec<SoakSample>,
    pub initial_rss_mib: f64,
    pub final_rss_mib: f64,
    pub peak_rss_mib: f64,
    pub peak_cpu_percent: f32,
    pub warmup_chats: u64,
    pub successful_chats: u64,
    pub failed_chats: u64,
    pub sampling_failures: u64,
    pub last_chat_error: Option<String>,
    pub process_early_exit: Option<String>,
    pub chat_worker_joined: bool,
    pub process_reaped: bool,
    pub growth_warn: bool,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SoakSample {
    /// Nominal progress retained for schema-v1 readers; real-time mode reports actual elapsed hours.
    pub hour: f64,
    pub elapsed_secs: f64,
    pub rss_mib: f64,
    pub cpu_percent: f32,
    /// Successful chats retained under the schema-v1 field name.
    pub chats: u64,
    pub failed_chats: u64,
}

struct KernelProcess {
    child: Child,
    reaped: bool,
}

impl KernelProcess {
    fn spawn(binary: &Path, root: &Path, port: u16) -> Result<Self> {
        let mut command = Command::new(binary);
        command
            .args(["--api", "--port", &port.to_string()])
            .current_dir(root)
            .env("OCLIVE_HTTP_API_MOCK_LLM", "1")
            .env("OCLIVE_API_TOKEN", crate::http_client::api_token())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .with_context(|| format!("spawn soak kernel {}", binary.display()))?;
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[soak] {line}");
                }
            });
        }
        Ok(Self {
            child,
            reaped: false,
        })
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        let status = self.child.try_wait().context("poll soak kernel")?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }

    fn stop_and_reap(&mut self) -> bool {
        if self.reaped {
            return true;
        }
        match self.child.try_wait() {
            Ok(Some(_)) => {
                self.reaped = true;
                true
            }
            Ok(None) => {
                if self.child.kill().is_err() {
                    return false;
                }
                let reaped = self.child.wait().is_ok();
                self.reaped = reaped;
                reaped
            }
            Err(_) => false,
        }
    }
}

impl Drop for KernelProcess {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
            self.reaped = true;
        }
    }
}

struct ProcessSampler {
    system: System,
    pid: Pid,
}

struct ChatWorker {
    requests: Option<mpsc::Sender<String>>,
    results: mpsc::Receiver<Result<(), String>>,
    thread: Option<JoinHandle<()>>,
}

impl ChatWorker {
    fn new(port: u16, role_path: PathBuf) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<String>();
        let (result_tx, result_rx) = mpsc::channel::<Result<(), String>>();
        let thread = std::thread::spawn(move || {
            while let Ok(message) = request_rx.recv() {
                let result = crate::bench_http::post_chat(
                    port,
                    &role_path,
                    &message,
                    Duration::from_secs(10),
                )
                .map(|_| ())
                .map_err(|error| format!("{error:#}"));
                if result_tx.send(result).is_err() {
                    break;
                }
            }
        });
        Self {
            requests: Some(request_tx),
            results: result_rx,
            thread: Some(thread),
        }
    }

    fn request(&self, message: String) -> Result<()> {
        self.requests
            .as_ref()
            .context("soak chat worker is stopped")?
            .send(message)
            .context("send soak chat request")
    }

    fn try_result(&self) -> Option<Result<(), String>> {
        self.results.try_recv().ok()
    }

    fn wait_result(&self, timeout: Duration) -> Option<Result<(), String>> {
        self.results.recv_timeout(timeout).ok()
    }

    fn stop_and_join(&mut self) -> bool {
        self.requests.take();
        self.thread
            .take()
            .is_none_or(|thread| thread.join().is_ok())
    }
}

impl Drop for ChatWorker {
    fn drop(&mut self) {
        let _ = self.stop_and_join();
    }
}

impl ProcessSampler {
    fn new(pid: u32) -> Self {
        let mut system = System::new();
        let pid = Pid::from_u32(pid);
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        Self { system, pid }
    }

    fn sample(&mut self) -> Option<(f64, f32)> {
        self.system
            .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);
        self.system.process(self.pid).map(|process| {
            (
                process.memory() as f64 / (1024.0 * 1024.0),
                process.cpu_usage(),
            )
        })
    }
}

/// Run the long-duration stability probe against a directly spawned release kernel.
///
/// # Errors
///
/// Returns an error for invalid timing, build/start failures, early process exit,
/// request failures, RSS growth above 20%, or a child that cannot be reaped.
pub fn run_soak(root: &Path, args: &BenchArgs) -> Result<()> {
    if !root.join("Cargo.toml").is_file() {
        bail!("missing Cargo.toml at {}", root.display());
    }
    let schedule = SoakSchedule::from_args(args)?;
    let package = crate::bench_cmd::read_package_name(root)?;
    let role_path = crate::bench_http::resolve_bench_role_path(root)?;
    let binary = build_release_kernel(root, &package, args)?;
    let port = available_local_port()?;
    let mut process = KernelProcess::spawn(&binary, root, port)?;
    let process_id = process.id();
    let readiness_started = Instant::now();
    wait_tcp(
        port,
        &mut process,
        readiness_started,
        Duration::from_secs(300),
    )?;
    crate::bench_http::post_chat(port, &role_path, "soak warmup", Duration::from_secs(10))
        .context("soak warmup chat failed")?;
    let warmup_chats = 1;
    // The requested soak clock begins only after the API is ready. Build and cold-start
    // latency plus the lazy-allocation warmup are covered separately and must not shorten
    // a real-time soak or inflate the steady-state RSS growth baseline.
    let started = Instant::now();

    let mut sampler = ProcessSampler::new(process_id);
    let mut chat_worker = ChatWorker::new(port, role_path);
    let mut samples = Vec::new();
    let mut successful_chats = 0u64;
    let mut failed_chats = 0u64;
    let mut sampling_failures = 0u64;
    let mut last_chat_error = None;
    let mut process_early_exit = None;
    let mut peak_rss_mib = 0.0f64;
    let mut peak_cpu_percent = 0.0f32;

    if !push_sample(
        &mut samples,
        &mut sampler,
        schedule,
        started.elapsed(),
        successful_chats,
        failed_chats,
        &mut peak_rss_mib,
        &mut peak_cpu_percent,
    ) {
        sampling_failures += 1;
    }

    let deadline = started + schedule.wall_duration;
    let mut next_chat = Instant::now();
    let mut next_sample = Instant::now() + schedule.sample_interval;
    let mut chat_in_flight = false;
    while Instant::now() < deadline {
        if let Some(status) = process.try_wait()? {
            process_early_exit = Some(status.to_string());
            break;
        }

        if chat_in_flight {
            if let Some(result) = chat_worker.try_result() {
                record_chat_result(
                    result,
                    &mut successful_chats,
                    &mut failed_chats,
                    &mut last_chat_error,
                );
                chat_in_flight = false;
            }
        }

        let now = Instant::now();
        if now >= next_chat && !chat_in_flight {
            chat_worker.request(format!("soak tick {}", successful_chats + failed_chats))?;
            chat_in_flight = true;
            next_chat = now + schedule.chat_interval;
        }
        if Instant::now() >= next_sample {
            if !push_sample(
                &mut samples,
                &mut sampler,
                schedule,
                started.elapsed(),
                successful_chats,
                failed_chats,
                &mut peak_rss_mib,
                &mut peak_cpu_percent,
            ) {
                sampling_failures += 1;
            }
            next_sample += schedule.sample_interval;
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        std::thread::sleep(LOOP_POLL_INTERVAL.min(remaining));
    }

    if !push_sample(
        &mut samples,
        &mut sampler,
        schedule,
        started.elapsed(),
        successful_chats,
        failed_chats,
        &mut peak_rss_mib,
        &mut peak_cpu_percent,
    ) {
        sampling_failures += 1;
    }
    if chat_in_flight {
        let result = chat_worker
            .wait_result(Duration::from_secs(11))
            .unwrap_or_else(|| Err("chat worker did not finish within 11s".to_string()));
        record_chat_result(
            result,
            &mut successful_chats,
            &mut failed_chats,
            &mut last_chat_error,
        );
    }
    let chat_worker_joined = chat_worker.stop_and_join();
    let process_reaped = process.stop_and_reap();

    let initial_rss_mib = samples.first().map_or(0.0, |sample| sample.rss_mib);
    let final_rss_mib = samples.last().map_or(0.0, |sample| sample.rss_mib);
    let growth_warn = initial_rss_mib > 0.0 && final_rss_mib > initial_rss_mib * 1.2;
    let ok = !growth_warn
        && failed_chats == 0
        && sampling_failures == 0
        && !samples.is_empty()
        && chat_worker_joined
        && process_early_exit.is_none()
        && process_reaped;
    let report = SoakReport {
        schema_version: 2,
        mode: schedule.mode,
        hours: schedule.requested_hours,
        wall_duration_secs: schedule.wall_duration.as_secs_f64(),
        sample_interval_secs: schedule.sample_interval.as_secs_f64(),
        chat_interval_secs: schedule.chat_interval.as_secs_f64(),
        port,
        process_id,
        samples,
        initial_rss_mib,
        final_rss_mib,
        peak_rss_mib,
        peak_cpu_percent,
        warmup_chats,
        successful_chats,
        failed_chats,
        sampling_failures,
        last_chat_error,
        process_early_exit,
        chat_worker_joined,
        process_reaped,
        growth_warn,
        ok,
    };

    emit_report(args, &report)?;
    if !report.ok {
        bail!("soak verification failed; inspect the emitted schema-v2 report");
    }
    Ok(())
}

fn build_release_kernel(root: &Path, package: &str, args: &BenchArgs) -> Result<PathBuf> {
    let mut command = Command::new("cargo");
    command
        .args(["build", "--release", "--bin", package])
        .args(&args.cargo_extra)
        .current_dir(root)
        .stdout(if args.json {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .stderr(Stdio::inherit());
    let status = command.status().context("build release kernel for soak")?;
    if !status.success() {
        bail!("cargo build --release --bin {package} failed: {status}");
    }

    let metadata = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .context("read cargo target directory for soak")?;
    if !metadata.status.success() {
        bail!("cargo metadata failed: {}", metadata.status);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&metadata.stdout).context("parse cargo metadata for soak")?;
    let target_dir = value
        .get("target_directory")
        .and_then(serde_json::Value::as_str)
        .context("cargo metadata missing target_directory")?;
    let mut binary = PathBuf::from(target_dir).join("release").join(package);
    if cfg!(windows) {
        binary.set_extension("exe");
    }
    if !binary.is_file() {
        bail!("built soak kernel not found at {}", binary.display());
    }
    Ok(binary)
}

fn available_local_port() -> Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).context("reserve soak API port")?;
    listener
        .local_addr()
        .map(|address| address.port())
        .context("read reserved soak API port")
}

fn wait_tcp(
    port: u16,
    process: &mut KernelProcess,
    since: Instant,
    timeout: Duration,
) -> Result<()> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    while since.elapsed() < timeout {
        if let Some(status) = process.try_wait()? {
            bail!("soak kernel exited before API readiness: {status}");
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(300)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    bail!("soak: API port {port} not ready within {timeout:?}");
}

fn record_chat_result(
    result: Result<(), String>,
    successful_chats: &mut u64,
    failed_chats: &mut u64,
    last_chat_error: &mut Option<String>,
) {
    match result {
        Ok(()) => *successful_chats += 1,
        Err(error) => {
            *failed_chats += 1;
            *last_chat_error = Some(error);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_sample(
    samples: &mut Vec<SoakSample>,
    sampler: &mut ProcessSampler,
    schedule: SoakSchedule,
    elapsed: Duration,
    successful_chats: u64,
    failed_chats: u64,
    peak_rss_mib: &mut f64,
    peak_cpu_percent: &mut f32,
) -> bool {
    let Some((rss_mib, cpu_percent)) = sampler.sample() else {
        return false;
    };
    *peak_rss_mib = (*peak_rss_mib).max(rss_mib);
    *peak_cpu_percent = (*peak_cpu_percent).max(cpu_percent);
    samples.push(SoakSample {
        hour: schedule.nominal_hour(elapsed),
        elapsed_secs: elapsed.as_secs_f64(),
        rss_mib,
        cpu_percent,
        chats: successful_chats,
        failed_chats,
    });
    true
}

fn emit_report(args: &BenchArgs, report: &SoakReport) -> Result<()> {
    if args.json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!(
        "oclive bench --soak ({:?}, requested {:.3}h, wall {:?})",
        report.mode,
        report.hours,
        Duration::from_secs_f64(report.wall_duration_secs)
    );
    println!(
        "  kernel pid: {}  RSS: {:.1} → {:.1} MiB (peak {:.1})  CPU peak: {:.1}%",
        report.process_id,
        report.initial_rss_mib,
        report.final_rss_mib,
        report.peak_rss_mib,
        report.peak_cpu_percent
    );
    println!(
        "  chats: {} warmup + {} successful / {} failed  samples: {} failed  worker joined: {}  process reaped: {}",
        report.warmup_chats,
        report.successful_chats,
        report.failed_chats,
        report.sampling_failures,
        report.chat_worker_joined,
        report.process_reaped
    );
    if report.growth_warn {
        println!("  ⚠ RSS grew more than 20% vs first sample");
    }
    if let Some(exit) = &report.process_early_exit {
        println!("  ⚠ kernel exited before the soak deadline: {exit}");
    }
    if let Some(error) = &report.last_chat_error {
        println!("  ⚠ last chat error: {error}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    fn args() -> BenchArgs {
        BenchArgs {
            path: PathBuf::from("."),
            runs: 1,
            inner_iters: 1,
            release: true,
            json: true,
            output: "-".into(),
            save: false,
            compare: false,
            history: false,
            watch: false,
            live: false,
            dashboard: false,
            matrix: false,
            regression: false,
            regression_threshold: None,
            compare_versions: None,
            stress: false,
            stress_concurrency: 1,
            stress_duration: 1,
            equivalence: false,
            soak: true,
            soak_duration: 72.0,
            soak_real_time: false,
            soak_sample_interval: 60,
            cold_start: false,
            cold_start_runs: 1,
            cold_start_warm_messages: 1,
            cargo_extra: Vec::new(),
        }
    }

    #[test]
    fn accelerated_schedule_preserves_bounded_smoke_clock() {
        let schedule = SoakSchedule::from_args(&args()).expect("accelerated schedule");
        assert_eq!(schedule.mode, SoakMode::Accelerated);
        assert_eq!(schedule.wall_duration, Duration::from_secs(120));
        assert_eq!(schedule.chat_interval, Duration::from_millis(500));
        assert_eq!(schedule.nominal_hour(Duration::from_secs(60)), 36.0);
    }

    #[test]
    fn real_time_schedule_uses_actual_fractional_hours() {
        let mut value = args();
        value.soak_real_time = true;
        value.soak_duration = 0.01;
        value.soak_sample_interval = 5;
        let schedule = SoakSchedule::from_args(&value).expect("real-time schedule");
        assert_eq!(schedule.mode, SoakMode::RealTime);
        assert_eq!(schedule.wall_duration, Duration::from_secs(36));
        assert_eq!(schedule.sample_interval, Duration::from_secs(5));
        assert_eq!(schedule.chat_interval, Duration::from_secs(5));
    }

    #[test]
    fn soak_duration_must_be_positive_and_finite() {
        for duration in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let mut value = args();
            value.soak_duration = duration;
            assert!(SoakSchedule::from_args(&value).is_err());
        }
    }

    #[test]
    fn available_port_is_nonzero() {
        assert_ne!(available_local_port().expect("available port"), 0);
    }

    #[test]
    fn cli_accepts_fractional_real_time_soak_settings() {
        let parsed = BenchArgs::try_parse_from([
            "bench",
            "--soak",
            "--soak-duration",
            "0.01",
            "--soak-real-time",
            "--soak-sample-interval",
            "5",
        ])
        .expect("parse real-time soak flags");
        assert!(parsed.soak);
        assert!(parsed.soak_real_time);
        assert_eq!(parsed.soak_duration, 0.01);
        assert_eq!(parsed.soak_sample_interval, 5);
    }
}
