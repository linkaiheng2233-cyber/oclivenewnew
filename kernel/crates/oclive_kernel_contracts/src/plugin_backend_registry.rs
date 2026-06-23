//! Composed plugin backend registry port (factory + local + MCP).

pub use crate::agent_mcp_registry_port::AgentMcpRegistryPort;
pub use crate::local_plugin_registry_port::LocalPluginRegistryPort;
pub use crate::slot_backend_factory::SlotBackendFactoryPort;

/// Full backend registry surface for [`PluginHost`](crate::plugin_host::PluginHostPort).
pub trait PluginBackendRegistryPort:
    SlotBackendFactoryPort + LocalPluginRegistryPort + AgentMcpRegistryPort
{
}

impl<T> PluginBackendRegistryPort for T where
    T: SlotBackendFactoryPort + LocalPluginRegistryPort + AgentMcpRegistryPort
{
}
