//! 内置事件影响 **`EventEstimator` 薄壳**：`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`。
//!
//! 开启 **`providers`**（默认）：装配到 `oclive_kernel_runtime` 的 `default-event-providers`。
//! 算法委托 [`EventImpactEngine`](oclive_kernel_core::EventImpactEngine)，由 runtime 桥接到 `event_impact_ai`。

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinEventEstimator, BuiltinEventEstimatorV2};
