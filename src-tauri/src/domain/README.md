# `src-tauri/src/domain` 依赖规则

编排与业务策略层。新代码应遵守下列方向（与 [ARCHITECTURE_LAYERING.md](../../../handoff/ARCHITECTURE_LAYERING.md) 一致）。

## 允许 / 禁止

| 模块 | 可依赖 | 不可依赖 |
|------|--------|----------|
| `domain/` | `domain/`、`models/`、`error/`、`domain/ports/`、`oclive_kernel_*` | `api/` |
| `infrastructure/` | `domain/`、`infrastructure/`、`models/` | `api/` |
| `api/` | `domain/`、`infrastructure/`、`state/` | — |

## 已知适配层（逐步收口）

以下文件仍直接 `use crate::infrastructure::…`（构造 `PluginHost`、Remote HTTP、目录子进程等）：

- `plugin_host.rs`、`role_manager.rs`、`agent.rs`、`slot_resolver.rs`、`role_manifest_validate.rs`

**原因**：实现尚未全部迁入 `infrastructure/*_wiring` + `ports` 工厂。**新功能**请优先扩展 `domain/ports` trait，勿新增 `domain → api` 引用。

## 编排入口

| 文件 | 职责 |
|------|------|
| `chat_engine/process_message.rs` | 单条消息总调度（健康检查、双核门控、异地分支） |
| `chat_engine/turn_pipeline.rs` | 共景同屏回合执行 |
| `dual_pipeline.rs` | 双核实验核 + 稳定核降级 |
| `slot_runner.rs` | 多实例槽位合并与调用 |
| `plugin_host.rs` | `plugin_backends` → `Arc<dyn …>` |
