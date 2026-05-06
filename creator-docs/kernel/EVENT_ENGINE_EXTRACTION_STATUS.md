# 事件引擎剥离状态（阶段备忘）

## 当前架构（与 `LIGHTWEIGHT_PROFILE.md` 一致）

- **`EventEstimator`** trait 位于 **`oclive_kernel_core`**（与 `PromptAssembler` 等门面同级）。
- **`default-event-providers` 开启**：链接 **`oclive_event_builtin`**（`BuiltinEventEstimator*`、`EventImpactEngine`），实现上 **委托** **`oclive_kernel_runtime`** 的 **`event_impact_ai`**、**`event_detector`** 与知识增强等编排；算法主体仍在 runtime，**非**「整算法已迁入 `oclive_event_builtin`」。
- **关闭 `default-event-providers`**：builtin 槽为 `Ignore` 桩，与 **`MODULE_NONE_SEMANTICS.md`** 一致。

## 目录插件替代

示例 **`examples/oclive-event-builtin-directory/`**（`event.estimate`，需 **`process:spawn`**）可在角色包中将 **`plugin_backends.event = directory`** 并指向该插件 id，以 **directory** 形态替代进程内 builtin 事件估计。

## 后续里程碑（未在本表承诺排期）

将 **`event_impact_ai` / `event_detector`** 等核心算法 **迁入 `oclive_event_builtin`**、runtime 仅保留薄编排，属于独立大 PR（需同步 bench / 契约测试与文档）。当前阶段以 **trait 在 core、builtin 在设施 crate、算法在 runtime、directory 示例可替代** 为验收基线。
