//! 插件宿主解析端口：`chat_engine` 经本 trait 依赖，不绑定具体 `PluginHost` 类型。

use oclive_kernel_types::{PluginBackends, PluginBackendsOverride, Role, SlotRegistryEntry};
use std::collections::BTreeMap;

/// 按角色包 / 会话有效后端解析各模块实现句柄。
pub trait PluginHostPort: Send + Sync {
    /// 单次解析结果（宿主侧一般为 `ResolvedRolePlugins`）。
    type Resolved: Clone + Send + Sync + 'static;

    /// 按 `role.plugin_backends` 解析（无会话覆盖）。
    fn resolve_for_role(&self, role: &Role) -> Self::Resolved;

    /// 按已合并的 effective 槽 + 可选 `slot_registry` 与覆盖解析。
    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> Self::Resolved;
}
