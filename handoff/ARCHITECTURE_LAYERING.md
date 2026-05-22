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

## 深化加固（2026-05 第二批）

| 项 | 状态 |
|----|------|
| Remote HTTP 统一 `RemoteHttpClientBlocking` / `RemoteHttpClientAsync` | 已落实 |
| `domain/error_helpers` 错误映射辅助 | 已落实 |
| `PluginHostPort` + `AppState::plugin_host_port` | 已落实 |
| CLI 废弃别名移除（见 `crates/oclive-cli/DEPRECATED_COMMANDS.md`） | 已落实 |
| `cargo udeps` 全 workspace | 需 **nightly**（本机 stable 未跑通）；见 `CONTRIBUTING.md` |
| 前端 `depcheck` | 已移除 `idb-keyval`、`monaco-editor`、`vite-plugin-monaco-editor` |

## 深化加固（2026-05 第三批）

| 项 | 状态 |
|----|------|
| `map_copresent_err!` / `map_plugin_err!` / `map_frontend_err!` 批量替换 | 已落实（`co_present` / `plugin_host` 无手写 `map_err`） |
| 遗留 manifest 专用 Tauri 写盘 API | 已移除 `RoleStorage::save_role_manifest`；C1 `set_session_plugin_backend` 仅委托 `set_session_slot_override` |
| `cargo udeps`（nightly，2026-05-22） | **无未使用依赖**；见 `CONTRIBUTING.md` |

## 内核 crate 拆分（2026-05-20）

| 项 | 状态 |
|----|------|
| `oclive_kernel_types`（DTO / `AppError` / 纯结构） | 已落实 |
| `oclive_kernel_contracts`（`MemoryRepository`、`MemoryRetrieval` 等 trait） | 已落实 |
| `oclive_kernel_runtime` 编排实现 + 过渡期 re-export | 已落实；详见 [KERNEL_CRATE_SPLIT_PLAN.md](KERNEL_CRATE_SPLIT_PLAN.md) |

## 最终收尾（2026-05-20）

| 项 | 状态 |
|----|------|
| v1→v2 创作者迁移指南 | [V1_TO_V2_MIGRATION.md](../creator-docs/role-pack/V1_TO_V2_MIGRATION.md) |
| `chat_engine` 经 `dyn PluginHostPort`（`plugin_resolve`） | 已落实；`co_present` / `process_message` 不引用 `PluginHost` 具体类型 |
| C1 薄包装 | 已落实；会话覆盖仅 `slot_registry` 路径 |
| `oclive test --oocp` | 已落实 |
| `oclive explain` 与 `ERROR_CODES.md` | 已补全 `AppError` 静态变体 |
| CLI 废弃蓝图模板文档 | 已删除；生成 `BLUEPRINT_V2_POINTER.md` |
| `oclive doctor` v2 蓝图三项检查 | 已落实 |
| 性能基线文档（v2 / matrix 说明） | 已更新 PERFORMANCE / LIGHTWEIGHT_PROFILE |

### 验证（2026-05-22，第三批）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 通过 |
| `cargo test --workspace --lib` / `-p oclive-cli` / `-p oclivenewnew-tauri --lib` | 通过 |
| `cargo test --workspace`（含 `tests/` 集成） | Windows 本机可能 `rlib format` 链接异常；以 CI Ubuntu 为准 |
| `npm run test:unit` / `npm run build`（oclivenewnew） | 通过 |

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
