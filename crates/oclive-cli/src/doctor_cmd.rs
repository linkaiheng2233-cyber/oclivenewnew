//! `oclive doctor` — 一键环境诊断。

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct DoctorArgs {
    /// 机器可读 JSON（`schema_version` + `checks[]`）
    #[arg(long)]
    pub json: bool,

    /// 工作区探针目录（默认可写性检查用当前目录）
    #[arg(short = 'o', long, default_value = ".")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorReport {
    pub schema_version: u32,
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorCheck {
    pub id: String,
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl DoctorCheck {
    fn ok(id: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "ok".into(),
            message: message.into(),
            detail: None,
        }
    }

    fn warn(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "warn".into(),
            message: message.into(),
            detail,
        }
    }

    fn fail(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "fail".into(),
            message: message.into(),
            detail,
        }
    }
}

pub fn run(args: DoctorArgs) -> Result<()> {
    let root = if args.path.is_absolute() {
        args.path.clone()
    } else {
        std::env::current_dir()?.join(&args.path)
    };
    let checks = vec![
        check_rust_toolchain(),
        check_cargo(),
        check_system_memory(),
        check_disk_space(&root),
        check_ollama(),
        check_network_github(),
        check_workspace_writable(&root),
    ];
    let ok = checks.iter().all(|c| c.status != "fail");
    let report = DoctorReport {
        schema_version: 1,
        ok,
        checks,
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}

fn print_human(report: &DoctorReport) {
    println!("oclive doctor — 环境诊断\n");
    for c in &report.checks {
        let icon = match c.status.as_str() {
            "ok" => "✅",
            "warn" => "⚠️",
            _ => "❌",
        };
        println!("{icon} [{}] {}", c.id, c.message);
        if let Some(d) = &c.detail {
            println!("    {d}");
        }
    }
    println!();
    if report.ok {
        println!("总结: 环境就绪，可执行 oclive init / cargo build。");
    } else {
        println!("总结: 存在失败项，请先修复后再初始化内核项目。");
    }
}

fn run_capture(cmd: &mut Command) -> Option<String> {
    let o = cmd.output().ok()?;
    if !o.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_rust_toolchain() -> DoctorCheck {
    if let Some(ver) = run_capture(Command::new("rustc").arg("--version")) {
        return DoctorCheck::ok("rust_toolchain", ver);
    }
    if let Some(show) = run_capture(Command::new("rustup").arg("show").arg("active-toolchain")) {
        let line = show.lines().next().unwrap_or(&show).trim();
        return DoctorCheck::ok("rust_toolchain", format!("rustup: {line}"));
    }
    DoctorCheck::fail(
        "rust_toolchain",
        "未检测到 Rust 工具链（rustc / rustup）",
        Some("请安装 https://rustup.rs/".into()),
    )
}

fn check_cargo() -> DoctorCheck {
    if let Some(ver) = run_capture(Command::new("cargo").arg("--version")) {
        DoctorCheck::ok("cargo", ver)
    } else {
        DoctorCheck::fail("cargo", "未检测到 cargo", None)
    }
}

fn check_system_memory() -> DoctorCheck {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory() / (1024 * 1024);
    let avail = sys.available_memory() / (1024 * 1024);
    let msg = format!("{total} MiB total, {avail} MiB available");
    if total < 4096 {
        DoctorCheck::warn(
            "system_memory",
            format!("{msg} — 低于 4 GiB 建议"),
            Some("Monolith 双 release 构建可能吃满内存".into()),
        )
    } else {
        DoctorCheck::ok("system_memory", msg)
    }
}

fn check_disk_space(path: &Path) -> DoctorCheck {
    match fs2::available_space(path) {
        Ok(bytes) => {
            let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let msg = format!("{gb:.1} GiB free ({})", path.display());
            if bytes < 1024 * 1024 * 1024 {
                DoctorCheck::warn(
                    "disk_space",
                    format!("{msg} — 低于 1 GiB"),
                    Some("cargo target 目录可能占满磁盘".into()),
                )
            } else {
                DoctorCheck::ok("disk_space", msg)
            }
        }
        Err(e) => DoctorCheck::warn(
            "disk_space",
            "无法读取可用磁盘空间",
            Some(e.to_string()),
        ),
    }
}

fn check_ollama() -> DoctorCheck {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(3)).build();
    match agent.get("http://127.0.0.1:11434/api/tags").call() {
        Ok(resp) => {
            if resp.status() != 200 {
                return DoctorCheck::fail(
                    "ollama",
                    format!("Ollama 响应 HTTP {}", resp.status()),
                    None,
                );
            }
            let body = resp.into_string().unwrap_or_default();
            let n = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v.get("models").and_then(|m| m.as_array()).map(|a| a.len()))
                .unwrap_or(0);
            DoctorCheck::ok("ollama", format!("已运行，{n} 个模型"))
        }
        Err(e) => DoctorCheck::fail(
            "ollama",
            "Ollama 未运行或不可达（127.0.0.1:11434）",
            Some(format!("{e}；纯 remote LLM 可忽略")),
        ),
    }
}

fn check_network_github() -> DoctorCheck {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(5)).build();
    match agent.get("https://github.com").call() {
        Ok(resp) if resp.status() == 200 => DoctorCheck::ok("network", "https://github.com 可达"),
        Ok(resp) => DoctorCheck::warn(
            "network",
            format!("GitHub 响应 HTTP {}", resp.status()),
            None,
        ),
        Err(e) => DoctorCheck::fail(
            "network",
            "无法访问 https://github.com",
            Some(e.to_string()),
        ),
    }
}

fn check_workspace_writable(path: &Path) -> DoctorCheck {
    let probe_dir = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    fs::create_dir_all(&probe_dir).ok();
    let probe = probe_dir.join(".oclive_doctor_probe");
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            DoctorCheck::ok("workspace_writable", format!("{} 可写", probe_dir.display()))
        }
        Err(e) => DoctorCheck::fail(
            "workspace_writable",
            format!("{} 不可写", probe_dir.display()),
            Some(e.to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_report_serializes() {
        let r = DoctorReport {
            schema_version: 1,
            ok: true,
            checks: vec![DoctorCheck::ok("cargo", "cargo 1.85.0")],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert!(v["checks"].is_array());
    }
}
