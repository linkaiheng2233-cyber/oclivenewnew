//! Monolith bench 子进程峰值内存采样（跨平台）。

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// 运行 bench 子进程并返回（耗时 ms, 峰值 RSS MiB）。
pub fn run_bench_child_with_peak(bin: &Path, inner_iters: u32) -> Result<(f64, u64)> {
    if !bin.is_file() {
        bail!("找不到二进制: {}", bin.display());
    }
    let mut child = Command::new(bin)
        .env("OCLIVE_KERNEL_BENCH_ITERS", inner_iters.to_string())
        .spawn()
        .with_context(|| format!("spawn {}", bin.display()))?;
    let t0 = Instant::now();
    let peak_bytes = poll_peak_rss(&mut child)?;
    let st = child.wait().with_context(|| format!("wait {}", bin.display()))?;
    if !st.success() {
        bail!("二进制退出失败: {:?}", st.code());
    }
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let peak_mib = peak_bytes / (1024 * 1024);
    Ok((elapsed_ms, peak_mib.max(1)))
}

fn poll_peak_rss(child: &mut Child) -> Result<u64> {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let pid = Pid::from_u32(child.id());
    let mut sys = System::new();
    let mut peak: u64 = 0;
    loop {
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        if let Some(proc) = sys.process(pid) {
            peak = peak.max(proc.memory());
        }
        match child.try_wait()? {
            Some(_) => break,
            None => std::thread::sleep(Duration::from_millis(15)),
        }
    }
    Ok(peak)
}

pub fn binary_file_size(path: &Path) -> Result<u64> {
    let meta = std::fs::metadata(path)
        .with_context(|| format!("stat {}", path.display()))?;
    Ok(meta.len())
}
