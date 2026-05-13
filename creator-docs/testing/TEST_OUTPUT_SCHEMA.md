# 统一测试结果输出（JSON）— Schema v1

> **目的**：让目录测试插件、CI 协议套件、以及未来社区贡献的「测试侧车」能用**同一套字段约定**输出结果，便于宿主 / 编写器 / 市场面板做通用渲染。  
> **参考实现**：官方 Vue 插件 `plugins/official-vue-test-runner` 的 `run_test` → `structured` 字段；OOCP 协议套件 `examples/oocp-test-suite/run.mjs --json` 使用 **`kind: oclive.protocol_conformance_report.v1`** 子集。

## 顶层

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schemaVersion` | `number` | ✅ | 固定 **1**（本版）。 |
| `kind` | `string` | ✅ | **`oclive.unit_test_run.v1`**：单元/组件测试运行器（Vitest/Jest/…）。**`oclive.protocol_conformance_report.v1`**：协议/集成步骤报告（无 `suites[]` 或 `suites` 为空数组亦可）。 |
| `summary` | `object` | ✅ | 见下表。 |
| `suites` | `array` | ❌ | 套件级统计；单测聚合工具可置 `[]`。 |
| `suiteTotals` | `object` | ❌ | 可选；顶层套件计数（Vitest JSON 的 `num*TestSuites`）。 |
| `failures` | `array` | ✅ | 失败列表；无失败时为 `[]`。 |
| `meta` | `object` | ❌ | 运行器特有元数据（headline、cwd、runner 名等）。 |

## `summary`

| 字段 | 类型 | 说明 |
|------|------|------|
| `passed` | `number` | 通过用例数。 |
| `failed` | `number` | 失败用例数。 |
| `skipped` | `number` | 跳过 / pending。 |
| `total` | `number` | 参与统计的用例总数。 |
| `passRate` | `number \| null` | `0..1`；`total===0` 时为 `null`。 |
| `durationMs` | `number` | 端到端耗时（毫秒）。 |
| `exitCode` | `number` | 子进程退出码或宿主约定。 |
| `ok` | `boolean` | 是否视为整体成功（含「无测试但退出码 0」等策略）。 |

## `suites[]`（`kind = oclive.unit_test_run.v1`）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 稳定 id（常用文件路径）。 |
| `name` | `string` | 展示名。 |
| `passed` / `failed` / `skipped` | `number` | 该文件/套件内断言统计。 |
| `durationMs` | `number \| null` | 可选；来自报告时间戳差。 |

## `failures[]`

| 字段 | 类型 | 说明 |
|------|------|------|
| `file` | `string` | 源文件路径（或 Vitest 套件名回退）。 |
| `line` / `column` | `number \| null` | 定位；未知为 `null`。 |
| `message` | `string` | 首条人类可读错误摘要。 |
| `expected` / `actual` | `string \| null` | 断言 diff 可选；无则为 `null`。 |
| `suiteTitle` / `testTitle` / `fullName` | `string \| null` | 辅助 UI。 |
| `messages` | `string[]` | 原始失败消息列表（可截断）。 |

## 与实现对齐

| 组件 | 对齐说明 |
|------|-----------|
| **official-vue-test-runner** | `run_test` 返回根级 `structured`，其形状与本页 **`oclive.unit_test_run.v1`** 一致。 |
| **oocp-test-suite** | `node run.mjs --json` 末尾打印 **`oclive.protocol_conformance_report.v1`**：`summary` + 空 `suites`/`failures` + `meta.scenarios`。 |

## 演进

- **MINOR**：仅追加可选字段或 `meta` 子键。  
- **MAJOR**：重命名 / 删除字段 → `schemaVersion` 递增至 2。
