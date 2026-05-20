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

    /// 对可自动修复项执行修复（交互确认）
    #[arg(long)]
    pub fix: bool,

    /// 与 `--fix` 联用：跳过确认
    #[arg(long)]
    pub yes: bool,
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
    #[serde(skip)]
    pub fix_command: Option<Vec<String>>,
}

impl DoctorCheck {
    fn ok(id: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "ok".into(),
            message: message.into(),
            detail: None,
            fix_command: None,
        }
    }

    fn warn(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "warn".into(),
            message: message.into(),
            detail,
            fix_command: None,
        }
    }

    fn fail(id: &str, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            id: id.into(),
            status: "fail".into(),
            message: message.into(),
            detail,
            fix_command: None,
        }
    }
}

pub fn run(args: DoctorArgs) -> Result<()> {
    let root = if args.path.is_absolute() {
        args.path.clone()
    } else {
        std::env::current_dir()?.join(&args.path)
    };
    let mut checks = vec![
        check_rust_toolchain(),
        check_cargo(),
        check_cpp_toolchain(),
        check_system_memory(),
        check_disk_space(&root),
        check_ollama(),
        check_network_github(),
        check_workspace_writable(&root),
    ];
    if args.fix {
        apply_fixes(&checks, args.yes)?;
        checks = vec![
            check_rust_toolchain(),
            check_cargo(),
            check_cpp_toolchain(),
            check_system_memory(),
            check_disk_space(&root),
            check_ollama(),
            check_network_github(),
            check_workspace_writable(&root),
        ];
    }
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

fn apply_fixes(checks: &[DoctorCheck], yes: bool) -> Result<()> {
    use dialoguer::Confirm;
    for c in checks {
        let Some(cmd) = &c.fix_command else { continue };
        let label = cmd.join(" ");
        let run_it = if yes {
            true
        } else {
            Confirm::new()
                .with_prompt(format!("修复 [{}]: 运行 `{label}`？", c.id))
                .default(true)
                .interact()?
        };
        if !run_it {
            continue;
        }
        if c.id == "disk_space" || c.id == "workspace_writable" || c.id == "cpp_toolchain" {
            println!("  → {}", c.detail.as_deref().unwrap_or(&label));
            continue;
        }
        let st = Command::new(&cmd[0]).args(&cmd[1..]).status();
        match st {
            Ok(s) if s.success() => println!("  ✓ 已执行: {label}"),
            Ok(s) => println!("  ⚠ 命令退出码 {:?}: {label}", s.code()),
            Err(e) => println!("  ⚠ 无法执行: {e}"),
        }
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
        if let Some((maj, min)) = parse_rustc_version(&ver) {
            if maj < 1 || (maj == 1 && min < 70) {
                return DoctorCheck {
                    id: "rust_toolchain".into(),
                    status: "warn".into(),
                    message: format!("{ver} — 建议 Rust 1.70+"),
                    detail: Some("可运行 rustup update stable".into()),
                    fix_command: Some(vec![
                        "rustup".into(),
                        "update".into(),
                        "stable".into(),
                    ]),
                };
            }
        }
        return DoctorCheck::ok("rust_toolchain", ver);
    }
    if let Some(show) = run_capture(Command::new("rustup").arg("show").arg("active-toolchain")) {
        let line = show.lines().next().unwrap_or(&show).trim();
        return DoctorCheck::ok("rust_toolchain", format!("rustup: {line}"));
    }
    DoctorCheck {
        id: "rust_toolchain".into(),
        status: "fail".into(),
        message: "未检测到 Rust 工具链（rustc / rustup）".into(),
        detail: Some("请安装 https://rustup.rs/".into()),
        fix_command: None,
    }
}

fn parse_rustc_version(ver: &str) -> Option<(u32, u32)> {
    let part = ver.split_whitespace().nth(1)?;
    let mut it = part.split('.');
    let maj: u32 = it.next()?.parse().ok()?;
    let min: u32 = it.next()?.parse().ok()?;
    Some((maj, min))
}

fn check_cargo() -> DoctorCheck {
    if let Some(ver) = run_capture(Command::new("cargo").arg("--version")) {
        DoctorCheck::ok("cargo", ver)
    } else {
        DoctorCheck {
            id: "cargo".into(),
            status: "fail".into(),
            message: "未检测到 cargo".into(),
            detail: if cfg!(windows) {
                Some("请安装 Rust: https://rustup.rs/ 或 Visual Studio Build Tools".into())
            } else {
                Some("请安装: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into())
            },
            fix_command: None,
        }
    }
}

fn check_cpp_toolchain() -> DoctorCheck {
    if cfg!(windows) {
        if run_capture(&mut Command::new("cl")).is_some() {
            return DoctorCheck::ok("cpp_toolchain", "MSVC cl 可用");
        }
        return DoctorCheck {
            id: "cpp_toolchain".into(),
            status: "warn".into(),
            message: "未检测到 MSVC cl（部分 native 依赖可能需要）".into(),
            detail: Some(
                "请安装 Visual Studio Build Tools（C++ 工作负载）或 rustup default stable-msvc"
                    .into(),
            ),
            fix_command: None,
        };
    }
    if run_capture(Command::new("cc").arg("--version")).is_some()
        || run_capture(Command::new("g++").arg("--version")).is_some()
    {
        return DoctorCheck::ok("cpp_toolchain", "C/C++ 编译器可用");
    }
    DoctorCheck {
        id: "cpp_toolchain".into(),
        status: "warn".into(),
        message: "未检测到 cc/g++".into(),
        detail: Some("Linux: sudo apt install build-essential · macOS: xcode-select --install".into()),
        fix_command: None,
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
                DoctorCheck {
                    id: "disk_space".into(),
                    status: "warn".into(),
                    message: format!("{msg} — 低于 1 GiB"),
                    detail: Some(
                        "建议: 清理 target/、bench_results/ 或增大磁盘（无法自动删除）".into(),
                    ),
                    fix_command: Some(vec!["echo".into(), "manual_cleanup".into()]),
                }
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
        Err(e) => DoctorCheck {
            id: "ollama".into(),
            status: "fail".into(),
            message: "Ollama 未运行或不可达（127.0.0.1:11434）".into(),
            detail: Some(format!("{e}；纯 remote LLM 可忽略")),
            fix_command: if cfg!(windows) {
                Some(vec![
                    "powershell".into(),
                    "-Command".into(),
                    "Start-Process ollama -ArgumentList serve".into(),
                ])
            } else {
                Some(vec!["sh".into(), "-c".into(), "ollama serve &".into()])
            },
        },
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
        Err(e) => DoctorCheck {
            id: "workspace_writable".into(),
            status: "fail".into(),
            message: format!("{} 不可写", probe_dir.display()),
            detail: Some(format!(
                "{e}；请检查目录权限或换可写路径（Windows: 以管理员运行或修改 ACL）"
            )),
            fix_command: Some(vec!["echo".into(), "fix_permissions".into()]),
        },
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
