//! `oclive compose` 多内核编排。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const COMPOSE_FILE: &str = "oclive-compose.yml";
const STATE_FILE: &str = ".oclive-compose.pids.json";

#[derive(Parser, Debug)]
pub struct ComposeCli {
    #[command(subcommand)]
    pub command: ComposeCommands,
}

#[derive(Subcommand, Debug)]
pub enum ComposeCommands {
    /// Generate oclive-compose.yml template
    Init(ComposeInitArgs),
    /// Start all services in dependency order
    Up(ComposeUpArgs),
    /// Stop all running instances
    Down(ComposeDownArgs),
    /// Show running status
    Ps(ComposePsArgs),
}

#[derive(Parser, Debug, Default)]
pub struct ComposeInitArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser, Debug, Default)]
pub struct ComposeUpArgs {
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,
    #[arg(short = 'o', long, default_value = ".")]
    pub cwd: PathBuf,
}

#[derive(Parser, Debug, Default)]
pub struct ComposeDownArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub cwd: PathBuf,
}

#[derive(Parser, Debug, Default)]
pub struct ComposePsArgs {
    #[arg(short = 'o', long, default_value = ".")]
    pub cwd: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComposeFile {
    services: HashMap<String, ComposeService>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ComposeService {
    path: PathBuf,
    port: u16,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    depends_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComposeState {
    services: HashMap<String, u32>,
}

pub fn run(cli: ComposeCli) -> Result<()> {
    match cli.command {
        ComposeCommands::Init(a) => run_init(a),
        ComposeCommands::Up(a) => run_up(a),
        ComposeCommands::Down(a) => run_down(a),
        ComposeCommands::Ps(a) => run_ps(a),
    }
}

fn run_init(args: ComposeInitArgs) -> Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path);
    let out = root.join(COMPOSE_FILE);
    if out.exists() {
        bail!("{} already exists", out.display());
    }
    let sample = r#"services:
  emotion-engine:
    path: ./emotion-engine
    port: 8421
    env:
      OCLIVE_HTTP_API_MOCK_LLM: "1"
  dialogue-engine:
    path: ./dialogue-engine
    port: 8422
    depends_on:
      - emotion-engine
    env:
      OCLIVE_HTTP_API_MOCK_LLM: "1"
"#;
    fs::write(&out, sample).context("write compose file")?;
    println!("Generated {}", out.display());
    Ok(())
}

fn compose_path(cwd: &Path, file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| cwd.join(COMPOSE_FILE))
}

fn load_compose(cwd: &Path, file: Option<PathBuf>) -> Result<(PathBuf, ComposeFile)> {
    let path = compose_path(cwd, file);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let file: ComposeFile = serde_yaml_ng::from_str(&raw).context("parse compose yaml")?;
    Ok((path, file))
}

fn topo_order(services: &HashMap<String, ComposeService>) -> Result<Vec<String>> {
    let mut indeg: HashMap<&str, usize> = services.keys().map(|k| (k.as_str(), 0)).collect();
    for (name, svc) in services {
        for dep in &svc.depends_on {
            if !services.contains_key(dep) {
                bail!("depends_on references unknown service: {dep}");
            }
            *indeg.get_mut(name.as_str()).unwrap() += 1;
        }
    }
    let mut q: VecDeque<&str> = indeg
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(k, _)| *k)
        .collect();
    let mut order = Vec::new();
    while let Some(n) = q.pop_front() {
        order.push(n.to_string());
        for (name, svc) in services {
            if svc.depends_on.iter().any(|d| d == n) {
                let e = indeg.get_mut(name.as_str()).unwrap();
                *e = e.saturating_sub(1);
                if *e == 0 {
                    q.push_back(name.as_str());
                }
            }
        }
    }
    if order.len() != services.len() {
        bail!("compose services have a circular dependency");
    }
    Ok(order)
}

fn state_path(cwd: &Path) -> PathBuf {
    cwd.join(STATE_FILE)
}

fn load_state(cwd: &Path) -> Result<ComposeState> {
    let p = state_path(cwd);
    if !p.is_file() {
        return Ok(ComposeState {
            services: HashMap::new(),
        });
    }
    let raw = fs::read_to_string(&p)?;
    serde_json::from_str(&raw).context("parse compose state")
}

fn save_state(cwd: &Path, state: &ComposeState) -> Result<()> {
    fs::write(state_path(cwd), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn spawn_service(cwd: &Path, name: &str, svc: &ComposeService) -> Result<Child> {
    let proj = cwd.join(&svc.path);
    if !proj.join("Cargo.toml").is_file() {
        bail!("service {name}: {} missing Cargo.toml", proj.display());
    }
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--release")
        .arg("--")
        .arg("--api")
        .arg("--port")
        .arg(svc.port.to_string())
        .current_dir(&proj)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in &svc.env {
        cmd.env(k, v);
    }
    let mut child = cmd
        .spawn()
        .with_context(|| format!("spawn service {name}"))?;
    if let Some(out) = child.stdout.take() {
        stream_lines(name, out, false);
    }
    if let Some(err) = child.stderr.take() {
        stream_lines(name, err, true);
    }
    Ok(child)
}

fn stream_lines<R: Read + Send + 'static>(name: &str, reader: R, is_stderr: bool) {
    let prefix = format!("[{name}] ");
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if is_stderr {
                        eprint!("{prefix}{line}");
                    } else {
                        print!("{prefix}{line}");
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn run_up(args: ComposeUpArgs) -> Result<()> {
    let cwd = args.cwd.canonicalize().unwrap_or(args.cwd);
    let (_path, compose) = load_compose(&cwd, args.file)?;
    let order = topo_order(&compose.services)?;
    let mut state = ComposeState {
        services: HashMap::new(),
    };
    for name in order {
        let svc = compose.services.get(&name).unwrap();
        eprintln!("[oclive compose] starting {name} (port {})…", svc.port);
        let child = spawn_service(&cwd, &name, svc)?;
        state.services.insert(name.clone(), child.id());
        std::mem::forget(child);
        thread::sleep(Duration::from_millis(800));
    }
    save_state(&cwd, &state)?;
    println!(
        "[oclive compose] started {} service(s) in background; stop with `oclive compose down`",
        state.services.len()
    );
    Ok(())
}

fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
}

fn run_down(args: ComposeDownArgs) -> Result<()> {
    let cwd = args.cwd.canonicalize().unwrap_or(args.cwd);
    let state = load_state(&cwd)?;
    if state.services.is_empty() {
        println!("[oclive compose] no running records");
        return Ok(());
    }
    let names: Vec<_> = state.services.keys().cloned().collect();
    for name in names.iter().rev() {
        if let Some(pid) = state.services.get(name) {
            eprintln!("[oclive compose] stopping {name} (pid {pid})");
            kill_pid(*pid);
        }
    }
    save_state(
        &cwd,
        &ComposeState {
            services: HashMap::new(),
        },
    )?;
    println!("[oclive compose] all stopped");
    Ok(())
}

fn run_ps(args: ComposePsArgs) -> Result<()> {
    let cwd = args.cwd.canonicalize().unwrap_or(args.cwd);
    let state = load_state(&cwd)?;
    if state.services.is_empty() {
        println!("(no compose state file or not yet up)");
        return Ok(());
    }
    for (name, pid) in &state.services {
        println!("{name}: pid={pid}");
    }
    Ok(())
}
