//! 插件宿主解析端口：`chat_engine` 经本 trait 依赖，不绑定 `PluginHost` 具体类型。

use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::models::{PluginBackends, PluginBackendsOverride, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

/// 按角色包 / 会话有效后端解析 `MemoryRetrieval` 等实现句柄。
pub trait PluginHostPort: Send + Sync {
    /// 按 `role.plugin_backends` 解析（无会话覆盖）。
    fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins;

    /// 按已合并的 effective 六槽 + 可选 `slot_registry` 与覆盖解析。
    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins;
}

impl PluginHostPort for PluginHost {
    fn resolve_for_role(&self, role: &Role) -> ResolvedRolePlugins {
        PluginHost::resolve_for_role(self, role)
    }

    fn resolve_for_effective_backends(
        &self,
        effective: &PluginBackends,
        slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
        backend_override: Option<&PluginBackendsOverride>,
    ) -> ResolvedRolePlugins {
        PluginHost::resolve_for_effective_backends(
            self,
            effective,
            slot_registry,
            backend_override,
        )
    }
}
