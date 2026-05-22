//! 蓝图 v2 `slot_registry` 多实例解析端口。
//!
//! 宿主侧 `SlotResolver` 为无状态 struct；实现本 trait 以接入编排防腐层。

use oclive_kernel_types::SlotRegistryEntry;
use std::collections::BTreeMap;

/// 按 `slot_registry` 解析多实例插件句柄（实现方提供 registry 与返回的 slots 视图类型）。
///
/// ## When to implement
///
/// - **谁**：**宿主运行时**（Tauri `SlotResolver` 已实现）；一般插件作者**不**实现本 trait。
/// - **何时**：新宿主（嵌入式 / 无头）需要把蓝图 registry 绑到 `BackendRegistry` 时。
///
/// ## When not to implement
///
/// - 编写目录插件或 Remote 服务时：应实现 `MemoryRetrieval` / `LlmClient` 等**槽位能力** trait，而非本解析器。
pub trait SlotRegistryResolver: Send + Sync {
    /// 宿主 `BackendRegistry`（或等价注册表）。
    type Registry;
    /// 解析结果（宿主一般为 `ResolvedRoleSlots`）。
    type ResolvedSlots: Clone + Send + Sync + 'static;

    /// 按 `slot_registry` 条目解析多实例插件句柄。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn resolve(
        &self,
        registry: &Self::Registry,
        slot_registry: &BTreeMap<String, SlotRegistryEntry>,
    ) -> Self::ResolvedSlots;
}
