//! # oclive_kernel_contracts — reference-runtime port layer
//!
//! **Role**: declares ports used by the complete reference runtime: the six slot contracts
//! and additional Host, facility, and adapter ports. Its full export surface is not the
//! small Kernel's minimal contract. Responsibility boundaries are maintained in the
//! repository's `handoff/MODULE_MAP_AND_HANDOFF.md`.
//!
//! **Current contents**: trait declarations, shared option/result types, default method
//! bodies, and convenience helpers, including [`LlmGenerateOpts::interactive`]. This crate
//! does not assemble the Host's complete turn workflow or its persistence services.
//!
//! **Dependencies**: [`oclive_kernel_types`] supplies shared data and errors; other support
//! dependencies are listed in `Cargo.toml`.
//! **Consumers**: `oclive_kernel_runtime` supplies builtin implementations;
//! `oclive_kernel_host` supplies additional implementations and resource adapters, and
//! binds the ports. Some runtime/Host modules retain compatibility re-exports; new port
//! imports should use this crate directly.
//!
//! **Key decision**: decoupling the traits from Tauri makes it easy to inject mocks for headless services, embedded use, or tests; plugin authors implement the traits in this crate rather than editing the orchestration code directly.
//!
//! ## Reference-runtime port responsibilities at a glance (single responsibility)
//!
//! | Trait | Responsibility | Typical implementer |
//! |-------|------|------------|
//! | [`LlmClient`] | Text generation / streaming | Ollama, Remote HTTP, directory plugin |
//! | [`MemoryRetrieval`] | Recent memory ranking and filtering | Builtin, Remote |
//! | [`UserEmotionAnalyzer`] | User message emotion | Builtin, Remote |
//! | [`EventEstimator`] | Event type and impact | Builtin + LLM, Remote |
//! | [`EventModule`] | Transform or supplement Event Ring envelopes | Builtin, host extension |
//! | [`PromptAssembler`] | Prompt fragment assembly | Builtin, Remote |
//! | [`ComplexEmotionProvider`] | `narrative_hint` resolution | Builtin keyword, Remote |
//! | [`AgentProvider`] | Agent turn short-circuit | Builtin ReAct, directory |
//! | [`PluginHostPort`] | Resolve `plugin_backends` → `dyn` handles | `PluginHost` (Tauri / headless) |
//! | [`SlotRegistryResolver`] | `slot_registry` multi-instance → `ResolvedRoleSlots` | `SlotResolver` |
//! | [`MemoryRepository`] / [`FavorabilityRepository`] | Persistence ports | SQL implementation |
//! | [`EmotionPolicy`] / [`MemoryPolicy`] / [`EventPolicy`] | Post-turn policies (retain/filter) | Builtin rules |
//!
//! Every trait method should have an orchestration caller or a documented reservation note; before adding a method, cross-check the `co_present` / `process_message` hot path.

pub(crate) mod agent_mcp_registry_port;
pub(crate) mod agent_provider;
pub(crate) mod complex_emotion;
pub(crate) mod event_estimator;
pub(crate) mod event_ring;
pub(crate) mod function_calling_parser;
pub(crate) mod llm;
pub(crate) mod local_plugin_bridge;
pub(crate) mod local_plugin_registry_port;
pub(crate) mod mcp_bridge;
pub(crate) mod memory_backend_port;
pub(crate) mod memory_retrieval;
pub(crate) mod plugin_backend_registry;
pub(crate) mod plugin_host;
pub(crate) mod policy;
pub(crate) mod prompt_assembler;
pub mod reply_post_processor;
pub(crate) mod repository;
pub(crate) mod resource_coordination;
pub(crate) mod slot_backend_factory;
pub(crate) mod slot_resolver;
pub mod theater_director;
pub(crate) mod user_emotion_analyzer;
pub(crate) mod user_llm_secrets;

pub use agent_mcp_registry_port::AgentMcpRegistryPort;
pub use agent_provider::AgentProvider;
pub use complex_emotion::ComplexEmotionProvider;
pub use event_estimator::EventEstimator;
pub use event_ring::{EventEmitter, EventModule, EventModuleRegistrar};
pub use function_calling_parser::FunctionCallingParserPort;
pub use llm::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
pub use local_plugin_bridge::LocalPluginBridge;
pub use local_plugin_registry_port::LocalPluginRegistryPort;
pub use mcp_bridge::McpBridgePort;
pub use memory_backend_port::MemoryBackendPort;
pub use memory_retrieval::MemoryRetrieval;
pub use plugin_backend_registry::PluginBackendRegistryPort;
pub use plugin_host::PluginHostPort;
pub use policy::{EmotionPolicy, EventPolicy, MemoryPolicy};
pub use prompt_assembler::PromptAssembler;
pub use reply_post_processor::{
    PostProcessInput, PostProcessOutput, ReplyPostProcessor, ReplyPostProcessorEffectiveConfig,
    ReplyPostProcessorResolver,
};
pub use repository::{
    ComplexEmotionHintStore, FavorabilityRepository, MemoryRepository, MutablePersonalityStore,
    RelationIdentityStore, VirtualTimeStore,
};
pub use resource_coordination::{
    ResourceAdapterController, ResourceAdapterControllerOutcome, ResourceAdapterRegistrar,
    ResourceSnapshotSource,
};
pub use slot_backend_factory::SlotBackendFactoryPort;
pub use slot_resolver::SlotRegistryResolver;
pub use theater_director::{
    TheaterDirectorBackendKind, TheaterDirectorEffectiveConfig, TheaterDirectorPromptProvider,
    TheaterDirectorResolver, TheaterPromptBuildInput, TheaterPromptBuildOutput,
    MAX_THEATER_PROMPT_LEN, THEATER_BUILD_PROMPT_METHOD,
};
pub use user_emotion_analyzer::UserEmotionAnalyzer;
pub use user_llm_secrets::UserLlmSecretsPort;
