//! 内置事件：**`EventDetector` / `event_impact_ai`** 与 **`EventEstimator`** 薄壳。
//!
//! - **`EventImpactEngine`** 的默认实现由 [`event_impact_ai::estimate_event_impact`] 提供；`oclive_kernel_runtime` 仅保留 `KernelEventImpactEngine` 桥接。
//! - 开启 **`providers`**（默认）：装配 `BuiltinEventEstimator` / V2。

mod affect_axis;
pub mod event_detector;
pub mod event_impact_ai;
mod json_loose;

pub use event_detector::EventDetector;
pub use event_impact_ai::{estimate_event_impact, event_impact_ai_enabled, soften_impact_factor};

#[cfg(feature = "providers")]
mod providers;

#[cfg(feature = "providers")]
pub use providers::{BuiltinEventEstimator, BuiltinEventEstimatorV2};
