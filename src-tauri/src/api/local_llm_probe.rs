//! 探测本机常见推理进程（Ollama / llama.cpp 系侧车），供前端轮询以刷新模型列表，无需重启应用。

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalLlmRuntimeProbeDto {
    /// 进程名包含 `ollama`（如 `ollama.exe`）。
    pub ollama_process: bool,
    /// 与 Ollama 区分后的 llama.cpp / 侧车等（避免把 `ollama` 误算成 llama）。
    pub llama_like_process: bool,
}

fn classify_process_name(raw: &str) -> (bool, bool) {
    let n = raw.to_lowercase();
    if n.contains("ollama") {
        return (true, false);
    }
    let llama_like = n.contains("llama-server")
        || n.contains("llamacpp")
        || n.contains("oclive-llama")
        || n.contains("oclive_llama")
        || (n.contains("llama") && n.contains("sidecar"));
    (false, llama_like)
}

fn probe_inner() -> LocalLlmRuntimeProbeDto {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new(),
    );
    let mut ollama_process = false;
    let mut llama_like_process = false;
    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy();
        let (o, l) = classify_process_name(name.as_ref());
        if o {
            ollama_process = true;
        }
        if l {
            llama_like_process = true;
        }
    }
    LocalLlmRuntimeProbeDto {
        ollama_process,
        llama_like_process,
    }
}

#[tauri::command]
pub async fn probe_local_llm_runtime() -> Result<LocalLlmRuntimeProbeDto, String> {
    tokio::task::spawn_blocking(probe_inner)
        .await
        .map_err(|e| e.to_string())
}
