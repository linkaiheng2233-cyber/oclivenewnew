//! Distro-managed llama-server runtime with Ollama fallback.
//!
//! Role packs continue to select the logical builtin local LLM slot (`ollama` on the wire).
//! The distro profile may implement that slot as llama-server first and Ollama second.

mod client;
mod llm_client;
mod runtime;
#[cfg(test)]
mod tests;

use crate::domain::host_profile::LocalLlmRuntimeProfile;
use crate::domain::ports::LlmClient;
use crate::domain::resource_coordinator::ResourceCoordinator;
use crate::error::{AppError, Result};
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::openai_compatible_llm::OpenAiCompatibleLlm;
use crate::infrastructure::performance_request_gate::PerformanceRequestGate;
use crate::infrastructure::resource_adapters::{
    llama_tier, LlamaRuntimeTier, LLAMA_RUNTIME_ADAPTER_ID,
};
use async_trait::async_trait;
use oclive_kernel_contracts::{ResourceAdapterController, ResourceAdapterControllerOutcome};
use oclive_kernel_types::ResourceAdapterOperation;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub const ENV_LLAMA_SERVER_PATH: &str = "OCLIVE_LLAMA_SERVER_PATH";
pub const ENV_LOCAL_LLM_MODEL_PATH: &str = "OCLIVE_LOCAL_LLM_MODEL_PATH";
pub const ENV_LOCAL_LLM_LORA_PATH: &str = "OCLIVE_LOCAL_LLM_LORA_PATH";
pub const ENV_LLAMA_GPU_RESERVATION_MIB: &str = "OCLIVE_LLAMA_GPU_RESERVATION_MIB";
pub const RUNTIME_PACK_MANIFEST: &str = "llm_runtime_pack.json";
const LLAMA_RUNTIME_WORKLOAD_ID: &str = "managed-runtime";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceLlmStatus {
    pub mode: String,
    pub endpoint: String,
    pub ready: bool,
    pub runtime_installed: bool,
    pub model_configured: bool,
    pub active_backend: String,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
struct RuntimePackManifest {
    schema_version: u32,
    component_id: String,
    component_type: String,
    engine: String,
    version: String,
    executable: String,
    executable_sha256: String,
}

struct SpawnedRuntime {
    child: Child,
    selection: RuntimeSelection,
    tier: LlamaRuntimeTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSelection {
    model_path: PathBuf,
    adapter_path: Option<PathBuf>,
}

fn append_runtime_selection_args(command: &mut Command, selection: &RuntimeSelection) {
    command.arg("-m").arg(&selection.model_path);
    if let Some(adapter_path) = selection.adapter_path.as_ref() {
        command.arg("--lora").arg(adapter_path);
    }
}

fn normalize_process_command(value: &str) -> String {
    value
        .replace('"', "")
        .replace('/', "\\")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalized_command_has_pair(command_line: &str, flag: &str, value: &str) -> bool {
    let padded_command = format!(" {command_line} ");
    let pair = format!(
        " {} {} ",
        normalize_process_command(flag),
        normalize_process_command(value)
    );
    padded_command.contains(&pair)
}

fn command_line_matches_managed_runtime(
    command_line: &str,
    binary: &Path,
    selection: &RuntimeSelection,
    model_alias: &str,
    port: u16,
    tier: LlamaRuntimeTier,
) -> bool {
    let command_line = normalize_process_command(command_line);
    let required = [
        normalize_process_command(&binary.display().to_string()),
        normalize_process_command(&selection.model_path.display().to_string()),
    ];
    required.iter().all(|part| command_line.contains(part))
        && normalized_command_has_pair(&command_line, "--alias", model_alias)
        && normalized_command_has_pair(&command_line, "--port", &port.to_string())
        && normalized_command_has_pair(
            &command_line,
            "--n-gpu-layers",
            &tier.gpu_layers.to_string(),
        )
        && selection.adapter_path.as_ref().is_none_or(|adapter| {
            command_line.contains(&normalize_process_command(&adapter.display().to_string()))
        })
}

impl Drop for SpawnedRuntime {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Managed llama-server primary plus the existing builtin Ollama client as fallback.
pub struct PerformanceLlmClient {
    profile: LocalLlmRuntimeProfile,
    app_data_dir: PathBuf,
    resource_root: Option<PathBuf>,
    primary: OpenAiCompatibleLlm,
    fallback: Arc<dyn LlmClient>,
    ollama_runtime: Option<Arc<OllamaClient>>,
    fallback_models: Mutex<BTreeSet<String>>,
    resource_coordinator: Arc<ResourceCoordinator>,
    runtime_lease_id: Mutex<Option<String>>,
    health_client: reqwest::Client,
    start_lock: tokio::sync::Mutex<()>,
    process: Mutex<Option<SpawnedRuntime>>,
    selected_runtime: Mutex<Option<RuntimeSelection>>,
    active_tier: RwLock<LlamaRuntimeTier>,
    retry_after: Mutex<Option<Instant>>,
    status: RwLock<PerformanceLlmStatus>,
    fallback_warned: AtomicBool,
    primary_enabled: AtomicBool,
    request_gate: Arc<PerformanceRequestGate>,
}

struct PerformanceLlmResourceController {
    client: Arc<PerformanceLlmClient>,
}

#[async_trait]
impl ResourceAdapterController for PerformanceLlmResourceController {
    fn adapter_id(&self) -> &str {
        LLAMA_RUNTIME_ADAPTER_ID
    }

    async fn transition(
        &self,
        operation: ResourceAdapterOperation,
        profile_id: Option<&str>,
        reason: Option<&str>,
    ) -> Result<ResourceAdapterControllerOutcome> {
        let requested_tier = profile_id
            .map(|profile_id| {
                llama_tier(profile_id).ok_or_else(|| {
                    AppError::InvalidParameter("resource_profile_unregistered".into())
                })
            })
            .transpose()?;
        match operation {
            ResourceAdapterOperation::Suspend => {
                let already_in_state = self.client.resource_suspension_active();
                if !already_in_state {
                    self.client
                        .suspend_managed_runtime_for_resource_pressure(
                            reason.unwrap_or("resource coordinator requested suspension"),
                        )
                        .await?;
                }
                Ok(ResourceAdapterControllerOutcome {
                    already_in_state,
                    recovery_scheduled: false,
                })
            }
            ResourceAdapterOperation::Resume => {
                let profile_changed =
                    requested_tier.is_some_and(|tier| tier != self.client.active_runtime_tier());
                if let Some(tier) = requested_tier {
                    self.client.set_active_runtime_tier(tier);
                }
                let already_in_state =
                    !self.client.resource_suspension_active() && !profile_changed;
                let recovery_scheduled = if already_in_state {
                    false
                } else {
                    self.client.resume_managed_runtime_after_resource_pressure()
                };
                if !already_in_state && !recovery_scheduled {
                    return Err(AppError::RemoteServiceUnavailable(
                        "llm_recovery_blocked_by_voice_residency".into(),
                    ));
                }
                Ok(ResourceAdapterControllerOutcome {
                    already_in_state,
                    recovery_scheduled,
                })
            }
            ResourceAdapterOperation::Start => {
                let already_in_state = requested_tier.is_none_or(|tier| {
                    tier == self.client.active_runtime_tier()
                        && !self.client.resource_suspension_active()
                        && self.client.status_snapshot().ready
                });
                if !already_in_state {
                    self.client
                        .apply_runtime_profile(
                            requested_tier.unwrap_or_else(|| self.client.active_runtime_tier()),
                        )
                        .await?;
                }
                Ok(ResourceAdapterControllerOutcome {
                    already_in_state,
                    recovery_scheduled: false,
                })
            }
            ResourceAdapterOperation::Observe
            | ResourceAdapterOperation::Unload
            | ResourceAdapterOperation::Release => Err(AppError::InvalidParameter(
                "resource_transition_operation_unsupported".into(),
            )),
        }
    }
}

struct ResourceSuspensionCancellationGuard<'a> {
    client: &'a PerformanceLlmClient,
    armed: bool,
}

impl ResourceSuspensionCancellationGuard<'_> {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for ResourceSuspensionCancellationGuard<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.client.enable_managed_runtime() {
            self.client.set_status(
                false,
                "pending",
                "resource transition cancelled before GPU ownership transfer",
            );
        }
    }
}

fn file_sha256_matches(path: &Path, expected: &str) -> bool {
    let expected = expected.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => hasher.update(&buffer[..read]),
            Err(_) => return false,
        }
    }
    format!("{:x}", hasher.finalize()) == expected
}

fn configured_model_path() -> Option<PathBuf> {
    let value = std::env::var(ENV_LOCAL_LLM_MODEL_PATH).ok()?;
    let path = PathBuf::from(value.trim());
    let valid_extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf") || ext.eq_ignore_ascii_case("bin"));
    (path.is_file() && valid_extension).then_some(path)
}

fn configured_lora_path() -> Result<Option<PathBuf>> {
    let Some(value) = std::env::var(ENV_LOCAL_LLM_LORA_PATH)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let path = PathBuf::from(value);
    let valid_extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"));
    if !path.is_file() || !valid_extension {
        return Err(AppError::InvalidParameter(
            "configured llama.cpp LoRA GGUF is missing or invalid".into(),
        ));
    }
    Ok(Some(path))
}

fn configured_runtime_selection() -> Result<Option<RuntimeSelection>> {
    let Some(model_path) = configured_model_path() else {
        return Ok(None);
    };
    Ok(Some(RuntimeSelection {
        model_path,
        adapter_path: configured_lora_path()?,
    }))
}
