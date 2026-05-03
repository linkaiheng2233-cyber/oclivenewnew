# 内核工程质量与生态路线图（DeepSeek 指令对齐）

> **定位**：在 **`oclive_kernel_runtime`** 上落实「工程质量 P0 / 内核能力 P1 / 开发者体验 P2」的分阶段清单。  
> **契约基准**：[`creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md`](../creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md)（Tauri 命令与 DTO）、[`creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md`](../creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md)、[`crates/oclive_kernel_runtime/src/models/dto.rs`](../crates/oclive_kernel_runtime/src/models/dto.rs)、[`handoff/10_ERROR_CODE_DICTIONARY.md`](./10_ERROR_CODE_DICTIONARY.md)。  
> **轻量维护**：[`handoff/LIGHTWEIGHT_FOLLOWUP_PLAN.md`](./LIGHTWEIGHT_FOLLOWUP_PLAN.md)。

---

## 仓库事实快照（避免重复造轮子）

| DeepSeek 项 | 当前仓库结论 |
|-------------|----------------|
| `reqwest::blocking` 全量替代 | **已完成**：workspace `reqwest` 无 `blocking`；runtime 使用 `Client` + async，同步边界见 **`blocking_http::block_on`**；详见 [`PERF_PHASES.md`](./PERF_PHASES.md)。 |
| `AppError` + 可诊断码 | **runtime 业务路径已收敛**：[`crates/oclive_kernel_runtime/src/error.rs`](../crates/oclive_kernel_runtime/src/error.rs) 提供 **`code()`** 与 **`to_frontend_error()`**（`[CODE]` 前缀）；`src/**/*.rs` 中 **`AppError::Unknown`** 已基本清除（保留枚举变体作兜底）；与字典 **`10_ERROR_CODE_DICTIONARY.md`** 持续对齐；其它 workspace crate 仍可按需收紧。 |
| 集成测试目录 | 大量逻辑在 **`#[cfg(test)]` 模块内**；**crate 级 `tests/*.rs`** 已含契约、会话烟测、**[`p0_support_modules_smoke.rs`](../crates/oclive_kernel_runtime/tests/p0_support_modules_smoke.rs)**（插件 semver / 市场缓存 / Expert 编译 / 本地导入）；仍可扩展 `expert_models_admin` 全链、`role_lifecycle` 删除等。 |
| crates.io / Docker | **未做**；见本文 P2 与验收门槛。 |

---

## 一、工程质量加固（P0）

### 1. API 对齐与清理

| 步骤 | 动作 |
|------|------|
| A1 | 以 **KERNEL_ENTRY_CHECKLIST** 为清单，对 `oclive_kernel_runtime::domain::*` 与 `api` 层做「命令 → 实现模块」交叉表，删除仅转发且无差异的中间层（每模块独立 PR）。 |
| A2 | 清理 **MATRIX「废弃 / 模糊地带」** 表中已解决项，保留仍开放的唯一入口说明。 |
| A3 | 禁止在 **`src-tauri/src/api`** 堆业务公式（与项目规则一致）；新增能力默认落在 **runtime**。 |

### 2. 补充测试体系

| 优先级 | 模块 | 建议测试锚点 |
|--------|------|----------------|
| P0 | **session / 状态** | `KernelAppState::new`、会话键、`process_message` 最小闭环（内存 DB + Mock LLM）。 |
| P0 | **plugin** | `PluginHost` 解析、`directory_plugins` bootstrap（已有部分单元测试，可抽 crate 级集成）。 |
| P0 | **expert-models** | `domain::expert_models_admin` 与 `role_runtime` JSON 读写边界。 |
| P0 | **role_lifecycle** | `domain::role_runtime_commands` / `role_info_snapshot` 与迁移表一致。 |
| P0 | **local_imports** | `domain::local_imports` 扫描与拒绝路径；crate 级烟测见 **`p0_support_modules_smoke`**。 |

**CI**：根工作区已 **`cargo test --workspace`**；新增 `crates/oclive_kernel_runtime/tests/*.rs` 会自动纳入，无需单独 job（除非要拆分 `--package` 加速）。

**已落地（首步）**：[`crates/oclive_kernel_runtime/tests/public_api_error_contract.rs`](../crates/oclive_kernel_runtime/tests/public_api_error_contract.rs) — `AppError` 与 **`10_ERROR_CODE_DICTIONARY`** Common 码、`[CODE]` 前缀契约。

**已落地（第二步）**：

- [`tests/session_process_message_smoke.rs`](../crates/oclive_kernel_runtime/tests/session_process_message_smoke.rs) — `process_message` + `shimeng` + Mock LLM 最小闭环。  
- [`tests/p0_support_modules_smoke.rs`](../crates/oclive_kernel_runtime/tests/p0_support_modules_smoke.rs) — **P0.T 子集**：`plugin_install` 依赖 semver、**`feature = market-sync`** 下索引磁盘缓存读写契约、`expert_models` 单基座编译、`local_imports` 扫描（无网络）。  
- 错误收紧（P0.E）：`plugin_archive`、`directory_plugin_commands`、`plugin_package_verify`、`llm_cancelable`；续：`plugin_install`、`plugin_index_sync`、`role_market_index_sync`、`plugin_reviews_index_sync`、`role_pack_archive`、`mcp_client`、`role_lifecycle` — **`oclive_kernel_runtime` 业务 `src` 不再使用 `AppError::Unknown`**（变体与契约测试除外）。  
- [`creator-docs/kernel/KERNEL_SDK.md`](../creator-docs/kernel/KERNEL_SDK.md) + [`scripts/run_kernel_server.sh`](../scripts/run_kernel_server.sh) / [`.ps1`](../scripts/run_kernel_server.ps1)（P2 体验子集）。

### 3. 错误处理系统化

| 步骤 | 动作 |
|------|------|
| E1 | 将 **`AppError::Unknown`** 在热路径上替换为 **`InvalidParameter` / `TransactionError { code, message }`** 等可分类变体，并同步 **错误码字典**。 |
| E2 | 审计 **`anyhow::Result`** 在 runtime 边界的使用：对外 API 应 **`Result<T, AppError>`**。 |
| E3 | 保证 **`to_frontend_error()`** 与前端 `tauri-api.ts` 映射一致（见字典 §Common）。 |

---

## 二、内核能力深度打磨（P1）

### 1. 异步化改造收尾

- 在 **`spawn_blocking` / `blocking_http`** 之外，扫描 **`std::fs`** 在 async 任务中的直调；长磁盘路径已部分在 `http_api` 使用 `spawn_blocking`，其余按文件列清单迁移。
- **禁止** 在 Tokio worker 内对同一 runtime **`Handle::block_on`**（见 `PERF_PHASES.md`）。

### 2. 启动性能优化

- 对 **`KernelAppState::new`** 做分段计时（日志或 `tracing`）；将非首屏路径（市场索引、MCP 扫描等）改为 **首次调用时初始化**（需行为评审，避免改变首次错误时机）。
- 嵌入式：**`default-features = false`** + 按需 feature（见 **`LIGHTWEIGHT_PROFILE.md`**）。

### 3. 内存占用分析

- Linux：`heaptrack` / `valgrind --tool=massif` 在固定脚本场景下跑；Windows：以 **任务管理器 + 长期 soak** 为基线，记录于 `handoff/` 新文件或 `PERF_PHASES` 附录。

---

## 三、开发者体验与生态准备（P2）

### 1. 官方 SDK 文档

- **首版已建**：[`creator-docs/kernel/KERNEL_SDK.md`](../creator-docs/kernel/KERNEL_SDK.md)（库模式、`process_message`、`kernel_server`、错误与互链）。  
- **待扩**：rustdoc 全量、`cargo doc --no-deps` 进 CI、更多集成示例。

### 2. crates.io 发布

- 补齐根 **`Cargo.toml` / crate `Cargo.toml`** 的 `repository`、`documentation`、`license` 与 **README 链接**；评估 **`oclive_core` / `oclive_validation`** 是否同步发布或保持 path。
- 发布前 **`cargo publish --dry-run -p oclive_kernel_runtime`**。

### 3. kernel_server 集成体验

- **脚本**：[`scripts/run_kernel_server.sh`](../scripts/run_kernel_server.sh)、[`scripts/run_kernel_server.ps1`](../scripts/run_kernel_server.ps1)（仓库根执行；可选端口）。  
- **待做**：最小 **Dockerfile**、Compose、与 **`OOCP_SPEC`** 的部署专页。

---

## 四、验收标准（分阶段）

| 阶段 | 门槛 |
|------|------|
| **P0 里程碑** | `cargo test --workspace`；runtime `tests/` 覆盖核心契约（错误码、关键状态）；MATRIX/CHECKLIST 与实现无矛盾。 |
| **P1 里程碑** | 启动分段数据写入 handoff；异步边界无新增 `block_on` 违规；可选内存基线一页。 |
| **P2 里程碑** | `KERNEL_SDK.md` + `cargo doc` 无报错；`cargo publish --dry-run`；kernel_server 一键脚本或 Docker 文档化。 |

---

## 建议执行顺序（给 Cursor / 子 Agent）

1. **P0.E** 错误码与 `Unknown` 收敛（小步、可测）。  
2. **P0.T** 继续扩展 **`crates/oclive_kernel_runtime/tests/`**（`expert_models_admin` / `role_lifecycle::delete_role` / 归档安装等重路径）。  
3. **P0.A** API 清单与死代码删除（按 CHECKLIST 分行 PR）。  
4. **P1** 启动与阻塞 I/O 清单。  
5. **P2** 文档与发布干跑。

本文档随里程碑更新「仓库事实快照」表，避免与代码漂移。
