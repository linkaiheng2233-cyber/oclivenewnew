//! 本地插件发现桥接 trait。

use oclive_kernel_types::LocalPluginProviderDescriptor;

/// Provider 发现桥接接口（后续可由 WASM / Native Process 两种实现提供）。
pub trait LocalPluginBridge: Send + Sync {
    /// 返回桥接实现名称（用于诊断与日志）。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn bridge_name(&self) -> &'static str;

    /// 扫描并返回可用的本地插件 Provider 描述符列表。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`；发现失败时实现应返回空列表或跳过无效项。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn discover_providers(&self) -> Vec<LocalPluginProviderDescriptor>;
}
