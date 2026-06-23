# 内核分层（domain / infrastructure / api）

**状态**：P0–P8 收口后的工程纪律说明（2026-06-09）。

**D-LAYER-05 ratchet（2026-06-11）**：`node scripts/check-domain-layering.mjs` — 生产 `crate::infrastructure::` FQ **1**；`#[cfg(test)]` `use crate::infrastructure` **3**；基线 [`LAYERING_BASELINE.json`](LAYERING_BASELINE.json)。明细见 [`kernel/crates/oclive_kernel_host/src/domain/README.md`](../kernel/crates/oclive_kernel_host/src/domain/README.md)。

## 关键架构决策（摘要）

完整说明见 [`creator-docs/architecture/DESIGN_DECISIONS.md`](../creator-docs/architecture/DESIGN_DECISIONS.md)。

| 决策 | 为什么这样做 |
|------|----------------|
| 蓝图从主路径移除 | 避免磁盘流程与 `process_message` 实际顺序不一致；顺序由代码审计 |
| 防腐层完整（`domain/ports` 零 trait） | 核心 trait 独立于 Tauri，任意宿主可实现 |
| `module_relations` 自动派生 | 禁止手写字段；从 `slot_registry` 推导边才可靠 |
| 分组（`groups`） | 创作者 UI 归类，不改变执行顺序 |
| 多实例合并策略 | memory 去重合并、llm last-wins、agent 在 PluginHost 合并工具集等 |
| C1 薄包装 | 旧 API 签名保留，内部委托 slot 覆盖，给下游过渡时间 |

---

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
| CLI 废弃别名移除（见 `kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md`） | 已落实 |
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
| `oclive_kernel_runtime` 编排实现 + 过渡期 re-export | 已落实；详见 [../kernel/crates/README.md](../kernel/crates/README.md) |

## 防腐层补全（2026-05-20）

| 项 | 状态 |
|----|------|
| `kernel_types` / `kernel_contracts` / `kernel_runtime` **pub 可见性审计** | 已落实（types 显式根导出；contracts `pub(crate)` 子模块；runtime `utils` 收紧 + `extract_json_object` 根导出） |
| `PluginHostPort` / `LlmClient` / `SlotRegistryResolver` 迁入 `oclive_kernel_contracts` | 已落实 |
| `EventEstimator` / `AgentProvider` 迁入 `oclive_kernel_contracts` | 已落实 |
| `distros/desktop-tauri/domain/ports/` **无 trait 定义**（仅 re-export + `impl`） | 已落实；`SlotResolver` struct 仍在 `domain/slot_resolver.rs`，经 `SlotRegistryResolver` 端口化 |
| 防腐层（`domain` → `ports` / `kernel_contracts`） | **完整**（2026-05-20） |

## 精修收尾（2026-05-20）

| 项 | 状态 |
|----|------|
| `oclive-cli` 子命令模块化（`commands/bench*`、`init*`、`lint*`） | 已落实 |
| Tauri 去除对 `oclive_kernel_contracts` 的直接依赖（经 `oclive_kernel_runtime` re-export） | 已落实 |
| `oclive_kernel_types` / `oclive_kernel_contracts` rustdoc（模块级 + 公开类型/trait 一句注释） | 已落实 |
| `init/mod.rs` ≤300 行、`bench/mod.rs` ≤300 行（`init_config` / `bench_core`） | 已落实 |

## 双核双态（2026-05）

| 项 | 状态 |
|----|------|
| P2 宿主 v3 加载 + `DualPipelineRunner` + `process_message` 门控 | **已完成** |
| P3 `oclive init --dual-core` | **已完成** |
| P4 OOCP S13（可选 `--include-s13`） | **已完成** |
| P5 Monolith `[dual_core]` 脚手架 | **已完成** |
| 深化：七槽 experimental method + 快照回滚 + Method 注册表 + 架构图 + 开发者指南 | **已完成** |
| 精修：`oclive_dual_core` 分级日志 + 性能结果解读文档 | **已完成** |

文档：[DEVELOPER_GUIDE.md](../creator-docs/dual-core/DEVELOPER_GUIDE.md) · [METHOD_REGISTRY.md](../creator-docs/dual-core/METHOD_REGISTRY.md) · [DUAL_CORE_ALIGNMENT.md](DUAL_CORE_ALIGNMENT.md)

## 最终精修（2026-05-22）

| 项 | 状态 |
|----|------|
| CI 一致性：`clippy` / `cargo test --workspace`（`CARGO_BUILD_JOBS=1`）/ `npm run test:unit` / `npm run build` | 见下表「最终精修验证」 |
| 集成测：`invoke_hotpath_matrix` / `narrative_hint_contract_audit` / `permission_three_way_consistency` / `oclive-cli` e2e | 见下表 |
| [TESTING_GUIDE.md](../creator-docs/testing/TESTING_GUIDE.md) 结果解读章节 | **已完成** |
| 双核调试日志（`DualPipelineRunner`） | **已完成** |

## 代码质量收尾（2026-05-22）

| 项 | 状态 |
|----|------|
| `oclive_kernel_types` 公开导出审计（`lib.rs` 约定 + `Role` 谓词文档） | **已完成** |
| `oclive_kernel_contracts` trait 职责一览 | **已完成** |
| 编排错误去重（`ProcessMessageError::dual_core_*`） | **已完成** |
| `domain/README.md` 依赖方向与已知适配层 | **已完成** |
| 双核注册表 `pub(crate)` 收紧 | **已完成** |
| 核心 / 双核 / `AppError` 注释增强 | **已完成** |

### 代码质量验证（2026-05-22）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings`（CI `rust` job SSOT） | ✅ |
| `cargo test -p oclivenewnew-tauri --lib` | ✅ 127 tests |
| `npm run test:unit` | ✅ 22 tests |

### 最终精修验证（2026-05-22）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings`（`CARGO_BUILD_JOBS=1`；勿把 `-j 1` 放在 `--` 后） | ✅ 通过 |
| `cargo test`（`distros/desktop-tauri/`，与 CI `rust` job 一致） | ✅ 通过（含集成测） |
| `cargo test --workspace --lib` | ✅ 通过 |
| `cargo test -p oclive_kernel_contracts --doc` | ✅ 通过（修正 `EventEstimator` 示例） |
| `npm run test:unit` / `npm run build` | ✅ 22 tests + vite build |
| `invoke_hotpath_matrix` / `narrative_hint_contract_audit` / `permission_three_way_consistency` | ✅ 通过 |
| `cargo test -p oclive-cli`（含 e2e_init / e2e_explain / e2e_dry_run 等） | ✅ 通过 |

## 最终扫尾（2026-05-20）

| 项 | 状态 |
|----|------|
| `EventEstimator` / `AgentProvider` 编排审计：`co_present` / `process_message` 无具体类型引用；热路径经 `Arc<dyn …>`（`slot_runner` / `plugin_host` / `AppState::*_for`） | 已落实 |
| `kernel_contracts` trait 方法审计 | 见 [KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md](KERNEL_CONTRACTS_TRAIT_METHOD_AUDIT.md) |
| `oclive doctor` 内核 trait 实现检查（`plugin_host_port_impl` 等 5 项） | 已落实 |
| `useless_format` 修复（`f2e44bf`） | 已在历史中；`lint_cmd` → `commands/lint.rs` |
| 依赖重复项 `cargo tree -d` | 见 [LIGHTWEIGHT_PROFILE.md](../creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6（不可统一项已记录） |
| 性能矩阵实测填充 `PERFORMANCE.md` | 延后（需本地 Monolith 工程 + 长时 `bench --matrix`） |
| 全量最终验证（任务 11） | 见下表「最终扫尾验证」 |

## 蓝图 groups（2026-05-20）

| 项 | 状态 |
|----|------|
| `pipeline.ocblueprint` `groups` Schema + `oclive_validation` 校验 | 已落实 |
| `RoleInfo.blueprint_groups_pack` / 架构图 `ArchGroupNode` 分组边框 | 已落实 |
| 活跃文档 v2 化（`CREATOR_WORKFLOW` / `ROLE_PACK_SPEC` 等） | 已落实 |

## Cursor 优化轮（2026-05-20）

| 项 | 状态 |
|----|------|
| `sqlx` 0.8+ + `cargo audit` 漏洞级清零 | 已落实 |
| `cargo deny check licenses` + `deny.toml` / `DISCLAIMER` §4 | 已落实 |
| `init` → `preset_config` / `project_config`（≤250 行/文件） | 已落实 |
| `bench` → `bench_runner`（`bench/mod.rs` ≤250 行） | 已落实 |
| `kernel_contracts` trait `# Errors` / `# Panics` + 核心 trait 示例 | 已落实 |
| `oclive ci init` 模板 `cargo-deny` + `loom` job | 已落实 |
| `oclive lint` 彩色输出 / 通过率 / 耗时 | 已落实 |
| `handoff/studio/USER_GUIDE.md` 创作 / 试聊 / 导出工作流 | 已落实 |

### 优化轮验证（任务 9）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 通过（2026-05-20） |
| `cargo test --workspace --lib` | ✅ 通过（280 tests） |
| `npm run test:unit` / `npm run build` | ✅ 通过（15 tests + vite build） |

## 测试前收尾（2026-05-20）

| 项 | 状态 |
|----|------|
| `PERFORMANCE.md` 矩阵 / 冷启动 / 长稳可复制命令（无 v1 对比） | 已落实 |
| [TESTING_GUIDE.md](../creator-docs/testing/TESTING_GUIDE.md)（三种测试） | 已落实 |
| `oclive ci init` 模板含 `cargo-audit` job | 已落实 |
| `oclive test --json` Schema + 报告结构 | 已落实 |
| `fuzz_blueprint_v2` + CI fuzz 冒烟 | 已落实 |
| `oclive lint --deny` + 根 `deny.toml` | 已落实 |
| `CONTRIBUTING` 模块负责人与 PR / CI 失败处理 | 已落实 |

### 测试前验证（任务 8）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 通过（2026-05-20） |
| `cargo test --workspace --lib` | ✅ 通过（122 tests） |
| `npm run test:unit` / `npm run build` | ✅ 通过（15 tests + vite build） |

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

### 验证（2026-05-22，第三批 + 精修）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 通过（精修后复验） |
| `cargo test --workspace --lib` / `-p oclive-cli` / `-p oclivenewnew-tauri --lib` | 通过 |
| `cargo test --workspace`（含 `tests/` 集成） | Windows 本机可能 `rlib format` 链接异常；以 CI Ubuntu 为准 |
| `cargo doc --no-deps -p oclive_kernel_types -p oclive_kernel_contracts` | 通过 |
| `npm run test:unit` / `npm run build`（oclivenewnew） | 通过（精修后复验） |

### 最终扫尾验证（2026-05-20，任务 11）

| 检查 | 结果 |
|------|------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 通过（2026-05-20） |
| `cargo test --workspace --lib` | ✅ 通过（2026-05-20） |
| `cargo test --workspace`（含 `tests/` 集成） | ⚠️ Windows 本机 `os error 1455`（页面文件不足 / rlib mmap）；以 CI Ubuntu 为准 |
| `npm run test:unit` / `npm run build` | ✅ 通过（15 tests + `vite build`，2026-05-20） |

## 已知适配层（后续可拆）

**SSOT**：`creator-docs/` 下无同名分层文档；计数与 FQ 清单以本文件 D-LAYER-05 段与 [`kernel/crates/oclive_kernel_host/src/domain/README.md`](../kernel/crates/oclive_kernel_host/src/domain/README.md) 为准。

生产路径剩余 **`domain → infrastructure` FQ 引用（1）**：

| 文件 | 引用 | 用途 |
|------|------|------|
| `user_llm_env.rs` | 1× `db_ports::DbSettingsPort` | 用户 LLM 环境读盘（Wave 1 port） |

**`use crate::infrastructure` 导入（3，全 `#[cfg(test)]`）**：`event_impact_ai.rs`、`complex_emotion_store.rs`、`mutable_profile_llm.rs`（`MockLlmClient` / `test_db`）。

`startup_health.rs` 经 **`DbHealthPort`**；`role_manager.rs` 经构造函数注入插件宿主（无生产 FQ）。

热路径持久化已经 **`domain/ports/`** turn ports（`ChatTurnPersistencePort` 等）注入；插件宿主、Remote/directory、reply post-processor、MCP 等工厂已迁至 `infrastructure/*_wiring`。

新代码应优先扩展 `domain/ports` 或 `oclive_kernel_contracts`，**不得净增**生产 FQ 或 `use` 计数（`node scripts/check-domain-layering.mjs`）。

## `unsafe` 审查（任务 8）

全仓 `rg '\bunsafe\b' --type rust`：**无** `unsafe` 块；工作区 `[workspace.lints] unsafe_code = "forbid"` 与 CI clippy 一致。

## 审阅命令

```bash
# domain 不得引用 api
rg "use crate::api" kernel/crates/oclive_kernel_host/src/domain

# infrastructure 不得引用 api
rg "use crate::api" distros/desktop-tauri/src/infrastructure
```
