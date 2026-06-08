//! Domain ports: **pure re-exports plus the host `impl` layer**.
//!
//! All trait definitions live in [`oclive_kernel_contracts`]; this directory contains no `trait` definitions.
//! New code should prefer `use oclive_kernel_contracts::…`.

pub mod conversation_persist;
pub mod llm;
pub mod plugin_host;
pub mod slot_resolver;
pub mod turn_persistence;
pub mod turn_policies;

pub use llm::LlmClient;
pub use oclive_kernel_contracts::{AgentProvider, EventEstimator, SlotRegistryResolver};
pub use plugin_host::PluginHostPort;
