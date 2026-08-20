//! Lightweight environment probe (`init --smart` / recommended presets; not equivalent to a full `doctor`).

use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaProbe {
    Running,
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct EnvironmentProbe {
    pub ollama: OllamaProbe,
    pub ollama_models: u32,
    pub gpu_nvidia: bool,
    pub total_memory_mib: u64,
}

impl EnvironmentProbe {
    pub fn collect() -> Self {
        let (ollama, ollama_models) = probe_ollama();
        Self {
            ollama,
            ollama_models,
            gpu_nvidia: probe_nvidia_gpu(),
            total_memory_mib: probe_total_memory_mib(),
        }
    }
}

fn probe_ollama() -> (OllamaProbe, u32) {
    let agent = crate::http_client::AgentBuilder::new()
        .timeout(Duration::from_secs(3))
        .build();
    match agent.get("http://127.0.0.1:11434/api/tags").call() {
        Ok(resp) if resp.status() == 200 => {
            let n = resp
                .into_string()
                .ok()
                .and_then(|body| {
                    serde_json::from_str::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|v| {
                            v.get("models")
                                .and_then(|m| m.as_array())
                                .map(|a| a.len() as u32)
                        })
                })
                .unwrap_or(0);
            (OllamaProbe::Running, n)
        }
        _ => (OllamaProbe::Unreachable, 0),
    }
}

fn probe_nvidia_gpu() -> bool {
    if Command::new("nvidia-smi")
        .args(["-L"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    std::env::var("CUDA_VISIBLE_DEVICES")
        .ok()
        .is_some_and(|v| !v.trim().is_empty() && v.trim() != "-1")
}

fn probe_total_memory_mib() -> u64 {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    sys.total_memory() / (1024 * 1024)
}

#[derive(Debug, Clone)]
pub struct InitPresetRecommendation {
    pub preset: &'static str,
    pub monolith: bool,
    pub monolith_preset: Option<&'static str>,
    pub rationale: &'static str,
    pub example_cmd: String,
}

pub fn recommend_init(probe: &EnvironmentProbe, project_name: &str) -> InitPresetRecommendation {
    let (preset, monolith, monolith_preset, rationale) = if probe.total_memory_mib < 4096 {
        (
            "minimal",
            false,
            None,
            "系统内存 < 4 GiB：建议 minimal、关闭 Monolith，降低编译与运行时占用",
        )
    } else if probe.ollama == OllamaProbe::Running && probe.ollama_models > 0 {
        let monolith_preset = if probe.gpu_nvidia {
            Some("latency")
        } else {
            Some("memory")
        };
        let rationale = if probe.gpu_nvidia {
            "检测到 Ollama 与 NVIDIA GPU：mixed 预设 + Monolith latency 档适合本地 LLM"
        } else {
            "检测到 Ollama（无 NVIDIA GPU）：mixed 预设；Monolith 建议 memory 档"
        };
        ("mixed", probe.gpu_nvidia, monolith_preset, rationale)
    } else if probe.ollama == OllamaProbe::Running {
        (
            "mixed",
            false,
            None,
            "Ollama 在运行但尚未 pull 模型：mixed 预设；请先 ollama pull，或配置 remote LLM",
        )
    } else {
        (
            "minimal",
            false,
            None,
            "未检测到本机 Ollama：minimal 预设；远程 LLM 请设置 OCLIVE_REMOTE_* 或 init 后改 settings",
        )
    };

    let mut example_cmd = format!(
        "cargo run -p oclive-cli -- init --non-interactive --preset {preset} --project-type kernel-server -o ./out --project-name {project_name}"
    );
    if monolith {
        example_cmd.push_str(" --monolith");
        if let Some(mp) = monolith_preset {
            example_cmd.push_str(&format!(" --monolith-preset {mp}"));
        }
    }

    InitPresetRecommendation {
        preset,
        monolith,
        monolith_preset,
        rationale,
        example_cmd,
    }
}

pub fn print_init_recommendations(probe: &EnvironmentProbe, project_name: &str) {
    let rec = recommend_init(probe, project_name);
    println!("\n—— oclive init 环境推荐（轻量探测，完整诊断请 oclive doctor）——");
    println!(
        "  Ollama: {}",
        match probe.ollama {
            OllamaProbe::Running => format!("运行中（{} 个模型）", probe.ollama_models),
            OllamaProbe::Unreachable => "未检测到（127.0.0.1:11434）".into(),
        }
    );
    println!(
        "  GPU: {}",
        if probe.gpu_nvidia {
            "检测到 NVIDIA（nvidia-smi）"
        } else {
            "未检测到 NVIDIA GPU（仍可用 CPU/Ollama）"
        }
    );
    println!("  内存: {} MiB", probe.total_memory_mib);
    println!();
    println!("  推荐 --preset: {}", rec.preset);
    if rec.monolith {
        println!(
            "  推荐 Monolith: 开启（--monolith-preset {}）",
            rec.monolith_preset.unwrap_or("latency")
        );
    } else {
        println!("  推荐 Monolith: 关闭（标准双进程即可）");
    }
    println!("  理由: {}", rec.rationale);
    println!();
    println!("  一键示例:");
    println!("    {}", rec.example_cmd);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_prefers_mixed_when_ollama_has_models() {
        let probe = EnvironmentProbe {
            ollama: OllamaProbe::Running,
            ollama_models: 2,
            gpu_nvidia: true,
            total_memory_mib: 16384,
        };
        let rec = recommend_init(&probe, "demo");
        assert_eq!(rec.preset, "mixed");
        assert!(rec.monolith);
        assert_eq!(rec.monolith_preset, Some("latency"));
    }

    #[test]
    fn recommend_minimal_without_ollama() {
        let probe = EnvironmentProbe {
            ollama: OllamaProbe::Unreachable,
            ollama_models: 0,
            gpu_nvidia: false,
            total_memory_mib: 8192,
        };
        let rec = recommend_init(&probe, "demo");
        assert_eq!(rec.preset, "minimal");
        assert!(!rec.monolith);
    }
}
