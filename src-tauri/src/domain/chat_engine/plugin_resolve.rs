//! 会话级插件解析：`chat_engine` 经 [`PluginHostPort`] 绑定实现，不引用 [`PluginHost`] 具体类型。

use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::ports::PluginHostPort;
use crate::models::{PluginBackends, Role};
use oclive_validation::SlotRegistryEntry;
use std::collections::BTreeMap;

/// 按角色包与会话有效 `slot_registry` 解析插件句柄（无会话时走包默认）。
#[must_use]
pub fn resolve_plugins_for_session(
    host: &dyn PluginHostPort,
    role: &Role,
    session_namespace: Option<&str>,
    effective_backends: &PluginBackends,
    slot_registry: Option<&BTreeMap<String, SlotRegistryEntry>>,
) -> ResolvedRolePlugins {
    if session_namespace.map(str::trim).filter(|s| !s.is_empty()).is_none() {
        return host.resolve_for_role(role);
    }
    host.resolve_for_effective_backends(effective_backends, slot_registry, None)
}
