//! Loopback port helpers for kernel replace (shared by CLI and desktop).

use std::process::Command;

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

/// Terminate processes listening on the port (best effort).
pub fn terminate_listeners_on_port(port: u16) {
    for pid in find_listener_pids(port) {
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
    }
}
