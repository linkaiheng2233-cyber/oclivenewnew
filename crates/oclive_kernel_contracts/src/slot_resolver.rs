//! 蓝图 v2 `slot_registry` 多实例解析端口。
//!
//! 宿主侧 `SlotResolver` 为无状态 struct；实现本 trait 以接入编排防腐层。

use oclive_kernel_types::SlotRegistryEntry;
use std::collections::BTreeMap;

/// 按 `slot_registry` 解析多实例插件句柄（实现方提供 registry 与返回的 slots 视图类型）。
pub trait SlotRegistryResolver: Send + Sync {
    /// 宿主 `BackendRegistry`（或等价注册表）。
    type Registry;
    /// 解析结果（宿主一般为 `ResolvedRoleSlots`）。
    type ResolvedSlots: Clone + Send + Sync + 'static;

    fn resolve(
        &self,
        registry: &Self::Registry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Self::ResolvedSlots;
}
