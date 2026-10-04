//! Stable, transport-independent facade for embedding the complete OCLive role kernel.
//!
//! This facade is the composition boundary for trusted Rust hosts. It delegates to the same
//! [`AppState`](crate::state::AppState), role service, turn pipeline, Event Ring, persistence, and
//! plugin wiring used by HTTP and Tauri. The additive minimal text entry also delegates to the
//! canonical chat entry, but does not construct a rich role or promise its persistence/extensions.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use oclive_kernel_contracts::{
    EventEmitter, EventModule, EventModuleRegistrar, LlmClient, LlmTokenSink, PromptBase,
};
use oclive_kernel_types::models::dto::{
    MinimalRoleMessageRequest, MinimalRoleMessageResponse, RoleData, RoleInfo, RoleSummary,
    SendMessageRequest, SendMessageResponse, TurnOrigin,
};
use oclive_kernel_types::{
    EventModuleDeclaration, EventModuleRegistryEntry, EventModuleRegistryPolicy,
    EventRingDiagnostics, ProactiveTurnProposal, RuntimeEventTraceDiagnostics,
};

use crate::command_error::CommandError;
use crate::domain::chat_engine::{
    process_message, process_message_stream, process_message_stream_with_origin,
    process_message_with_origin, process_minimal_message, process_minimal_message_with_prompt,
    process_proactive_turn,
};
use crate::domain::event_ring::{propose_proactive_turn, ProactiveTurnPermit};
use crate::domain::host_profile::HostProfile;
use crate::error::AppError;
use crate::service::role::{get_role_info_impl, list_roles_impl, load_role_impl};
use crate::state::{AppState, AppStateBuilder};

pub use crate::domain::chat_engine::message_error::MinimalRoleMessageError;
pub use crate::service::role::minimal::PreparedMinimalRole;

/// Stable error surface for the in-process kernel.
///
/// The inner variants retain the same [`oclive_kernel_types::KernelErrorBody`] codes used by HTTP
/// and Tauri through [`CommandError::kernel_error_body`].
pub type KernelError = CommandError;

/// Result returned by the stable in-process kernel facade.
pub type KernelResult<T> = std::result::Result<T, KernelError>;

/// Filesystem roots required by one complete role-kernel instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcliveKernelConfig {
    app_data_dir: PathBuf,
    roles_dir: PathBuf,
    database_path: PathBuf,
    runtime_event_trace_path: Option<PathBuf>,
}

impl OcliveKernelConfig {
    /// Create a configuration using `app_data_dir/app.db` for persistent kernel state.
    #[must_use]
    pub fn new(app_data_dir: impl AsRef<Path>, roles_dir: impl AsRef<Path>) -> Self {
        let app_data_dir = app_data_dir.as_ref().to_path_buf();
        let database_path = oclive_kernel_runtime::find_db_path(&app_data_dir);
        Self {
            app_data_dir,
            roles_dir: roles_dir.as_ref().to_path_buf(),
            database_path,
            runtime_event_trace_path: None,
        }
    }

    /// Override the SQLite path while keeping plugin/runtime data under `app_data_dir`.
    #[must_use]
    pub fn with_database_path(mut self, database_path: impl AsRef<Path>) -> Self {
        self.database_path = database_path.as_ref().to_path_buf();
        self
    }

    /// Enable the trace-only Runtime Event shadow at an independent default SQLite path.
    #[must_use]
    pub fn with_runtime_event_trace(mut self) -> Self {
        self.runtime_event_trace_path = Some(self.app_data_dir.join("runtime-event-trace.sqlite3"));
        self
    }

    /// Enable the trace-only Runtime Event shadow at an explicit independent SQLite path.
    #[must_use]
    pub fn with_runtime_event_trace_path(mut self, path: impl AsRef<Path>) -> Self {
        self.runtime_event_trace_path = Some(path.as_ref().to_path_buf());
        self
    }

    #[must_use]
    pub fn app_data_dir(&self) -> &Path {
        self.app_data_dir.as_path()
    }

    #[must_use]
    pub fn roles_dir(&self) -> &Path {
        self.roles_dir.as_path()
    }

    #[must_use]
    pub fn database_path(&self) -> &Path {
        self.database_path.as_path()
    }

    /// `None` means the shadow recorder is fully disabled.
    #[must_use]
    pub fn runtime_event_trace_path(&self) -> Option<&Path> {
        self.runtime_event_trace_path.as_deref()
    }
}

/// Builder for a complete in-process OCLive kernel.
pub struct OcliveKernelBuilder {
    config: OcliveKernelConfig,
    host_profile: Option<HostProfile>,
    llm: Option<Arc<dyn LlmClient>>,
}

impl OcliveKernelBuilder {
    #[must_use]
    pub fn new(config: OcliveKernelConfig) -> Self {
        Self {
            config,
            host_profile: None,
            llm: None,
        }
    }

    /// Override environment-based distro policy with an explicit trusted host profile.
    #[must_use]
    pub fn with_host_profile(mut self, host_profile: HostProfile) -> Self {
        self.host_profile = Some(host_profile);
        self
    }

    /// Inject the host's model adapter without changing kernel orchestration or persistence.
    #[must_use]
    pub fn with_llm_client(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Build all production services and verify that the database is reachable.
    ///
    /// # Errors
    ///
    /// Returns role-storage, database, migration, policy, plugin bootstrap, or health-check errors.
    pub async fn build(self) -> KernelResult<OcliveKernel> {
        std::fs::create_dir_all(self.config.app_data_dir()).map_err(AppError::from)?;
        let mut state_builder = AppStateBuilder::production(
            self.config.database_path(),
            self.config.roles_dir().to_path_buf(),
            self.config.app_data_dir(),
        );
        if let Some(trace_path) = self.config.runtime_event_trace_path() {
            state_builder = state_builder.with_runtime_event_trace_path(trace_path);
        }
        if let Some(host_profile) = self.host_profile {
            state_builder = state_builder.with_host_profile(host_profile);
        }
        if let Some(llm) = self.llm {
            state_builder = state_builder.with_llm_client(llm);
        }
        let state = state_builder.build().await?;
        let kernel = OcliveKernel {
            config: self.config,
            state,
        };
        kernel.health_check().await?;
        Ok(kernel)
    }
}

/// Complete role-kernel handle for one in-process host.
///
/// The handle is intentionally not `Clone`: callers may share it behind `Arc`, while consuming
/// [`shutdown`](Self::shutdown) remains an explicit proof that the owner stopped issuing turns.
pub struct OcliveKernel {
    config: OcliveKernelConfig,
    state: AppState,
}

impl OcliveKernel {
    #[must_use]
    pub fn builder(config: OcliveKernelConfig) -> OcliveKernelBuilder {
        OcliveKernelBuilder::new(config)
    }

    /// Start a kernel with environment-selected host policy and model adapter.
    ///
    /// # Errors
    ///
    /// Returns the same startup failures as [`OcliveKernelBuilder::build`].
    pub async fn start(config: OcliveKernelConfig) -> KernelResult<Self> {
        Self::builder(config).build().await
    }

    #[must_use]
    pub fn config(&self) -> &OcliveKernelConfig {
        &self.config
    }

    /// Verify the global database dependency without invoking an LLM or loading a role.
    ///
    /// # Errors
    ///
    /// Returns a database health error when the kernel state cannot be read.
    pub async fn health_check(&self) -> KernelResult<()> {
        crate::domain::startup_health::run_global_db_ping(
            &crate::infrastructure::db_ports::DbHealthPortAdapter(self.state.db_manager.as_ref()),
        )
        .await
        .map_err(Into::into)
    }

    /// List role packs visible to this kernel instance.
    ///
    /// # Errors
    ///
    /// Returns role-pack filesystem or validation errors.
    pub async fn list_roles(&self) -> KernelResult<Vec<RoleSummary>> {
        list_roles_impl(&self.state).await
    }

    /// Activate one role while preserving its persisted portrait emotion.
    ///
    /// # Errors
    ///
    /// Returns role-pack, capability-plan, or persistence errors.
    pub async fn load_role(&self, role_id: &str) -> KernelResult<RoleData> {
        load_role_impl(&self.state, role_id, false).await
    }

    /// Invoke the basic text path on already prepared developer content.
    ///
    /// Reuses the Host's current model settings and assembled model client. No
    /// rich `Role`, role activation, chat persistence, retry or product extension
    /// is created. Asset bytes remain available through the handle for a distro
    /// renderer; this method never decodes or renders them. The Host owner manages
    /// scheduling and cancellation; the Base Prompt future need not be `Send`.
    ///
    /// # Errors
    /// Returns typed Prompt Unsupported for nonempty extra requirements, or the
    /// original Host input/settings/model error. Failures are not fallback replies.
    pub async fn process_minimal_message(
        &self,
        role: &PreparedMinimalRole,
        request: &MinimalRoleMessageRequest,
    ) -> std::result::Result<MinimalRoleMessageResponse, MinimalRoleMessageError> {
        process_minimal_message(&self.state, role, request).await
    }

    /// Invoke the same basic text path with the Host's chosen Prompt Base.
    ///
    /// Supply an implementation that accepts prepared material fragments, not a
    /// prompt already configured with this role's persona. The shared consumer
    /// adds that persona once and passes the request's requirements unchanged.
    /// Choose a capability whose agreement serves the current purpose; this
    /// method does not broaden that capability's supported requirements.
    ///
    /// The original minimal entry remains the default literal implementation.
    /// This additive entry uses the same model settings, one model call, result
    /// and error carriers. The selected local future need not be `Send`; the
    /// caller owns scheduling, cancellation and any capability resource access.
    ///
    /// # Errors
    /// Returns the complete Prompt error or original Host input/settings/model
    /// error. Empty user input is refused before any capability call. There is no
    /// fallback, retry, persistence or synthetic product-extension state.
    ///
    /// ```no_run
    /// use oclive_kernel_contracts::PromptBase;
    /// use oclive_kernel_host::{MinimalRoleMessageError, OcliveKernel, PreparedMinimalRole};
    /// use oclive_kernel_host::models::dto::{MinimalRoleMessageRequest, MinimalRoleMessageResponse};
    ///
    /// async fn send(
    ///     kernel: &OcliveKernel,
    ///     role: &PreparedMinimalRole,
    ///     prompt: &dyn PromptBase,
    /// ) -> Result<MinimalRoleMessageResponse, MinimalRoleMessageError> {
    ///     let request = MinimalRoleMessageRequest {
    ///         user_message: "Hello.".into(),
    ///         requirements: "Preserve the subject.".into(),
    ///     };
    ///     kernel.process_minimal_message_with_prompt(role, &request, prompt).await
    /// }
    /// ```
    pub async fn process_minimal_message_with_prompt(
        &self,
        role: &PreparedMinimalRole,
        request: &MinimalRoleMessageRequest,
        prompt: &dyn PromptBase,
    ) -> std::result::Result<MinimalRoleMessageResponse, MinimalRoleMessageError> {
        process_minimal_message_with_prompt(&self.state, role, request, prompt).await
    }

    /// Borrow this Host's already composed text client as an LLM Base.
    ///
    /// Construction sends nothing. Each polled call uses the current Host model
    /// settings and the same `generate_with_opts` entry as the minimal text path,
    /// with no extra options. Prepared input and normally returned text (including
    /// empty text) pass through unchanged. No persona is implicitly added.
    ///
    /// This retains the assembled client's resource and authorisation wrappers;
    /// it does not build a second Ollama client or change capability selection.
    /// One call invokes that client once, without adapter retries, probes, preload
    /// or chat persistence. Existing settings synchronisation (including its
    /// process environment updates) is retained. The client's own fallback and
    /// transport policy remains its own:
    /// this is not a promise of one underlying provider request or text quality.
    /// The caller owns the operation graph and the borrowed local future's lifetime.
    /// Dropping the future does not prove that provider work stopped.
    ///
    /// Typed authorisation refusals and `RemoteServiceUnavailable` are projected
    /// to Base Unavailable; other Host errors become Failed with their original
    /// diagnostic text. Host errors have no typed timeout/cancellation source here,
    /// so their wording is never used to invent TimedOut or Cancelled. Existing
    /// product entries still return their original Host error carrier.
    ///
    /// ```no_run
    /// use oclive_kernel_contracts::LlmBase;
    /// use oclive_kernel_host::OcliveKernel;
    /// use oclive_kernel_types::{BaseCallError, LlmBaseRequest};
    ///
    /// async fn generate(kernel: &OcliveKernel, input: &str) -> Result<String, BaseCallError> {
    ///     let model = kernel.text_generation_base();
    ///     model.generate(LlmBaseRequest { input }).await
    /// }
    /// ```
    pub fn text_generation_base(&self) -> impl oclive_kernel_contracts::LlmBase + '_ {
        crate::domain::chat_engine::minimal_llm::HostTextGenerationBase { state: &self.state }
    }

    /// Read the effective runtime snapshot for a role and optional isolated session.
    ///
    /// # Errors
    ///
    /// Returns role loading, session, or persistence errors.
    pub async fn role_info(
        &self,
        role_id: &str,
        session_id: Option<&str>,
    ) -> KernelResult<RoleInfo> {
        get_role_info_impl(&self.state, role_id, session_id).await
    }

    /// Execute one ordinary user-origin role turn through the canonical pipeline.
    ///
    /// # Errors
    ///
    /// Returns startup, role, slot, model, post-processing, or persistence errors.
    pub async fn process_message(
        &self,
        request: &SendMessageRequest,
    ) -> KernelResult<SendMessageResponse> {
        process_message(&self.state, request)
            .await
            .map_err(Into::into)
    }

    /// Execute a trusted user, sensor, or system turn through the canonical origin boundary.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::process_message`].
    pub async fn process_message_with_origin(
        &self,
        request: &SendMessageRequest,
        origin: TurnOrigin,
    ) -> KernelResult<SendMessageResponse> {
        process_message_with_origin(&self.state, request, origin)
            .await
            .map_err(Into::into)
    }

    /// Stream main-model tokens while retaining the same post-turn response and side effects.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::process_message`].
    pub async fn process_message_stream(
        &self,
        request: &SendMessageRequest,
        on_token: LlmTokenSink,
    ) -> KernelResult<SendMessageResponse> {
        process_message_stream(&self.state, request, on_token)
            .await
            .map_err(Into::into)
    }

    /// Streaming counterpart of [`Self::process_message_with_origin`].
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::process_message_stream`].
    pub async fn process_message_stream_with_origin(
        &self,
        request: &SendMessageRequest,
        on_token: LlmTokenSink,
        origin: TurnOrigin,
    ) -> KernelResult<SendMessageResponse> {
        process_message_stream_with_origin(&self.state, request, on_token, origin)
            .await
            .map_err(Into::into)
    }

    /// Submit a proactive proposal through a source-bound Event Ring emitter.
    ///
    /// This only performs Event authority/admission. A returned permit must be consumed by
    /// [`Self::process_proactive_turn`] after dispatch has completed.
    ///
    /// # Errors
    ///
    /// Returns Event Ring declaration, validation, or dispatch errors.
    pub async fn propose_proactive_turn(
        &self,
        emitter: &dyn EventEmitter,
        stream_key: &str,
        correlation_id: &str,
        proposal: ProactiveTurnProposal,
    ) -> KernelResult<Option<ProactiveTurnPermit>> {
        propose_proactive_turn(emitter, stream_key, correlation_id, proposal)
            .await
            .map_err(Into::into)
    }

    /// Consume one Event-authorized permit and execute a side-effect-bounded proactive turn.
    ///
    /// # Errors
    ///
    /// Returns role, prompt, model, or post-processing errors.
    pub async fn process_proactive_turn(
        &self,
        permit: ProactiveTurnPermit,
    ) -> KernelResult<SendMessageResponse> {
        process_proactive_turn(&self.state, permit)
            .await
            .map_err(Into::into)
    }

    /// Return privacy-minimized Event Ring registry, runtime, and bounded-history diagnostics.
    #[must_use]
    pub fn event_ring_diagnostics(&self, recent_limit: usize) -> EventRingDiagnostics {
        self.state.event_ring.diagnostics_snapshot(recent_limit)
    }

    /// Return privacy-minimized health counters for the trace-only Runtime Event shadow.
    #[must_use]
    pub fn runtime_event_trace_diagnostics(&self) -> RuntimeEventTraceDiagnostics {
        self.state.runtime_event_trace.diagnostics()
    }

    /// Stop managed child processes and wait for the SQLite pool to close.
    ///
    /// Consuming the handle prevents the owner from issuing additional turns. Dropping a handle
    /// without this call still triggers component-level cleanup, but does not wait for SQLite.
    pub async fn shutdown(self) {
        self.state.directory_plugins.shutdown_all();
        if let Some(performance) = self.state.performance_llm.as_ref() {
            performance.suspend_managed_runtime("role kernel shutdown");
        }
        self.state.runtime_event_trace.shutdown().await;
        self.state.db_manager.pool.close().await;
    }
}

impl EventModuleRegistrar for OcliveKernel {
    fn register_event_module(
        &self,
        module: Arc<dyn EventModule>,
    ) -> std::result::Result<Arc<dyn EventEmitter>, String> {
        self.state.event_ring.register_event_module(module)
    }

    fn register_event_module_with_policy(
        &self,
        module: Arc<dyn EventModule>,
        policy: EventModuleRegistryPolicy,
    ) -> std::result::Result<Arc<dyn EventEmitter>, String> {
        self.state
            .event_ring
            .register_event_module_with_policy(module, policy)
    }

    fn event_module_registry(&self) -> Vec<EventModuleRegistryEntry> {
        self.state.event_ring.event_module_registry()
    }

    fn event_module_declarations(&self) -> Vec<EventModuleDeclaration> {
        self.state.event_ring.event_module_declarations()
    }
}
