# oclive_event_builtin

进程内 **`EventEstimator` 薄壳**（`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`）：实现 `oclive_kernel_core::EventEstimator`，将计算委托给 **`EventImpactEngine`**。

- **算法正文**（`estimate_event_impact`、规则回退等）仍在 **`oclive_kernel_runtime::domain::event_impact_ai`**，由运行时提供 `EventImpactEngine` 实现。
- 目录插件示例：`examples/oclive-event-builtin-directory/`（`event.estimate` JSON-RPC）。
