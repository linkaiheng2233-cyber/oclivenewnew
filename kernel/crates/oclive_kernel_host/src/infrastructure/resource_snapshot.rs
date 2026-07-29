//! Best-effort device telemetry for the Resource Coordinator.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use oclive_kernel_contracts::ResourceSnapshotSource;
use oclive_kernel_types::{GpuDeviceSnapshot, ResourceSnapshot};

const NVIDIA_SMI_TIMEOUT: Duration = Duration::from_secs(2);

pub struct NvidiaSmiResourceSnapshotSource;

#[async_trait]
impl ResourceSnapshotSource for NvidiaSmiResourceSnapshotSource {
    async fn snapshot(&self) -> ResourceSnapshot {
        tokio::task::spawn_blocking(snapshot_with_nvidia_smi)
            .await
            .unwrap_or_else(|_| {
                ResourceSnapshot::unavailable("nvidia_smi", "gpu_snapshot_task_failed")
            })
    }
}

fn snapshot_with_nvidia_smi() -> ResourceSnapshot {
    let mut child = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,memory.total,memory.free,memory.used",
            "--format=csv,noheader,nounits",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            return ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_unavailable");
        }
    };
    let deadline = Instant::now() + NVIDIA_SMI_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_query_failed");
                }
                let output = child.wait_with_output();
                return match output {
                    Ok(output) => parse_nvidia_smi_output(&String::from_utf8_lossy(&output.stdout)),
                    Err(_) => {
                        ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_output_unavailable")
                    }
                };
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_timeout");
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_wait_failed");
            }
        }
    }
}

fn parse_nvidia_smi_output(raw: &str) -> ResourceSnapshot {
    let gpu_devices = raw
        .lines()
        .filter_map(parse_nvidia_smi_line)
        .collect::<Vec<_>>();
    if gpu_devices.is_empty() {
        return ResourceSnapshot::unavailable("nvidia_smi", "nvidia_smi_no_devices");
    }
    ResourceSnapshot {
        captured_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64),
        source: "nvidia_smi".into(),
        available: true,
        gpu_devices,
        reason_codes: Vec::new(),
    }
}

fn parse_nvidia_smi_line(line: &str) -> Option<GpuDeviceSnapshot> {
    let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
    if fields.len() != 5 {
        return None;
    }
    Some(GpuDeviceSnapshot {
        device_index: fields[0].parse().ok()?,
        name: fields[1].to_string(),
        total_mib: fields[2].parse().ok()?,
        free_mib: fields[3].parse().ok()?,
        used_mib: fields[4].parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_nvidia_devices_without_locale_units() {
        let snapshot = parse_nvidia_smi_output(
            "0, RTX 5060 Laptop, 8151, 3000, 5151\n1, RTX A, 4096, 1024, 3072",
        );
        assert!(snapshot.available);
        assert_eq!(snapshot.gpu_devices.len(), 2);
        assert_eq!(snapshot.gpu_devices[0].free_mib, 3000);
        assert_eq!(snapshot.gpu_devices[1].name, "RTX A");
    }

    #[test]
    fn malformed_output_is_explicitly_unavailable() {
        let snapshot = parse_nvidia_smi_output("not csv");
        assert!(!snapshot.available);
        assert_eq!(snapshot.reason_codes, vec!["nvidia_smi_no_devices"]);
    }
}
