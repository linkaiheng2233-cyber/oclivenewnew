//! Construction, inspection, and runtime-selection entry points for the managed llama-server.

use super::{
    configured_model_path, configured_runtime_selection, PerformanceLlmClient,
    PerformanceLlmResourceController, PerformanceLlmStatus,
};

use crate::domain::host_profile::LocalLlmRuntimeProfile;
use crate::domain::ports::LlmClient;
use crate::domain::resource_coordinator::ResourceCoordinator;
use crate::error::{AppError, Result};
use crate::infrastructure::ollama_client::OllamaClient;
use crate::infrastructure::openai_compatible_llm::OpenAiCompatibleLlm;
use crate::infrastructure::performance_request_gate::PerformanceRequestGate;
use crate::infrastructure::resource_adapters::{
    configured_llama_tier_with_default, llama_server_descriptor, ollama_descriptor,
    LlamaRuntimeTier,
};
use crate::infrastructure::resource_snapshot::NvidiaSmiResourceSnapshotSource;
use oclive_kernel_contracts::ResourceAdapterController;
use oclive_kernel_types::ResourceCoordinatorPolicy;
use parking_lot::{Mutex, RwLock};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

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

    pub(super) fn set_active_runtime_tier(&self, tier: LlamaRuntimeTier) {
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

    pub(super) async fn apply_runtime_profile(&self, tier: LlamaRuntimeTier) -> Result<()> {
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
}
