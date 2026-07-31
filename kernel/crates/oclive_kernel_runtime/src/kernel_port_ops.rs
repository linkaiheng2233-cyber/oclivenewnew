//! Loopback process helpers for kernel replace (shared by CLI, desktop, and host).

use std::process::{Command, Stdio};

/// PIDs listening on `127.0.0.1:port` (best effort).
#[must_use]
pub fn find_listener_pids(port: u16) -> Vec<u32> {
    #[cfg(windows)]
    {
        let Ok(output) = Command::new("netstat").args(["-ano"]).output() else {
            return Vec::new();
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let mut pids = Vec::new();
        for line in text.lines() {
            if !line.contains(&format!(":{port}")) || !line.contains("LISTENING") {
                continue;
            }
            let parts: Vec<_> = line.split_whitespace().collect();
            if let Some(pid) = parts.last().and_then(|s| s.parse::<u32>().ok()) {
                pids.push(pid);
            }
        }
        pids
    }
    #[cfg(not(windows))]
    {
        let Ok(output) = Command::new("lsof")
            .args(["-ti", &format!(":{port}"), "-sTCP:LISTEN"])
            .output()
        else {
            return Vec::new();
        };
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect()
    }
}

/// Read a process command line (best effort).
#[must_use]
pub fn process_command_line(pid: u32) -> Option<String> {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-CimInstance Win32_Process -Filter \"ProcessId={pid}\").CommandLine"),
            ])
            .output()
            .ok()?;
        let command_line = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!command_line.is_empty()).then_some(command_line)
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "command="])
            .output()
            .ok()?;
        let command_line = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!command_line.is_empty()).then_some(command_line)
    }
}

/// Terminate a process and its descendants (best effort).
///
/// Desktop-owned kernels can spawn managed runtimes such as `llama-server`.
/// Killing only the kernel would otherwise leave GPU-resident orphans behind.
#[must_use]
pub fn terminate_process_tree(pid: u32) -> bool {
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("pkill")
            .args(["-TERM", "-P", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

/// Terminate processes listening on the port (best effort).
pub fn terminate_listeners_on_port(port: u16) {
    for pid in find_listener_pids(port) {
        let _ = terminate_process_tree(pid);
    }
}
