# Kernel V2：极薄内核与默认实现分层

> 状态：**阶段 5 主体已落地**（记忆 / 情绪 / 复杂情感 / Agent ReAct 默认实现已迁至设施 crate；事件与 Prompt 仍留 runtime，后续单独设计）  
> Baseline v1 契约与测试须保持向后兼容；物理拆分以独立提交推进。

## 1. 目标

- **极薄核心（`oclive_kernel_core`）**：协议与「宪法」级抽象——共享错误、核心 DTO/模型、数据访问 **trait**、后续逐步迁入的调度与 OOCP 路由等（按依赖顺序）。
- **默认实现（`oclive_kernel_runtime`）**：builtin 引擎、SQLite、`Ollama`/云端 LLM、远程/目录插件、市场同步、角色包 ZIP 等；通过 **`PluginBackends`** 的 `builtin` / `remote` / `directory` 与 **Cargo feature** 与核心衔接。
- **兼容**：`oclive_kernel_runtime` 的 **`full`** 继续代表官方「全功能」体验，等价于当前 V1 发行版能力组合。

## 2. 特性与「极薄」模式（规划）

| 组合 | 含义 |
|------|------|
| `oclive_kernel_runtime` **`full`**（默认） | 与现网 V1 一致：HTTP/OOCP、ZIP、市场、Agent 等。 |
| `default-features = false` + 按需子特性 | 裁剪默认实现；长期目标为**仅核心调度 + 必选协议**的极简运行时（阶段 3–4 用独立 `default-*-providers` feature 细化）。 |

## 3. 阶段与仓库映射（摘要）

| 阶段 | 内容 |
|------|------|
| 1 | 本文档 + 边界说明（ ongoing ） |
| 2 | 新建 `oclive_kernel_core`，迁入 `AppError`、`Memory` / `MemoryContext`、`repository` traits；`runtime` 再导出 |
| 3 | 以 **LLM** 为试点：`default-llm-providers`（默认开），关则仅保留 trait/远程或目录后端 |
| 4 | Memory / Emotion / Event / Prompt 等 **default-\*-providers** |
| 5 | `default-*-providers` 细化：设施 crate（memory / emotion / complex_emotion / **agent ReAct**）+ directory 示例；`kernel-agent` 与 **`default-agent-providers`** 分离（MCP 基础 vs 进程内 Builtin ReAct）；极薄 `cargo check -p oclive_kernel_runtime --no-default-features` |

## 4. 设施 crate 与默认提供者（阶段 5 快照）

| 设施 crate | `default-*-providers` | runtime 中保留的薄层 / 基础能力 |
|------------|----------------------|----------------------------------|
| `oclive_memory_builtin` | `default-memory-providers` | `classic` 再导出；directory 示例 `memory.rank` |
| `oclive_emotion_builtin` | `default-emotion-providers` | `EmotionAnalyzer` / `EmotionResultExt`；`emotion.analyze` 示例 |
| `oclive_complex_emotion_builtin` | `default-complex-emotion-providers` | `affect_metrics_from_seven_dim`；`complex_emotion.resolve_turn` 示例 |
| `oclive_agent_builtin` | `default-agent-providers` | **`McpShellAgent`** 仍在 runtime；ReAct 在设施 crate；`agent.process` 示例（契约演示） |

**Agent**：`kernel-agent` 控制 MCP 栈与轻量 **`McpShellAgent`**；**`default-agent-providers`** 单独控制是否链接 **`BuiltinReActAgent`**（`oclive_agent_builtin/providers`）。

## 5. 已实现（阶段 2 首包）

- Crate：`crates/oclive_kernel_core`
- 已迁入：`AppError` / `Result`、`models::memory`、`repository`（`MemoryRepository`、`FavorabilityRepository`、`ExpertModelsRepository`）
- `oclive_kernel_runtime` 对上述路径 **再导出**；移除 `tauri_invoke`（`AppError` 外置后不再在 runtime 内提供 `From<AppError> for InvokeError`，桌面显式 `to_frontend_error()`）。

## 6. 参考

- [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md)
- [KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)
- [LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)
