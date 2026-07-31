//! Distro-managed llama-server runtime with Ollama fallback.
//!
//! Role packs continue to select the logical builtin local LLM slot (`ollama` on the wire).
//! The distro profile may implement that slot as llama-server first and Ollama second.

use crate::domain::host_profile::LocalLlmRuntimeProfile;
use crate::domain::ports::LlmClient;
use crate::domain::resource_coordinator::{configured_gpu_device_index, ResourceCoordinator};
use crate::error::{AppError, Result};
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::openai_compatible_llm::OpenAiCompatibleLlm;
use crate::infrastructure::performance_request_gate::PerformanceRequestGate;
#[cfg(test)]
use crate::infrastructure::performance_request_gate::REQUEST_BLOCKED_RESOURCE_TRANSITION;
use crate::infrastructure::resource_adapters::{
    configured_llama_tier_with_default, llama_server_descriptor, llama_tier, llama_tiers_from,
    ollama_descriptor, LlamaRuntimeTier, COSYVOICE_ADAPTER_ID, LLAMA_RUNTIME_ADAPTER_ID,
    LLAMA_RUNTIME_PROFILE_FULL, OLLAMA_ADAPTER_ID,
};
use crate::infrastructure::resource_snapshot::NvidiaSmiResourceSnapshotSource;
use async_trait::async_trait;
use oclive_kernel_contracts::{
    LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink, ResourceAdapterController,
    ResourceAdapterControllerOutcome,
};
use oclive_kernel_runtime::{find_listener_pids, process_command_line, terminate_process_tree};
use oclive_kernel_types::{
    ResourceAdapterOperation, ResourceAdmissionDecision, ResourceAdmissionMode,
    ResourceAdmissionRequest, ResourceControlMode, ResourceCoordinatorPolicy, ResourcePriority,
};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

impl PerformanceLlmClient {
    #[must_use]
    pub fn resource_controller(self: &Arc<Self>) -> Arc<dyn ResourceAdapterController> {
        Arc::new(PerformanceLlmResourceController {
            client: Arc::clone(self),
        })
    }

    /// Build the performance-mode client. The endpoint is rejected unless it is loopback.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid endpoint or HTTP client construction failure.
    pub fn new(
        profile: LocalLlmRuntimeProfile,
        app_data_dir: PathBuf,
        resource_root: Option<PathBuf>,
        fallback: Arc<dyn LlmClient>,
        ollama_runtime: Option<Arc<OllamaClient>>,
        fallback_model: String,
    ) -> Result<Self> {
        Self::new_with_resource_coordinator(
            profile,
            app_data_dir,
            resource_root,
            fallback,
            ollama_runtime,
            fallback_model,
            Arc::new(ResourceCoordinator::new(
                ResourceCoordinatorPolicy::default(),
                Arc::new(NvidiaSmiResourceSnapshotSource),
            )),
        )
    }

    /// Build the performance-mode client with the host-wide resource coordinator.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid endpoint or HTTP client construction failure.
    pub fn new_with_resource_coordinator(
        profile: LocalLlmRuntimeProfile,
        app_data_dir: PathBuf,
        resource_root: Option<PathBuf>,
        fallback: Arc<dyn LlmClient>,
        ollama_runtime: Option<Arc<OllamaClient>>,
        fallback_model: String,
        resource_coordinator: Arc<ResourceCoordinator>,
    ) -> Result<Self> {
        let timeout = Duration::from_millis(profile.startup_timeout_ms.max(120_000));
        let primary = OpenAiCompatibleLlm::for_local_runtime(&profile.endpoint, timeout)?;
        let health_client = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_millis(500))
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| AppError::InvalidParameter(format!("llama-server health client: {e}")))?;
        let initial_status = PerformanceLlmStatus {
            mode: profile.mode.as_str().into(),
            endpoint: profile.endpoint.clone(),
            ready: false,
            runtime_installed: false,
            model_configured: configured_model_path().is_some(),
            active_backend: "pending".into(),
            detail: "performance runtime has not been probed".into(),
        };
        let mut fallback_models = BTreeSet::new();
        if !fallback_model.trim().is_empty() {
            fallback_models.insert(fallback_model.trim().to_string());
        }
        resource_coordinator
            .register_adapter(llama_server_descriptor())
            .map_err(AppError::InvalidParameter)?;
        if ollama_runtime.is_some() {
            resource_coordinator
                .register_adapter(ollama_descriptor())
                .map_err(AppError::InvalidParameter)?;
        }
        let initial_tier = configured_llama_tier_with_default(&profile.performance_profile);
        Ok(Self {
            profile,
            app_data_dir,
            resource_root,
            primary,
            fallback,
            ollama_runtime,
            fallback_models: Mutex::new(fallback_models),
            resource_coordinator,
            runtime_lease_id: Mutex::new(None),
            health_client,
            start_lock: tokio::sync::Mutex::new(()),
            process: Mutex::new(None),
            selected_runtime: Mutex::new(configured_runtime_selection().ok().flatten()),
            active_tier: RwLock::new(initial_tier),
            retry_after: Mutex::new(None),
            status: RwLock::new(initial_status),
            fallback_warned: AtomicBool::new(false),
            primary_enabled: AtomicBool::new(true),
            request_gate: Arc::new(PerformanceRequestGate::default()),
        })
    }

    #[must_use]
    pub fn status_snapshot(&self) -> PerformanceLlmStatus {
        let mut status = self.status.read().clone();
        status.runtime_installed = self.discover_runtime_binary().is_some();
        status.model_configured = configured_model_path().is_some();
        status
    }

    /// Whether a coordinated resource transition currently owns the local
    /// Performance LLM suspension. This is intentionally separate from
    /// `active_backend == "inactive"` because cloud-provider selection uses the
    /// same display state without transferring GPU ownership.
    #[must_use]
    pub(crate) fn resource_suspension_active(&self) -> bool {
        self.request_gate.is_blocked()
    }

    #[must_use]
    pub(crate) fn active_runtime_tier(&self) -> LlamaRuntimeTier {
        *self.active_tier.read()
    }

    fn set_active_runtime_tier(&self, tier: LlamaRuntimeTier) {
        *self.active_tier.write() = tier;
    }

    /// Whether this client has a runtime class it can actually release before a
    /// foreground media workload retries admission.
    ///
    /// A host-owned llama process is directly controllable. Tracked Ollama
    /// models are also releasable through Ollama's unload API. An unrelated
    /// external llama-server is deliberately not treated as controllable.
    #[must_use]
    pub(crate) fn has_releasable_gpu_residency(&self) -> bool {
        self.process.lock().is_some()
            || (self.ollama_runtime.is_some() && !self.fallback_models.lock().is_empty())
    }

    /// Probe the configured endpoint without starting a process.
    pub async fn inspect(&self) -> PerformanceLlmStatus {
        if self.request_gate.is_blocked() {
            return self.status_snapshot();
        }
        if self.endpoint_ready().await {
            if let Ok(Some(selection)) = configured_runtime_selection() {
                if selection.adapter_path.is_some() && !self.managed_process_matches(&selection) {
                    self.set_status(
                        false,
                        "ollama",
                        "selected LoRA is not loaded by the running external llama-server",
                    );
                    return self.status_snapshot();
                }
            }
            self.set_status(true, "performance", "llama-server is ready");
        } else {
            let runtime = self.discover_runtime_binary().is_some();
            let model = configured_model_path().is_some();
            let detail = match (runtime, model) {
                (false, _) => "llama-server runtime pack is not installed",
                (true, false) => "no GGUF model is selected for performance mode",
                (true, true) => "llama-server is installed but not running",
            };
            self.set_status(false, "ollama", detail);
        }
        self.status_snapshot()
    }

    /// Warm the managed primary in the background. Missing optional packs are non-fatal.
    pub fn schedule_warmup(self: &Arc<Self>) {
        if !self.enable_managed_runtime() {
            tracing::info!(
                target: "oclive_resource",
                reason = "llm_recovery_blocked_by_voice_residency",
                "Performance LLM warmup remains deferred while bundled voice owns its lease"
            );
            return;
        }
        let this = Arc::clone(self);
        tokio::spawn(async move {
            this.request_gate.wait_until_open().await;
            if let Err(error) = this.warmup_primary().await {
                tracing::info!(
                    target: "oclive_llm",
                    error = %error,
                    "performance LLM unavailable during warmup; Ollama remains active"
                );
            }
        });
    }

    /// Ensure the primary is ready without generating text.
    ///
    /// # Errors
    ///
    /// Returns the runtime discovery, spawn, or readiness error. Callers may then warm
    /// the Ollama fallback without loading both GPU runtimes at the same time.
    pub async fn warmup_primary(&self) -> Result<()> {
        self.ensure_primary_ready().await
    }

    /// Re-enable the managed runtime and synchronously apply the current model/LoRA selection.
    ///
    /// # Errors
    ///
    /// Returns the runtime discovery, spawn, adapter-load, or readiness error.
    pub async fn apply_runtime_selection(&self) -> Result<()> {
        if !self.enable_managed_runtime() {
            return Err(AppError::RemoteServiceUnavailable(
                "llm_recovery_blocked_by_voice_residency".into(),
            ));
        }
        self.request_gate.wait_until_open().await;
        self.ensure_primary_ready().await
    }

    async fn apply_runtime_profile(&self, tier: LlamaRuntimeTier) -> Result<()> {
        let current = self.active_runtime_tier();
        let running = self.process.lock().is_some();
        if current != tier && running {
            self.suspend_managed_runtime_for_resource_pressure(
                "resource coordinator requested llama-server profile switch",
            )
            .await?;
        }
        self.set_active_runtime_tier(tier);
        if !self.enable_managed_runtime() {
            return Err(AppError::RemoteServiceUnavailable(
                "llm_recovery_blocked_by_voice_residency".into(),
            ));
        }
        self.request_gate.wait_until_open().await;
        if let Err(error) = self.ensure_primary_ready().await {
            self.set_active_runtime_tier(current);
            return Err(error);
        }
        let selection = configured_runtime_selection()?.ok_or_else(|| {
            AppError::RemoteServiceUnavailable(
                "no GGUF model is selected for performance mode".into(),
            )
        })?;
        if !self.managed_process_matches(&selection) {
            self.set_active_runtime_tier(current);
            return Err(AppError::RemoteServiceUnavailable(
                "resource_profile_external_runtime_uncontrolled".into(),
            ));
        }
        Ok(())
    }

    /// Stop only the llama-server process owned by this host while preserving
    /// ordinary Ollama fallback semantics.
    pub fn suspend_managed_runtime(&self, reason: &str) {
        if !self.resource_suspension_active() {
            self.request_gate.open();
        }
        self.primary_enabled.store(false, Ordering::Release);
        self.process.lock().take();
        self.release_runtime_lease();
        *self.retry_after.lock() = None;
        self.set_status(false, "inactive", reason);
    }

    /// Stop the managed llama-server and prevent Ollama from reclaiming its GPU
    /// allocation during a coordinated resource transition.
    ///
    /// New Performance requests are rejected immediately. Existing primary and
    /// fallback calls drain before the managed primary and tracked Ollama models
    /// are unloaded, so another managed GPU runtime starts only after completion.
    ///
    /// Cancellation before the method returns reopens request admission because
    /// GPU ownership has not yet transferred to the caller. Once the method
    /// returns, the caller owns release confirmation and recovery.
    ///
    /// # Errors
    ///
    /// Returns an error if an explicit provider/runtime recovery supersedes the
    /// transition before request draining and model unload complete.
    pub async fn suspend_managed_runtime_for_resource_pressure(&self, reason: &str) -> Result<()> {
        let (block_attempt, reason) = self.request_gate.begin_block(reason)?;
        let mut cancellation = ResourceSuspensionCancellationGuard {
            client: self,
            armed: true,
        };
        self.primary_enabled.store(false, Ordering::Release);
        *self.retry_after.lock() = None;
        self.set_status(false, "inactive", &reason);
        self.request_gate.wait_until_drained().await;
        self.process.lock().take();
        self.release_runtime_lease();
        self.unload_tracked_ollama_models().await;
        cancellation.disarm();
        if !block_attempt.finish() {
            return Err(AppError::RemoteServiceUnavailable(
                "llm_resource_transition_superseded_by_recovery".into(),
            ));
        }
        Ok(())
    }

    /// Re-open request admission after a coordinated external runtime confirms
    /// that it released GPU residency, then warm the managed primary in the
    /// background.
    #[must_use]
    pub fn resume_managed_runtime_after_resource_pressure(self: &Arc<Self>) -> bool {
        if !self.enable_managed_runtime() {
            return false;
        }
        self.schedule_warmup();
        true
    }

    fn enable_managed_runtime(&self) -> bool {
        if self
            .resource_coordinator
            .has_adapter_lease(COSYVOICE_ADAPTER_ID)
        {
            return false;
        }
        self.request_gate.open();
        self.primary_enabled.store(true, Ordering::Release);
        *self.retry_after.lock() = None;
        true
    }

    pub fn record_fallback_model(&self, model: &str) {
        let model = model.trim();
        if !model.is_empty() {
            self.fallback_models.lock().insert(model.to_string());
        }
    }

    fn set_status(&self, ready: bool, active_backend: &str, detail: &str) {
        let mut status = self.status.write();
        status.ready = ready;
        status.active_backend = active_backend.into();
        status.detail = detail.into();
        status.runtime_installed = self.discover_runtime_binary().is_some();
        status.model_configured = configured_model_path().is_some();
    }

    fn health_url(&self) -> Result<reqwest::Url> {
        let mut url = reqwest::Url::parse(&self.profile.endpoint)
            .map_err(|e| AppError::InvalidParameter(format!("llama-server endpoint: {e}")))?;
        url.set_path("/health");
        url.set_query(None);
        url.set_fragment(None);
        Ok(url)
    }

    async fn endpoint_ready(&self) -> bool {
        let Ok(url) = self.health_url() else {
            return false;
        };
        self.health_client
            .get(url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }

    fn runtime_pack_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![
            self.app_data_dir.join("components").join("llm-runtime"),
            self.app_data_dir.join("llm-runtime"),
        ];
        if let Some(ref root) = self.resource_root {
            roots.push(root.join("components").join("llm-runtime"));
            roots.push(root.join("llm-runtime"));
        }
        roots
    }

    fn binary_from_manifest(root: &Path) -> Option<PathBuf> {
        let raw = std::fs::read_to_string(root.join(RUNTIME_PACK_MANIFEST)).ok()?;
        let manifest: RuntimePackManifest = serde_json::from_str(&raw).ok()?;
        if manifest.schema_version != 1
            || manifest.component_id.trim().is_empty()
            || manifest.component_type != "llm_runtime"
            || manifest.engine != "llama.cpp"
            || semver::Version::parse(manifest.version.trim()).is_err()
        {
            return None;
        }
        let relative = PathBuf::from(manifest.executable);
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return None;
        }
        let binary = root.join(relative);
        if !binary.is_file() || !file_sha256_matches(&binary, manifest.executable_sha256.trim()) {
            return None;
        }
        Some(binary)
    }

    #[must_use]
    pub fn discover_runtime_binary(&self) -> Option<PathBuf> {
        if let Ok(value) = std::env::var(ENV_LLAMA_SERVER_PATH) {
            let path = PathBuf::from(value.trim());
            if path.is_file() {
                return Some(path);
            }
        }
        for root in self.runtime_pack_roots() {
            if let Some(binary) = Self::binary_from_manifest(&root) {
                return Some(binary);
            }
            #[cfg(debug_assertions)]
            {
                #[cfg(target_os = "windows")]
                let conventional = root.join("bin").join("llama-server.exe");
                #[cfg(not(target_os = "windows"))]
                let conventional = root.join("bin").join("llama-server");
                if conventional.is_file() {
                    return Some(conventional);
                }
            }
        }
        None
    }

    fn mark_retry_cooldown(&self, detail: &str) {
        *self.retry_after.lock() =
            Some(Instant::now() + Duration::from_millis(self.profile.retry_cooldown_ms));
        self.set_status(false, "ollama", detail);
    }

    fn degrade_to_ollama(&self, detail: &str) {
        self.process.lock().take();
        self.release_runtime_lease();
        self.mark_retry_cooldown(detail);
    }

    fn release_runtime_lease(&self) {
        if let Some(lease_id) = self.runtime_lease_id.lock().take() {
            self.resource_coordinator.release(&lease_id);
        }
    }

    fn runtime_reservation_mib(selection: &RuntimeSelection, tier: LlamaRuntimeTier) -> u64 {
        if tier.gpu_layers <= 0 {
            return 0;
        }
        if tier.profile_id == LLAMA_RUNTIME_PROFILE_FULL {
            if let Some(configured) = std::env::var(ENV_LLAMA_GPU_RESERVATION_MIB)
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
            {
                return configured.min(65_536);
            }
        }
        let model_mib = std::fs::metadata(&selection.model_path)
            .map(|metadata| metadata.len().saturating_add(1024 * 1024 - 1) / (1024 * 1024))
            .unwrap_or(0);
        let estimated_layer_share = model_mib
            .saturating_mul((tier.gpu_layers as u64).min(32))
            .saturating_add(31)
            / 32;
        estimated_layer_share.saturating_add(512).min(65_536)
    }

    fn runtime_ram_reservation_mib(selection: &RuntimeSelection) -> u64 {
        std::fs::metadata(&selection.model_path)
            .map(|metadata| {
                metadata
                    .len()
                    .saturating_add(1024 * 1024 - 1)
                    .checked_div(1024 * 1024)
                    .unwrap_or(0)
                    .saturating_add(512)
                    .min(65_536)
            })
            .unwrap_or(512)
    }

    async fn reserve_runtime_start(&self, selection: &RuntimeSelection) -> Result<String> {
        let requested_tier = self.active_runtime_tier();
        let mut last_reasons = Vec::new();
        for tier in llama_tiers_from(requested_tier) {
            self.set_active_runtime_tier(tier);
            let reservation_mib = Self::runtime_reservation_mib(selection, tier);
            let admission = self
                .resource_coordinator
                .admit(ResourceAdmissionRequest {
                    adapter_id: LLAMA_RUNTIME_ADAPTER_ID.into(),
                    workload_id: LLAMA_RUNTIME_WORKLOAD_ID.into(),
                    profile_id: Some(tier.profile_id.into()),
                    gpu_device_index: (tier.gpu_layers > 0)
                        .then(configured_gpu_device_index)
                        .flatten(),
                    reservation_mib,
                    ram_reservation_mib: Self::runtime_ram_reservation_mib(selection),
                    cpu_thread_reservation: if tier.gpu_layers > 0 { 2 } else { 4 },
                    priority: ResourcePriority::BackgroundWarmup,
                    control_mode: ResourceControlMode::Managed,
                    admission_mode: ResourceAdmissionMode::Enforced,
                })
                .await;
            if admission.decision == ResourceAdmissionDecision::Denied {
                let capacity_denial = admission.reason_codes.iter().any(|reason| {
                    matches!(
                        reason.as_str(),
                        "insufficient_gpu_headroom"
                            | "gpu_device_unavailable"
                            | "gpu_snapshot_unavailable"
                            | "insufficient_cpu_thread_headroom"
                    )
                });
                last_reasons = admission.reason_codes;
                if capacity_denial {
                    continue;
                }
                self.set_active_runtime_tier(requested_tier);
                return Err(AppError::RemoteServiceUnavailable(format!(
                    "llama-server resource admission denied: {}",
                    last_reasons.join(",")
                )));
            }
            let lease_id = admission.lease.map(|lease| lease.lease_id).ok_or_else(|| {
                AppError::RemoteServiceUnavailable(
                    "llama-server resource admission returned no lease".into(),
                )
            })?;
            if tier != requested_tier {
                tracing::info!(
                    target: "oclive_resource",
                    requested_profile = requested_tier.profile_id,
                    selected_profile = tier.profile_id,
                    gpu_layers = tier.gpu_layers,
                    "llama-server admission selected a lower resource tier"
                );
            }
            *self.runtime_lease_id.lock() = Some(lease_id.clone());
            return Ok(lease_id);
        }
        self.set_active_runtime_tier(requested_tier);
        Err(AppError::RemoteServiceUnavailable(format!(
            "llama-server resource admission denied for every tier: {}",
            last_reasons.join(",")
        )))
    }

    async fn track_ready_managed_runtime(&self, selection: &RuntimeSelection) {
        if self.runtime_lease_id.lock().is_some() {
            return;
        }
        let tier = self.active_runtime_tier();
        let admission = self
            .resource_coordinator
            .admit(ResourceAdmissionRequest {
                adapter_id: LLAMA_RUNTIME_ADAPTER_ID.into(),
                workload_id: LLAMA_RUNTIME_WORKLOAD_ID.into(),
                profile_id: Some(tier.profile_id.into()),
                gpu_device_index: (tier.gpu_layers > 0)
                    .then(configured_gpu_device_index)
                    .flatten(),
                reservation_mib: 0,
                ram_reservation_mib: Self::runtime_ram_reservation_mib(selection),
                cpu_thread_reservation: if tier.gpu_layers > 0 { 2 } else { 4 },
                priority: ResourcePriority::Resident,
                control_mode: ResourceControlMode::Managed,
                admission_mode: ResourceAdmissionMode::ObserveOnly,
            })
            .await;
        if let Some(lease) = admission.lease {
            self.resource_coordinator.activate(
                &lease.lease_id,
                Some(Self::runtime_reservation_mib(selection, tier)),
            );
            *self.runtime_lease_id.lock() = Some(lease.lease_id);
        }
    }

    fn activate_runtime_lease(&self, lease_id: &str, selection: &RuntimeSelection) {
        let tier = self.active_runtime_tier();
        self.resource_coordinator.activate(
            lease_id,
            Some(Self::runtime_reservation_mib(selection, tier)),
        );
    }

    async fn unload_tracked_ollama_models(&self) {
        let Some(ollama) = self.ollama_runtime.as_ref() else {
            return;
        };
        let models: Vec<String> = self.fallback_models.lock().iter().cloned().collect();
        let mut unloaded = 0usize;
        for model in models {
            match ollama.unload(&model).await {
                Ok(()) => unloaded += 1,
                Err(error) => tracing::debug!(
                    target: "oclive_llm",
                    model,
                    %error,
                    "targeted Ollama unload before llama-server primary failed"
                ),
            }
        }
        if unloaded > 0 {
            tracing::info!(
                target: "oclive_llm",
                unloaded,
                "unloaded OCLive Ollama fallback models before llama-server became primary"
            );
        }
    }

    fn retry_is_cooling_down(&self) -> bool {
        self.retry_after
            .lock()
            .as_ref()
            .is_some_and(|until| *until > Instant::now())
    }

    fn reconcile_selected_runtime(&self) -> Result<(Option<RuntimeSelection>, bool)> {
        let selected = configured_runtime_selection()?;
        let selection_changed = {
            let mut previous = self.selected_runtime.lock();
            if *previous == selected {
                false
            } else {
                *previous = selected.clone();
                true
            }
        };
        let running_model_changed = self
            .process
            .lock()
            .as_ref()
            .is_some_and(|running| selected.as_ref() != Some(&running.selection));
        if running_model_changed {
            self.process.lock().take();
            self.release_runtime_lease();
        }
        if selection_changed {
            *self.retry_after.lock() = None;
            self.set_status(
                false,
                "pending",
                "selected GGUF or LoRA changed; managed llama-server will restart",
            );
        }
        Ok((selected, running_model_changed))
    }

    fn managed_process_matches(&self, selection: &RuntimeSelection) -> bool {
        let tier = self.active_runtime_tier();
        self.process.lock().as_mut().is_some_and(|running| {
            running.selection == *selection
                && running.tier == tier
                && running.child.try_wait().ok().flatten().is_none()
        })
    }

    fn endpoint_port(&self) -> Option<u16> {
        reqwest::Url::parse(&self.profile.endpoint)
            .ok()
            .and_then(|url| url.port_or_known_default())
    }

    fn stop_stale_managed_runtime(&self, selection: &RuntimeSelection) -> bool {
        if !self.profile.auto_start {
            return false;
        }
        let Some(binary) = self.discover_runtime_binary() else {
            return false;
        };
        let Some(port) = self.endpoint_port() else {
            return false;
        };
        let mut matched = false;
        for pid in find_listener_pids(port) {
            let Some(command_line) = process_command_line(pid) else {
                continue;
            };
            if !command_line_matches_managed_runtime(
                &command_line,
                &binary,
                selection,
                &self.profile.model_alias,
                port,
                self.active_runtime_tier(),
            ) {
                continue;
            }
            matched = true;
            let terminated = terminate_process_tree(pid);
            tracing::warn!(
                target: "oclive_llm",
                pid,
                port,
                terminated,
                "reclaimed stale OCLive-managed llama-server runtime"
            );
        }
        matched
    }

    async fn wait_for_stopped_endpoint(&self) {
        for _ in 0..20 {
            if !self.endpoint_ready().await {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn accept_ready_endpoint(&self, selection: Option<&RuntimeSelection>) -> Result<()> {
        if let Some(selection) = selection {
            if self.managed_process_matches(selection) {
                self.track_ready_managed_runtime(selection).await;
            } else {
                self.release_runtime_lease();
            }
        }
        self.ensure_primary_enabled()?;
        if self.status.read().active_backend != "performance" {
            self.unload_tracked_ollama_models().await;
        }
        self.set_status(true, "performance", "llama-server is ready");
        Ok(())
    }

    fn spawn_runtime(&self, binary: &Path, selection: &RuntimeSelection) -> Result<Child> {
        let url = reqwest::Url::parse(&self.profile.endpoint)
            .map_err(|e| AppError::InvalidParameter(format!("llama-server endpoint: {e}")))?;
        let host = url.host_str().unwrap_or("127.0.0.1");
        let port = url.port_or_known_default().unwrap_or(8421);
        let log_dir = self.app_data_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("llama-server.log");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        let stdout = log_file
            .as_ref()
            .and_then(|file| file.try_clone().ok())
            .map_or_else(Stdio::null, Stdio::from);
        let stderr = log_file.map_or_else(Stdio::null, Stdio::from);
        let mut command = Command::new(binary);
        let tier = self.active_runtime_tier();
        append_runtime_selection_args(&mut command, selection);
        command
            .arg("--host")
            .arg(host)
            .arg("--port")
            .arg(port.to_string())
            .arg("--alias")
            .arg(&self.profile.model_alias)
            .arg("--n-gpu-layers")
            .arg(tier.gpu_layers.to_string())
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        command.spawn().map_err(|e| {
            AppError::RemoteServiceUnavailable(format!(
                "spawn llama-server {}: {e}",
                binary.display()
            ))
        })
    }

    async fn ensure_primary_ready(&self) -> Result<()> {
        self.ensure_primary_enabled()?;
        let (selection, stopped_managed_runtime) = self.reconcile_selected_runtime()?;
        if stopped_managed_runtime {
            self.wait_for_stopped_endpoint().await;
        }
        if self.endpoint_ready().await {
            let requires_managed_lora = selection.as_ref().is_some_and(|selection| {
                selection.adapter_path.is_some() && !self.managed_process_matches(selection)
            });
            if !requires_managed_lora {
                return self.accept_ready_endpoint(selection.as_ref()).await;
            }
        }

        let _guard = self.start_lock.lock().await;
        self.ensure_primary_enabled()?;
        let (selection, stopped_managed_runtime) = self.reconcile_selected_runtime()?;
        if stopped_managed_runtime {
            self.wait_for_stopped_endpoint().await;
        }
        if self.endpoint_ready().await {
            let requires_managed_lora = selection.as_ref().is_some_and(|selection| {
                selection.adapter_path.is_some() && !self.managed_process_matches(selection)
            });
            if requires_managed_lora {
                let selection = selection.as_ref().ok_or_else(|| {
                    AppError::RemoteServiceUnavailable(
                        "selected runtime disappeared during stale process recovery".into(),
                    )
                })?;
                if !self.stop_stale_managed_runtime(selection) {
                    return Err(AppError::RemoteServiceUnavailable(
                        "selected LoRA cannot be applied to an external llama-server; stop it so OCLive can start the managed runtime".into(),
                    ));
                }
                self.wait_for_stopped_endpoint().await;
                if self.endpoint_ready().await {
                    return Err(AppError::RemoteServiceUnavailable(
                        "stale OCLive-managed llama-server did not stop before restart".into(),
                    ));
                }
                *self.retry_after.lock() = None;
            } else {
                return self.accept_ready_endpoint(selection.as_ref()).await;
            }
        }
        if self.retry_is_cooling_down() {
            return Err(AppError::RemoteServiceUnavailable(
                "llama-server retry cooldown is active".into(),
            ));
        }
        if !self.profile.auto_start {
            self.mark_retry_cooldown("llama-server is not running and auto_start is disabled");
            return Err(AppError::RemoteServiceUnavailable(
                "llama-server is not running".into(),
            ));
        }
        let binary = self.discover_runtime_binary().ok_or_else(|| {
            self.mark_retry_cooldown("llama-server runtime pack is not installed");
            AppError::RemoteServiceUnavailable("llama-server runtime pack is not installed".into())
        })?;
        let selection = selection.ok_or_else(|| {
            self.mark_retry_cooldown("no GGUF model is selected for performance mode");
            AppError::RemoteServiceUnavailable(
                "no GGUF model is selected for performance mode".into(),
            )
        })?;
        self.ensure_primary_enabled()?;
        if self
            .resource_coordinator
            .has_active_adapter(OLLAMA_ADAPTER_ID)
        {
            return Err(AppError::RemoteServiceUnavailable(
                "Ollama is serving another foreground request; managed llama-server start deferred"
                    .into(),
            ));
        }
        self.unload_tracked_ollama_models().await;
        let runtime_lease_id = self.reserve_runtime_start(&selection).await?;

        {
            let mut process = self.process.lock();
            self.ensure_primary_enabled()?;
            let reuse = process.as_mut().is_some_and(|running| {
                running.selection == selection
                    && running.tier == self.active_runtime_tier()
                    && running.child.try_wait().ok().flatten().is_none()
            });
            if !reuse {
                *process = None;
                let child = match self.spawn_runtime(&binary, &selection) {
                    Ok(child) => child,
                    Err(error) => {
                        self.release_runtime_lease();
                        self.mark_retry_cooldown("llama-server process could not be started");
                        return Err(error);
                    }
                };
                *process = Some(SpawnedRuntime {
                    child,
                    selection: selection.clone(),
                    tier: self.active_runtime_tier(),
                });
                tracing::info!(
                    target: "oclive_llm",
                    binary = %binary.display(),
                    model = %selection.model_path.display(),
                    lora = selection.adapter_path.as_ref().map_or_else(
                        || "none".to_string(),
                        |path| path.display().to_string()
                    ),
                    endpoint = %self.profile.endpoint,
                    log = %self.app_data_dir.join("logs").join("llama-server.log").display(),
                    "spawned performance llama-server runtime"
                );
            }
        }

        let deadline =
            Instant::now() + Duration::from_millis(self.profile.startup_timeout_ms.max(1_000));
        while Instant::now() < deadline {
            if let Err(error) = self.ensure_primary_enabled() {
                self.process.lock().take();
                self.release_runtime_lease();
                return Err(error);
            }
            if self.endpoint_ready().await {
                self.activate_runtime_lease(&runtime_lease_id, &selection);
                *self.retry_after.lock() = None;
                self.set_status(true, "performance", "llama-server is ready");
                return Ok(());
            }
            let exited = self
                .process
                .lock()
                .as_mut()
                .and_then(|running| running.child.try_wait().ok().flatten());
            if let Some(status) = exited {
                let detail = format!("llama-server exited during startup: {status}");
                self.process.lock().take();
                self.release_runtime_lease();
                self.mark_retry_cooldown(&detail);
                return Err(AppError::RemoteServiceUnavailable(detail));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        self.process.lock().take();
        self.release_runtime_lease();
        self.mark_retry_cooldown("llama-server startup timed out");
        Err(AppError::RemoteServiceUnavailable(
            "llama-server startup timed out".into(),
        ))
    }

    fn ensure_primary_enabled(&self) -> Result<()> {
        if self.primary_enabled.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(AppError::RemoteServiceUnavailable(
                "managed llama-server is inactive for the selected provider".into(),
            ))
        }
    }

    fn warn_fallback_once(&self, operation: &str, error: &AppError) {
        if self
            .fallback_warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_llm",
                operation,
                error = %error,
                "performance LLM unavailable; falling back to Ollama"
            );
        }
    }

    fn record_fallback(&self, operation: &str, model: Option<&str>, primary_error: &AppError) {
        self.warn_fallback_once(operation, primary_error);
        if let Some(model) = model {
            self.record_fallback_model(model);
        }
    }

    fn fallback_error_with_primary(primary_error: &AppError, fallback_error: AppError) -> AppError {
        let fallback_detail = match fallback_error {
            AppError::OllamaError(detail) => detail,
            other => other.to_string(),
        };
        AppError::OllamaError(format!(
            "Performance LLM primary unavailable: {primary_error}; Ollama fallback unavailable: {fallback_detail}"
        ))
    }

    async fn primary_or_fallback_generate(&self, model: &str, prompt: &str) -> Result<String> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate", Some(model), &error);
            return self
                .fallback
                .generate(model, prompt)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate(&self.profile.model_alias, prompt)
            .await
        {
            Ok(reply) => {
                self.set_status(true, "performance", "llama-server served the last request");
                Ok(reply)
            }
            Err(error) => {
                self.record_fallback("generate", Some(model), &error);
                self.degrade_to_ollama("llama-server request failed");
                self.fallback
                    .generate(model, prompt)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
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

#[async_trait]
impl LlmClient for PerformanceLlmClient {
    fn supports_prefix_cache(&self) -> bool {
        true
    }

    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        self.primary_or_fallback_generate(model, prompt).await
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_tag", Some(model), &error);
            return self
                .fallback
                .generate_tag(model, prompt)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate_tag(&self.profile.model_alias, prompt)
            .await
        {
            Ok(reply) => Ok(reply),
            Err(error) => {
                self.record_fallback("generate_tag", Some(model), &error);
                self.degrade_to_ollama("llama-server tag request failed");
                self.fallback
                    .generate_tag(model, prompt)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_with_opts", Some(model), &error);
            return self
                .fallback
                .generate_with_opts(model, prompt, opts)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        match self
            .primary
            .generate_with_opts(&self.profile.model_alias, prompt, opts)
            .await
        {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                self.record_fallback("generate_with_opts", Some(model), &error);
                self.degrade_to_ollama("llama-server request failed");
                self.fallback
                    .generate_with_opts(model, prompt, opts)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        self.generate_stream_with_opts(model, prompt, on_token, None)
            .await
            .map(|outcome| outcome.reply)
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _request_guard = self.request_gate.try_enter()?;
        if let Err(error) = self.ensure_primary_ready().await {
            self.record_fallback("generate_stream", Some(model), &error);
            return self
                .fallback
                .generate_stream_with_opts(model, prompt, on_token, opts)
                .await
                .map_err(|fallback_error| {
                    Self::fallback_error_with_primary(&error, fallback_error)
                });
        }
        let emitted = Arc::new(AtomicBool::new(false));
        let emitted_for_sink = Arc::clone(&emitted);
        let downstream = Arc::clone(&on_token);
        let guarded_sink: LlmTokenSink = Arc::new(move |token| {
            if !token.is_empty() {
                emitted_for_sink.store(true, Ordering::Release);
            }
            downstream(token);
        });
        match self
            .primary
            .generate_stream_with_opts(&self.profile.model_alias, prompt, guarded_sink, opts)
            .await
        {
            Ok(outcome) => {
                self.set_status(true, "performance", "llama-server served the last stream");
                Ok(outcome)
            }
            Err(error) if !emitted.load(Ordering::Acquire) => {
                self.record_fallback("generate_stream", Some(model), &error);
                self.degrade_to_ollama("llama-server stream failed before first token");
                self.fallback
                    .generate_stream_with_opts(model, prompt, on_token, opts)
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
            Err(error) => {
                self.degrade_to_ollama("llama-server stream failed after emitting content");
                Err(error)
            }
        }
    }

    async fn startup_probe(&self) -> Result<()> {
        let _request_guard = self.request_gate.try_enter()?;
        match self.ensure_primary_ready().await {
            Ok(()) => Ok(()),
            Err(error) => {
                self.record_fallback("startup_probe", None, &error);
                self.fallback
                    .startup_probe()
                    .await
                    .map_err(|fallback_error| {
                        Self::fallback_error_with_primary(&error, fallback_error)
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::MockLlmClient;
    use axum::{
        body::{Body, Bytes},
        response::Response,
        routing::get,
        routing::post,
        Router,
    };
    use futures_util::StreamExt;
    use oclive_kernel_contracts::ResourceSnapshotSource;
    use oclive_kernel_types::{
        CpuSnapshot, GpuDeviceSnapshot, ResourceSnapshot, SystemMemorySnapshot,
    };
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicU64, AtomicUsize};
    use tempfile::tempdir;

    fn profile(endpoint: String) -> LocalLlmRuntimeProfile {
        LocalLlmRuntimeProfile {
            mode: crate::domain::host_profile::LocalLlmRuntimeMode::Performance,
            endpoint,
            auto_start: false,
            startup_timeout_ms: 1_000,
            retry_cooldown_ms: 1_000,
            model_alias: "test-performance".into(),
            performance_profile: "gpu_balanced".into(),
        }
    }

    struct FixedResourceSnapshot(ResourceSnapshot);

    #[async_trait]
    impl ResourceSnapshotSource for FixedResourceSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            self.0.clone()
        }
    }

    #[test]
    fn managed_runtime_passes_selected_lora_to_llama_server() {
        let selection = RuntimeSelection {
            model_path: PathBuf::from("base.gguf"),
            adapter_path: Some(PathBuf::from("adapter.gguf")),
        };
        let mut command = Command::new("llama-server");
        append_runtime_selection_args(&mut command, &selection);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args, ["-m", "base.gguf", "--lora", "adapter.gguf"]);
    }

    #[test]
    fn stale_runtime_signature_requires_every_owned_argument() {
        let binary = PathBuf::from(r"D:\OCLive\components\llama.cpp\llama-server.exe");
        let selection = RuntimeSelection {
            model_path: PathBuf::from(r"D:\OCLive\models\qwen.gguf"),
            adapter_path: Some(PathBuf::from(r"D:\OCLive\models\mumu-adapter.gguf")),
        };
        let tier = llama_tier("gpu_balanced").expect("balanced tier");
        let command_line = format!(
            r#""D:\OCLive\components\llama.cpp\llama-server.exe" -m "D:\OCLive\models\qwen.gguf" --lora "D:\OCLive\models\mumu-adapter.gguf" --host 127.0.0.1 --port 8421 --alias oclive-performance --n-gpu-layers {}"#,
            tier.gpu_layers
        );

        assert!(command_line_matches_managed_runtime(
            &command_line,
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
        assert!(!command_line_matches_managed_runtime(
            &command_line.replace("--alias oclive-performance", "--alias user-server"),
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
        assert!(!command_line_matches_managed_runtime(
            &command_line.replace("mumu-adapter.gguf", "other-adapter.gguf"),
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
        assert!(!command_line_matches_managed_runtime(
            &command_line.replace("--port 8421", "--port 8422"),
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
        assert!(!command_line_matches_managed_runtime(
            &command_line.replace("--port 8421", "--port 84210"),
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
        assert!(!command_line_matches_managed_runtime(
            &command_line.replace(
                "--alias oclive-performance",
                "--alias oclive-performance-user",
            ),
            &binary,
            &selection,
            "oclive-performance",
            8421,
            tier,
        ));
    }

    #[tokio::test]
    async fn managed_runtime_admission_falls_from_balanced_gpu_to_cpu() {
        let dir = tempdir().unwrap();
        let model_path = dir.path().join("test-model.gguf");
        std::fs::write(&model_path, vec![0_u8; 4 * 1024 * 1024]).unwrap();
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(FixedResourceSnapshot(ResourceSnapshot {
                captured_at_ms: 1,
                source: "test".into(),
                available: true,
                gpu_devices: vec![GpuDeviceSnapshot {
                    device_index: 0,
                    name: "constrained".into(),
                    total_mib: 8192,
                    free_mib: 1000,
                    used_mib: 7192,
                }],
                system_memory: Some(SystemMemorySnapshot {
                    total_mib: 16_384,
                    available_mib: 4_000,
                    used_mib: 12_384,
                }),
                cpu: Some(CpuSnapshot {
                    logical_cores: 8,
                    physical_cores: Some(4),
                }),
                reason_codes: Vec::new(),
            })),
        ));
        let client = PerformanceLlmClient::new_with_resource_coordinator(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            Arc::new(MockLlmClient {
                reply: "fallback".into(),
            }),
            None,
            "fallback-model".into(),
            Arc::clone(&coordinator),
        )
        .unwrap();
        client.set_active_runtime_tier(llama_tier("gpu_balanced").unwrap());
        let lease_id = client
            .reserve_runtime_start(&RuntimeSelection {
                model_path,
                adapter_path: None,
            })
            .await
            .unwrap();

        assert_eq!(client.active_runtime_tier().profile_id, "cpu_compatibility");
        let lease = coordinator
            .diagnostics_snapshot()
            .leases
            .into_iter()
            .find(|lease| lease.lease_id == lease_id)
            .unwrap();
        assert_eq!(lease.profile_id.as_deref(), Some("cpu_compatibility"));
        assert_eq!(lease.reservation_mib, 0);
        assert_eq!(lease.cpu_thread_reservation, 4);
        assert!(lease.ram_reservation_mib >= 512);
        client.release_runtime_lease();
    }

    #[tokio::test]
    async fn missing_optional_runtime_falls_back_to_ollama_client() {
        let dir = tempdir().unwrap();
        let fallback: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: "fallback-ok".into(),
        });
        let client = PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();
        assert!(client
            .resource_coordinator
            .diagnostics_snapshot()
            .adapters
            .iter()
            .any(|adapter| adapter.descriptor.adapter_id == LLAMA_RUNTIME_ADAPTER_ID));
        assert_eq!(
            client.generate("fallback-model", "hello").await.unwrap(),
            "fallback-ok"
        );
        assert_eq!(client.status_snapshot().active_backend, "ollama");
    }

    #[tokio::test]
    async fn unavailable_primary_and_fallback_report_both_causes() {
        let dir = tempdir().unwrap();
        let fallback: Arc<dyn LlmClient> = Arc::new(FailingFallback);
        let client = PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();

        let error = client
            .generate("fallback-model", "hello")
            .await
            .unwrap_err();
        let AppError::OllamaError(detail) = error else {
            panic!("expected LLM_ERROR-compatible OllamaError");
        };
        assert!(detail.contains("Performance LLM primary unavailable"));
        assert!(detail.contains("Ollama fallback unavailable"));
        assert!(detail.contains("fallback transport offline"));
    }

    #[tokio::test]
    async fn suspended_runtime_cannot_be_restarted_by_a_queued_request() {
        let primary_calls = Arc::new(AtomicUsize::new(0));
        let primary_calls_for_route = Arc::clone(&primary_calls);
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route(
                "/v1/chat/completions",
                post(move || {
                    let calls = Arc::clone(&primary_calls_for_route);
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        r#"{"choices":[{"message":{"content":"primary"}}]}"#
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let fallback_calls = Arc::new(AtomicUsize::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
            calls: Arc::clone(&fallback_calls),
        });
        let dir = tempdir().unwrap();
        let client = PerformanceLlmClient::new(
            profile(format!("http://{addr}")),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();
        client.suspend_managed_runtime("cloud provider is active");

        assert_eq!(
            client.generate("fallback-model", "hello").await.unwrap(),
            "fallback"
        );
        assert_eq!(primary_calls.load(Ordering::SeqCst), 0);
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
        assert_eq!(client.status_snapshot().active_backend, "inactive");
        server.abort();
    }

    #[tokio::test]
    async fn resource_pressure_suspension_blocks_every_ollama_fallback_entrypoint() {
        let fallback_calls = Arc::new(AtomicUsize::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
            calls: Arc::clone(&fallback_calls),
        });
        let dir = tempdir().unwrap();
        let client = PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();
        client
            .suspend_managed_runtime_for_resource_pressure(
                "bundled voice runtime requires GPU headroom",
            )
            .await
            .unwrap();

        let errors = [
            client
                .generate("fallback-model", "hello")
                .await
                .unwrap_err(),
            client
                .generate_tag("fallback-model", "hello")
                .await
                .unwrap_err(),
            client
                .generate_with_opts("fallback-model", "hello", None)
                .await
                .unwrap_err(),
            client
                .generate_stream_with_opts("fallback-model", "hello", Arc::new(|_| {}), None)
                .await
                .unwrap_err(),
            client.startup_probe().await.unwrap_err(),
        ];
        assert!(errors.iter().all(|error| error
            .to_string()
            .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION)));
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
        assert_eq!(client.status_snapshot().active_backend, "inactive");
        assert_eq!(client.inspect().await.active_backend, "inactive");
        assert!(client
            .suspend_managed_runtime_for_resource_pressure("duplicate suspension")
            .await
            .unwrap_err()
            .to_string()
            .contains("llm_resource_transition_already_active"));

        client.enable_managed_runtime();
        assert_eq!(
            client.generate("fallback-model", "hello").await.unwrap(),
            "fallback"
        );
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn runtime_recovery_refuses_to_overlap_retained_voice_residency() {
        let dir = tempdir().unwrap();
        let client = PerformanceLlmClient::new(
            profile("http://127.0.0.1:9".into()),
            dir.path().to_path_buf(),
            None,
            Arc::new(MockLlmClient {
                reply: "fallback".into(),
            }),
            None,
            "fallback-model".into(),
        )
        .unwrap();
        let voice_lease = client.resource_coordinator.begin_observed_activity(
            COSYVOICE_ADAPTER_ID,
            "retained-after-unconfirmed-release",
            None,
            ResourcePriority::ForegroundMedia,
        );

        assert!(client
            .apply_runtime_selection()
            .await
            .unwrap_err()
            .to_string()
            .contains("llm_recovery_blocked_by_voice_residency"));

        client.resource_coordinator.release(&voice_lease);
        assert!(!client
            .resource_coordinator
            .has_adapter_lease(COSYVOICE_ADAPTER_ID));
    }

    #[test]
    fn runtime_manifest_cannot_escape_component_root() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join(RUNTIME_PACK_MANIFEST),
            r#"{
                "schema_version": 1,
                "component_id": "com.oclive.runtime.llama-cpp",
                "component_type": "llm_runtime",
                "engine": "llama.cpp",
                "version": "1.0.0",
                "executable": "../llama-server.exe",
                "executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }"#,
        )
        .unwrap();
        assert!(PerformanceLlmClient::binary_from_manifest(dir.path()).is_none());
    }

    #[test]
    fn runtime_manifest_accepts_hashed_binary_inside_component_root() {
        let dir = tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let binary = bin_dir.join("llama-server.test");
        std::fs::write(&binary, b"test-runtime").unwrap();
        let hash = format!("{:x}", Sha256::digest(b"test-runtime"));
        std::fs::write(
            dir.path().join(RUNTIME_PACK_MANIFEST),
            format!(
                r#"{{
                    "schema_version": 1,
                    "component_id": "com.oclive.runtime.llama-cpp",
                    "component_type": "llm_runtime",
                    "engine": "llama.cpp",
                    "version": "1.0.0",
                    "executable": "bin/llama-server.test",
                    "executable_sha256": "{hash}"
                }}"#
            ),
        )
        .unwrap();
        assert_eq!(
            PerformanceLlmClient::binary_from_manifest(dir.path()),
            Some(binary)
        );
    }

    struct CountingFallback {
        calls: Arc<AtomicUsize>,
    }

    struct FailingFallback;

    #[async_trait]
    impl LlmClient for FailingFallback {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            Err(AppError::OllamaError("fallback transport offline".into()))
        }

        async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
            self.generate(model, prompt).await
        }
    }

    #[async_trait]
    impl LlmClient for CountingFallback {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok("fallback".into())
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok("fallback".into())
        }

        async fn startup_probe(&self) -> Result<()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct BlockingFallback {
        started: Arc<tokio::sync::Notify>,
        release: Arc<tokio::sync::Semaphore>,
    }

    struct BlockingThenCountingFallback {
        started: Arc<tokio::sync::Notify>,
        release: Arc<tokio::sync::Semaphore>,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl LlmClient for BlockingThenCountingFallback {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                self.started.notify_one();
                let permit = self.release.acquire().await.map_err(|error| {
                    AppError::RemoteServiceUnavailable(format!(
                        "test fallback release semaphore closed: {error}"
                    ))
                })?;
                permit.forget();
            }
            Ok("fallback".into())
        }

        async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
            self.generate(model, prompt).await
        }
    }

    #[async_trait]
    impl LlmClient for BlockingFallback {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            self.started.notify_one();
            let permit = self.release.acquire().await.map_err(|error| {
                AppError::RemoteServiceUnavailable(format!(
                    "test fallback release semaphore closed: {error}"
                ))
            })?;
            permit.forget();
            Ok("fallback".into())
        }

        async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
            self.generate(model, prompt).await
        }
    }

    #[tokio::test]
    async fn resource_pressure_suspension_waits_for_inflight_fallback_to_drain() {
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(BlockingFallback {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        });
        let dir = tempdir().unwrap();
        let client = Arc::new(
            PerformanceLlmClient::new(
                profile("http://127.0.0.1:9".into()),
                dir.path().to_path_buf(),
                None,
                fallback,
                None,
                "fallback-model".into(),
            )
            .unwrap(),
        );

        let started_wait = started.notified();
        let request_client = Arc::clone(&client);
        let request =
            tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
        started_wait.await;

        let suspension_client = Arc::clone(&client);
        let mut suspension = tokio::spawn(async move {
            suspension_client
                .suspend_managed_runtime_for_resource_pressure(
                    "bundled voice runtime requires GPU headroom",
                )
                .await
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut suspension)
                .await
                .is_err(),
            "resource suspension must wait for the active fallback request"
        );

        release.add_permits(1);
        assert_eq!(request.await.unwrap().unwrap(), "fallback");
        suspension.await.unwrap().unwrap();
        assert!(client
            .generate("fallback-model", "blocked")
            .await
            .unwrap_err()
            .to_string()
            .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION));
    }

    #[tokio::test]
    async fn cancelled_suspension_reopens_requests_before_voice_ownership_transfer() {
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let fallback_calls = Arc::new(AtomicUsize::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(BlockingThenCountingFallback {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
            calls: Arc::clone(&fallback_calls),
        });
        let dir = tempdir().unwrap();
        let client = Arc::new(
            PerformanceLlmClient::new(
                profile("http://127.0.0.1:9".into()),
                dir.path().to_path_buf(),
                None,
                fallback,
                None,
                "fallback-model".into(),
            )
            .unwrap(),
        );

        let started_wait = started.notified();
        let request_client = Arc::clone(&client);
        let request =
            tokio::spawn(async move { request_client.generate("fallback-model", "first").await });
        started_wait.await;

        let suspension_client = Arc::clone(&client);
        let suspension = tokio::spawn(async move {
            suspension_client
                .suspend_managed_runtime_for_resource_pressure(
                    "bundled voice runtime requires GPU headroom",
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while !client.resource_suspension_active() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        suspension.abort();
        assert!(suspension.await.unwrap_err().is_cancelled());
        assert!(!client.resource_suspension_active());

        release.add_permits(1);
        assert_eq!(request.await.unwrap().unwrap(), "fallback");
        assert_eq!(
            client.generate("fallback-model", "second").await.unwrap(),
            "fallback"
        );
        assert_eq!(fallback_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn resource_pressure_suspension_does_not_interrupt_inflight_primary_request() {
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let started_for_route = Arc::clone(&started);
        let release_for_route = Arc::clone(&release);
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route(
                "/v1/chat/completions",
                post(move || {
                    let started = Arc::clone(&started_for_route);
                    let release = Arc::clone(&release_for_route);
                    async move {
                        started.notify_one();
                        let permit = release.acquire().await.unwrap();
                        permit.forget();
                        r#"{"choices":[{"message":{"content":"primary"}}]}"#
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let dir = tempdir().unwrap();
        let client = Arc::new(
            PerformanceLlmClient::new(
                profile(format!("http://{addr}")),
                dir.path().to_path_buf(),
                None,
                Arc::new(MockLlmClient {
                    reply: "fallback".into(),
                }),
                None,
                "fallback-model".into(),
            )
            .unwrap(),
        );

        let started_wait = started.notified();
        let request_client = Arc::clone(&client);
        let request =
            tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
        started_wait.await;

        let suspension_client = Arc::clone(&client);
        let mut suspension = tokio::spawn(async move {
            suspension_client
                .suspend_managed_runtime_for_resource_pressure(
                    "bundled voice runtime requires GPU headroom",
                )
                .await
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut suspension)
                .await
                .is_err(),
            "resource suspension must wait for the active primary request"
        );

        release.add_permits(1);
        assert_eq!(request.await.unwrap().unwrap(), "primary");
        suspension.await.unwrap().unwrap();
        assert!(client
            .generate("fallback-model", "blocked")
            .await
            .unwrap_err()
            .to_string()
            .contains(REQUEST_BLOCKED_RESOURCE_TRANSITION));
        server.abort();
    }

    #[tokio::test]
    async fn explicit_recovery_supersedes_a_draining_resource_suspension_without_opening_early() {
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(BlockingFallback {
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        });
        let dir = tempdir().unwrap();
        let client = Arc::new(
            PerformanceLlmClient::new(
                profile("http://127.0.0.1:9".into()),
                dir.path().to_path_buf(),
                None,
                fallback,
                None,
                "fallback-model".into(),
            )
            .unwrap(),
        );

        let started_wait = started.notified();
        let request_client = Arc::clone(&client);
        let request =
            tokio::spawn(async move { request_client.generate("fallback-model", "hello").await });
        started_wait.await;

        let suspension_client = Arc::clone(&client);
        let suspension = tokio::spawn(async move {
            suspension_client
                .suspend_managed_runtime_for_resource_pressure(
                    "bundled voice runtime requires GPU headroom",
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if client.status_snapshot().active_backend == "inactive" {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        client.enable_managed_runtime();
        assert!(
            client.request_gate.is_blocked(),
            "recovery must not reopen fallback while the suspension is still draining"
        );

        release.add_permits(1);
        assert_eq!(request.await.unwrap().unwrap(), "fallback");
        assert!(suspension
            .await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("llm_resource_transition_superseded_by_recovery"));
        assert!(!client.request_gate.is_blocked());

        release.add_permits(1);
        assert_eq!(
            client
                .generate("fallback-model", "recovered")
                .await
                .unwrap(),
            "fallback"
        );
    }

    #[tokio::test]
    async fn stream_failure_after_first_token_does_not_duplicate_via_fallback() {
        async fn broken_stream() -> Response {
            Response::builder()
                .header("content-type", "text/event-stream")
                .body(Body::from(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\
                     data: {broken-json}\n",
                ))
                .unwrap()
        }
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/v1/chat/completions", post(broken_stream));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let calls = Arc::new(AtomicUsize::new(0));
        let fallback: Arc<dyn LlmClient> = Arc::new(CountingFallback {
            calls: Arc::clone(&calls),
        });
        let dir = tempdir().unwrap();
        let client = PerformanceLlmClient::new(
            profile(format!("http://{addr}")),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();
        let emitted = Arc::new(Mutex::new(String::new()));
        let emitted_for_sink = Arc::clone(&emitted);
        let result = client
            .generate_stream(
                "fallback-model",
                "hello",
                Arc::new(move |token| emitted_for_sink.lock().push_str(token)),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(emitted.lock().as_str(), "partial");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        server.abort();
    }

    #[tokio::test]
    async fn performance_stream_emits_first_token_before_generation_finishes() {
        async fn delayed_stream() -> Response {
            let chunks = [
                (
                    Duration::from_millis(10),
                    "data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n\n",
                ),
                (
                    Duration::from_millis(180),
                    "data: {\"choices\":[{\"delta\":{\"content\":\"-second\"}}]}\n\n",
                ),
                (Duration::from_millis(10), "data: [DONE]\n\n"),
            ];
            let stream = futures_util::stream::iter(chunks).then(|(delay, chunk)| async move {
                tokio::time::sleep(delay).await;
                Ok::<Bytes, Infallible>(Bytes::from_static(chunk.as_bytes()))
            });
            Response::builder()
                .header("content-type", "text/event-stream")
                .body(Body::from_stream(stream))
                .unwrap()
        }
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/v1/chat/completions", post(delayed_stream));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let fallback: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: "fallback".into(),
        });
        let dir = tempdir().unwrap();
        let client = PerformanceLlmClient::new(
            profile(format!("http://{addr}")),
            dir.path().to_path_buf(),
            None,
            fallback,
            None,
            "fallback-model".into(),
        )
        .unwrap();
        let started = Instant::now();
        let first_token_ms = Arc::new(AtomicU64::new(0));
        let first_token_for_sink = Arc::clone(&first_token_ms);
        let reply = client
            .generate_stream(
                "fallback-model",
                "hello",
                Arc::new(move |_| {
                    let elapsed = started.elapsed().as_millis() as u64;
                    let _ = first_token_for_sink.compare_exchange(
                        0,
                        elapsed.max(1),
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                }),
            )
            .await
            .unwrap();
        let total_ms = started.elapsed().as_millis() as u64;
        assert_eq!(reply, "first-second");
        assert!(first_token_ms.load(Ordering::SeqCst) + 100 < total_ms);
        server.abort();
    }
}
