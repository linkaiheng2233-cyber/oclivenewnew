//! Managed llama-server process lifecycle: discovery, spawning, leases, and readiness.

use super::{
    append_runtime_selection_args, command_line_matches_managed_runtime, configured_model_path,
    configured_runtime_selection, file_sha256_matches, PerformanceLlmClient,
    ResourceSuspensionCancellationGuard, RuntimePackManifest, RuntimeSelection, SpawnedRuntime,
    ENV_LLAMA_GPU_RESERVATION_MIB, ENV_LLAMA_SERVER_PATH, LLAMA_RUNTIME_WORKLOAD_ID,
    RUNTIME_PACK_MANIFEST,
};

use crate::domain::resource_coordinator::configured_gpu_device_index;
use crate::error::{AppError, Result};
use crate::infrastructure::background_process::configure_background_process;
use crate::infrastructure::resource_adapters::{
    llama_tiers_from, LlamaRuntimeTier, COSYVOICE_ADAPTER_ID, LLAMA_RUNTIME_ADAPTER_ID,
    LLAMA_RUNTIME_PROFILE_FULL, OLLAMA_ADAPTER_ID,
};
use oclive_kernel_runtime::{find_listener_pids, process_command_line, terminate_process_tree};
use oclive_kernel_types::{
    ResourceAdmissionDecision, ResourceAdmissionMode, ResourceAdmissionRequest,
    ResourceControlMode, ResourcePriority,
};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl PerformanceLlmClient {
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

    pub(super) fn enable_managed_runtime(&self) -> bool {
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

    pub(super) fn set_status(&self, ready: bool, active_backend: &str, detail: &str) {
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

    pub(super) async fn endpoint_ready(&self) -> bool {
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

    pub(super) fn binary_from_manifest(root: &Path) -> Option<PathBuf> {
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

    pub(super) fn degrade_to_ollama(&self, detail: &str) {
        self.process.lock().take();
        self.release_runtime_lease();
        self.mark_retry_cooldown(detail);
    }

    pub(super) fn release_runtime_lease(&self) {
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

    pub(super) async fn reserve_runtime_start(
        &self,
        selection: &RuntimeSelection,
    ) -> Result<String> {
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

    pub(super) fn managed_process_matches(&self, selection: &RuntimeSelection) -> bool {
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
        configure_background_process(&mut command);
        command.spawn().map_err(|e| {
            AppError::RemoteServiceUnavailable(format!(
                "spawn llama-server {}: {e}",
                binary.display()
            ))
        })
    }

    pub(super) async fn ensure_primary_ready(&self) -> Result<()> {
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
}
