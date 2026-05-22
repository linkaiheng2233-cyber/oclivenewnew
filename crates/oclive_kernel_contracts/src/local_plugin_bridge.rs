//! 本地插件发现桥接 trait。

use oclive_kernel_types::LocalPluginProviderDescriptor;

/// Provider 发现桥接接口（后续可由 WASM / Native Process 两种实现提供）。
pub trait LocalPluginBridge: Send + Sync {
    fn bridge_name(&self) -> &'static str;
    fn discover_providers(&self) -> Vec<LocalPluginProviderDescriptor>;
}
