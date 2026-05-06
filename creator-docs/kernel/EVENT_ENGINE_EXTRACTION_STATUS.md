# 事件引擎剥离状态

## 结论：**已完成**（算法在 `oclive_event_builtin`）

- **`EventEstimator`** trait：**`oclive_kernel_core`**。
- **`EventImpactEstimate` / `Event` / `EventType` 等 DTO**：**`oclive_kernel_models`**。
- **`EventDetector` / `event_impact_ai`（含 LLM `generate_tag`、JSON 解析、规则回退、人格轴辅助、json 截取）**：**`oclive_event_builtin`**。
- **`oclive_kernel_runtime`**：**`KernelEventImpactEngine`** 实现 **`EventImpactEngine`**，调用 **`oclive_event_builtin::estimate_event_impact`**；`domain::event_detector` / `domain::event_impact_ai` 为 **re-export**（与历史 `use` 路径兼容）。

## 目录插件替代

示例 **`examples/oclive-event-builtin-directory/`**（`event.estimate`，需 **`process:spawn`**）可在角色包设 **`plugin_backends.event = directory`**，以进程外实现替代内置 **`EventEstimator`**。

## 参考

- 阶段总述与依赖表：**[`KERNEL_V2_DESIGN.md`](./KERNEL_V2_DESIGN.md)**（§6、阶段 7 行）。
- 特性与 SKU：**[`LIGHTWEIGHT_PROFILE.md`](./LIGHTWEIGHT_PROFILE.md)**。
