# 内核分层（domain / infrastructure / api）

**状态**：P0–P8 收口后的工程纪律说明（2026-05-20）。

## 目标依赖方向

| 层 | 允许依赖 | 禁止依赖 |
|----|----------|----------|
| `domain/` | `domain/`、`models/`、`error/`、**`domain/ports/`** | `api/` |
| `infrastructure/` | `domain/`、`infrastructure/`、`models/` | `api/` |
| `api/` | `domain/`、`infrastructure/`、`state/` | — |

## 已落实

- **`domain/ports/llm.rs`**：`LlmClient` trait；编排与策略通过 `domain::ports::LlmClient` 引用，实现留在 `infrastructure/llm.rs`。
- **`CoPresentSlotRunner`**：`co_present` 仅经 trait 调用槽位合并，不直接耦合 `process_message` 其它子模块实现。
- **`module_relations`**：禁止写入 `pipeline.ocblueprint`（`oclive_validation`）；架构图边由前端 `buildBlueprintEdges(slot_registry)` **只读派生**，无 Rust/磁盘直写路径。
- **`api/`**：无 `domain` → `api` 引用。

## 已知适配层（后续可拆）

以下 **`domain` 仍引用 `infrastructure` 具体类型**（插件宿主、Remote HTTP、目录子进程、高风险授权等），属于 **防腐层未完全抽出** 的技术债；新代码应优先扩展 `domain` 内已有 trait（`MemoryRetrieval`、`PluginHost` 解析接口等），避免新增 `domain → infrastructure` 依赖：

- `domain/plugin_host.rs`、`domain/role_manager.rs`、`domain/agent.rs`、`domain/role_manifest_validate.rs`

拆法建议（非本迭代范围）：将 `PluginHost::resolve_*` 的 **工厂** 迁至 `infrastructure/plugin_wiring.rs`，`domain` 只保留 trait 与 DTO。

## `unsafe` 审查（任务 8）

全仓 `rg '\bunsafe\b' --type rust`：**无** `unsafe` 块；工作区 `[workspace.lints] unsafe_code = "forbid"` 与 CI clippy 一致。

## 审阅命令

```bash
# domain 不得引用 api
rg "use crate::api" src-tauri/src/domain

# infrastructure 不得引用 api
rg "use crate::api" src-tauri/src/infrastructure
```
