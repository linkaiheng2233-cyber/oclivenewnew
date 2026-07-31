//! Loopback port helpers for kernel replace (delegates to shared runtime).

use oclive_kernel_runtime::KernelCandidate;
use oclive_kernel_runtime::{
    find_listener_pids as runtime_find_pids, process_command_line,
    terminate_listeners_on_port as runtime_terminate,
};

/// True when listeners look like OCLive-managed kernels (not ad-hoc shells).
pub fn is_known_distribution_kernel(port: u16, candidates: &[KernelCandidate]) -> bool {
    let pids = runtime_find_pids(port);
    if pids.is_empty() {
        return true;
    }
    let known: Vec<String> = candidates
        .iter()
        .map(|c| c.binary.display().to_string().to_lowercase())
        .collect();

    for pid in pids {
        let Some(cmd) = process_command_line(pid) else {
            continue;
        };
        let lower = cmd.to_lowercase();
        if known.iter().any(|k| lower.contains(k)) {
            return true;
        }
        if lower.contains("oclive-kernel-server")
            || lower.contains("oclivenewnew-tauri")
            || lower.contains(&format!(
                "{}oclive{}runtime{}",
                MAIN_SEP, MAIN_SEP, MAIN_SEP
            ))
        {
            return true;
        }
    }
    false
}

pub fn terminate_listeners_on_port(port: u16) {
    runtime_terminate(port);
}

#[cfg(windows)]
const MAIN_SEP: &str = "\\";
#[cfg(not(windows))]
const MAIN_SEP: &str = "/";
