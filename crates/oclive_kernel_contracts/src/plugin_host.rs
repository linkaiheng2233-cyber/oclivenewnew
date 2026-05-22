//! 插件宿主解析端口：`chat_engine` 经本 trait 依赖，不绑定具体 `PluginHost` 类型。

use oclive_kernel_types::{PluginBackends, PluginBackendsOverride, Role, SlotRegistryEntry};
use std::collections::BTreeMap;

/// 按角色包 / 会话有效后端解析各模块实现句柄。
///
/// ## When to implement
///
/// - **谁**：Tauri 桌面宿主（`PluginHost`）、无头 `oclive_kernel_server` 等需要把 `Role` 变成 `ResolvedRolePlugins` 的运行时。
/// - **何时**：编排层（`process_message` / `co_present`）需要按角色或会话解析插件句柄时。
///
/// ## When not to implement
///
/// - 单元测试可对 `ResolvedRolePlugins` 手工组装，无需 mock 整个宿主。
/// - 纯数据结构校验（`oclive_validation`）不依赖本 trait。
///
/// # Examples
///
/// ```no_run
/// use oclive_kernel_contracts::PluginHostPort;
/// use oclive_kernel_types::Role;
///
/// fn resolve_example(host: &impl PluginHostPort, role: &Role) {
///     let resolved = host.resolve_for_role(role);
///     let _ = resolved;
/// }
/// ```
pub trait PluginHostPort: Send + Sync {
    /// 单次解析结果（宿主侧一般为 `ResolvedRolePlugins`）。
    type Resolved: Clone + Send + Sync + 'static;

    /// 按 `role.plugin_backends` 解析（无会话覆盖）。
    ///
    /// # Errors
    ///
    /// 当角色包后端配置无效、目录插件加载失败或内部 I/O 失败时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic；实现应返回 `Result` 或在内层捕获错误。
    fn resolve_for_role(&self, role: &Role) -> Self::Resolved;

    /// 按已合并的 effective 槽 + 可选 `slot_registry` 与覆盖解析。
    ///
    /// # Errors
    ///
    /// 当 `effective` / `slot_registry` / `backend_override` 组合不合法或插件句柄不可解析时返回 `Err`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> Self::Resolved;
}
