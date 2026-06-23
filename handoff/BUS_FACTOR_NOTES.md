# Bus Factor：关键路径交接笔记

**读者**：有经验的 Rust / Vue 工程师。  
**目标**：在 **约半天** 内能定位任意主路径模块的 **入口文件**、**核心类型/函数**，并理解 **为何这样设计**（意图级，非逐行教程）。

约定：路径以仓库根 **`oclivenewnew`** 为准；`src-tauri` 为桌面宿主内核与 Tauri 命令。

---

## 0. 内核 crate 拆分（2026-05 后路径）

| Crate | 路径 | 职责 |
|-------|------|------|
| **`oclive_kernel_types`** | `kernel/crates/oclive_kernel_types/` | DTO、`AppError`、纯数据结构 |
| **`oclive_kernel_contracts`** | `kernel/crates/oclive_kernel_contracts/` | `LlmClient`、`PluginHostPort`、`EventEstimator`、`AgentProvider` 等 trait |
| **`oclive_kernel_runtime`** | `kernel/crates/oclive_kernel_runtime/` | 编排实现；宿主经 `src-tauri` re-export 消费 |
| **`oclive_validation`** | `kernel/crates/oclive_validation/` | manifest / **`pipeline.ocblueprint` v2** 校验 |
| **`oclive-cli`** | `kernel/crates/oclive-cli/` | 脚手架、`bench` / `test` / `doctor` / `ci init` |

**`distros/desktop-tauri/domain/ports/`** 仅 re-export trait + 本地 `impl`；编排应依赖 **`dyn`** 端口，见 [`ARCHITECTURE_LAYERING.md`](./ARCHITECTURE_LAYERING.md)。

---

## 0.5 角色包 vs 蓝图（配置分责）

| 层 | 磁盘 | 谁改 | 关键路径 |
|----|------|------|----------|
| **角色包** | `meta` 创作者子集、`prompts/`、`scenes/`、`core_personality.txt` | 初级创作者 | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) §0 |
| **蓝图** | 同文件 `pipeline.ocblueprint` 内 **`slot_registry`**、**`groups`**、（目标）**`runtime_config`** | 管理员 / `oclive plugin manage` / `save_role_slot_registry` | [SETTINGS_REFERENCE.md](../creator-docs/cli/SETTINGS_REFERENCE.md) §零 |
| **边界 SSOT** | — | — | **[ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md)** |

**今日实现**：引擎字段（如 `meta.interaction_mode`）仍在 `meta`，与边界文档「目标迁至 `runtime_config`」并存；改边界时同步 `oclive_validation` 与 `RoleStorage::load_role_from_dir`。

**双核**：`dual_core.enabled` 仅蓝图（[DUAL_CORE_CURSOR_HANDOFF.md](./DUAL_CORE_CURSOR_HANDOFF.md)）；非创作者字段。

**高危能力**：不在 `settings.json` / 蓝图 — 见目录插件 **`permissions`** + **`high_risk_grants.json`**（§2 与 PLUGIN_V1）。

---

## 1. 内核编排：`process_message`

### 入口与主语义

| 项目 | 说明 |
|------|------|
| **对外入口** | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/mod.rs` 再导出 `process_message`；实现主体在 **`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`**。 |
| **HTTP / Tauri** | 与 OOCP / `invoke` 对齐的请求体见 `oclive_kernel_runtime` DTO（宿主经 `kernel/crates/oclive_kernel_types/src/models/mod.rs` 再导出）。 |
| **主语义（概念六段）** | 文件头注释：**分析情绪 → 检测事件 → 演化性格 → 构建 Prompt → 调用 LLM → 持久化**；实际执行会根据 **Agent 短路**、**异地 / 远程人生** 分支到 `process_remote_stub` / `process_remote_life`，否则进入 **`co_present::process_co_present`**。 |
| **阶段标注** | `ProcessMessageError` / `pm!` 宏带 `stage` 字符串（如 `ensure_role_loaded`、`startup_health`），日志检索用 `target: "oclive_chat"`。 |

### 从用户输入到 LLM 返回（追踪顺序）

1. **API 层**：Tauri `generate_handler` 注册的命令或 `http_api.rs` 路由 → 调用 `domain::process_message`（同一代码路径意图）。
2. **`process_message::run`**：校验场景、`ensure_role_runtime`、加载 `Role`、`effective_plugin_backends_for_session`、`startup_health::ensure_once`。
3. **Agent 分支**：若 `pl.agent.process` 返回 `handled`，则走短路径组装 `SendMessageResponse` 并返回。
4. **异地**：`user_is_remote_from_character` + `remote_life_enabled` → `process_remote_stub` 或 `process_remote_life`。
5. **共景主路径**：**`co_present::process_co_present`**（见下一节关联）。

**设计意图**：单入口便于审计与测试；分支显式化避免「隐式 pipeline DSL」与运行时不一致（历史上去除 `pipeline.ocblueprint` 主路径的原因，见 `AGENTS.md` 内核架构小节）。

---

## 2. 插件调度：`PluginHost` 与 `plugin_backends`

| 项目 | 说明 |
|------|------|
| **装配与解析** | **`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`**：`PluginHost::resolve_for_role` 按角色包 + 会话覆盖解析 **第 1–6 模块**（见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)），得到 `ResolvedRolePlugins`（各槽 `Arc<dyn …Provider>`）。 |
| **配置来源** | v2：**`pipeline.ocblueprint` → `slot_registry`**（折叠六槽）+ DB 会话覆盖；legacy：`settings.json` → `plugin_backends`。有效值：`effective_plugin_backends_for_session`（`AppState`）。 |
| **模块编号与枚举** | **[`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)**、**[`SETTINGS_REFERENCE.md`](../creator-docs/cli/SETTINGS_REFERENCE.md)**、**[`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)**。 |
| **降级策略** | 目录插件 / Remote 失败时主对话路径尽量 **记日志 + 回退内置或 Ollama**（具体分支见 `co_present`、remote 子模块与插件运行时；错误码见 ERROR_CODES）。 |
| **目录插件运行时** | `kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/`（manifest 校验、`runtime` 等）；与 **`high_risk_grants`**、**`mcp_client`** 联动见 [`A4_CLOSURE_SUMMARY.md`](./A4_CLOSURE_SUMMARY.md)。 |

**设计意图**：编译期内置实现 + 运行时解析目录/Remote，使同一编排代码不随插件数量重新编译（Monolith 模式另见 §7）。

---

## 3. 错误码体系

| 环节 | 文件/类型 | 说明 |
|------|-----------|------|
| **核心错误枚举** | `oclive_kernel_runtime` 的 **`AppError`**（宿主 `distros/desktop-tauri/src/error.rs` 再导出） | 业务语义与 `thiserror` 变体。 |
| **→ 前端 / HTTP JSON** | **`AppError::to_kernel_json()`** → **`KernelErrorBody`** 单行 JSON | Tauri `invoke` 失败与 HTTP `error` 对象同源；见 **`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`**。 |
| **宿主桥** | `distros/desktop-tauri/src/error.rs` 的 **`to_invoke_error`** | 将内核错误转为 `InvokeError`。 |
| **目录插件 ApiError** | `distros/desktop-tauri/src/api/error.rs` | 与内核 `code` 对齐。 |
| **前端 i18n** | `distros/shared/distros/shared/src/i18n/locales/fragments/apiErrors.*` + `toFriendlyErrorMessage` 等 | 机器码 → 用户可读文案；新增码需 **中英** 词条 + [`ERROR_CODES.md`](../creator-docs/getting-started/ERROR_CODES.md) 速查表。 |

**设计意图**：机器可读 `code` 稳定，文案可迭代；HTTP 与桌面一致，便于 OOCP 黑盒断言。

---

## 4. 数据库与迁移

| 项目 | 说明 |
|------|------|
| **迁移位置** | **`kernel/crates/oclive_kernel_host/migrations/*.sql`**，按序号递增；**勿虚构表名**（以迁移文件为准）。 |
| **`role_runtime`** | 在 **`001_init.sql`** 创建；后续迁移追加列（如 `relation_state`、`virtual_time_ms`、`interaction_mode` 等）。 per-role 会话与立绘情绪等核心运行时态多在此表。 |
| **`app_settings`** | **`011_app_settings.sql`**：`key` / `value` 文本键值；应用级（非角色包）如 `interaction_mode`、`remote_fallback_to_builtin`。 |
| **新迁移步骤** | 新增 `0NN_*.sql` → 若需 ORM/仓库层映射，改 **`distros/desktop-tauri/src/infrastructure`** 与 **`domain/repository`** trait → 跑 `cargo test` 与相关集成测 → 文档若暴露给用户则更新 ERROR_CODES / FAQ。 |

**设计意图**：SQLite 单文件、显式迁移版本链，避免「隐式自动建表」与生产数据分叉。

---

## 5. 抽象情感与 `narrative_hint`

| 项目 | 说明 |
|------|------|
| **Trait / 类型** | **`kernel/crates/oclive_kernel_runtime/src/domain/complex_emotion.rs`** 再导出内核 `ComplexEmotionInput` / `ComplexEmotionOutput` 等；内置 **`BuiltinKeywordComplexEmotionProvider`**。 |
| **Remote 可选** | **`kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/complex_emotion_http.rs`**。 |
| **注入 Prompt 链路** | **[`turn_pipeline/`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/)**：在 **`load_recent_context` 之后、`build_prompt` 之前** 调用解析；上一轮 hint 缓存在 **`AppState`**（按会话 `srid`）；经 **`PromptInput::previous_complex_emotion_narrative_hint`** 传入 **`PromptBuilder::build_prompt`**（`prompt_builder/mod.rs`）。 |
| **测试** | **`distros/desktop-tauri/tests/narrative_hint_prompt_roundtrip.rs`**。 |

**设计意图**：复杂情感是「回合间状态」，不能只在 UI 层拼接；必须进入 Prompt 构造输入才能保证模型侧一致。

---

## 6. 高耦合编译模式（Monolith）

| 项目 | 说明 |
|------|------|
| **权威 RFC** | **`creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md`**。 |
| **脚手架** | **`kernel/crates/oclive-cli/`**：`init --monolith` 生成 **`monolith.toml`**、**`vendor/oclive_monolith_builtin/`**、**`process_message_monolith.rs`**、双 **`[[bin]]`**。 |
| **符号对接** | 焊接桩模板唯一源在 **vendor** 路径；`build` / `bench` 子命令生成与对比性能（见 **`creator-docs/cli/OCLIVE_CLI_GUIDE.md`**）。 |

**设计意图**：可选把热路径后端编译进单二进制，减少 IPC/动态加载；默认宿主仍走「解析 + 插件」路径以降低迭代成本。

---

## 7. 角色包系统

| 项目 | 说明 |
|------|------|
| **磁盘格式权威** | **`creator-docs/role-pack/ROLE_PACK_SPEC.md`** ↔ 加载逻辑 **`RoleStorage::load_role`**（`src-tauri` infrastructure 层搜索 `load_role`）。 |
| **manifest 与 README** | 根目录 **`distros/chat-pro/roles/README_MANIFEST.md`**。 |
| **校验 crate** | **`kernel/crates/oclive_validation`**：与宿主、编写器共享；顶层键等见 crate 内 `json_keys` / manifest 校验模块。 |
| **编写器 wasm** | **oclive-pack-editor**：`npm run wasm:build` 指向相邻克隆的 **`oclivenewnew/kernel/crates/oclive_validation`**，详见该仓 **README / CONTRIBUTING**；未构建 wasm 时 TypeScript 子集回退。 |

**设计意图**：「单真相」在 Rust 校验；UI 侧 wasm 减少与宿主行为漂移。

---

## 8. 测试体系与 CI

| 层级 | 位置 | 说明 |
|------|------|------|
| **OOCP HTTP 黑盒** | **`examples/oocp-test-suite/`**（`node run.mjs`）；文档 **`creator-docs/testing/OOCP_TEST_SUITE.md`**。 |
| **CI job `oocp-test-suite`** | **`.github/workflows/ci.yml`** | 构建 `oclivenewnew-tauri`、`--api` 拉起、健康检查后跑 OOCP + **`scripts/e2e-core-api-restart.mjs`**。 |
| **`invoke` 热路径** | **`distros/desktop-tauri/tests/invoke_hotpath_matrix.rs`**；矩阵 **`handoff/INVOKE_HOTPATH_MATRIX.md`**。 |
| **Vue 组件 / E2E** | **oclive-pack-editor** 仓（本仓不重复维护全量 UI 树）；本仓 **`npm run test:unit`** + **Ubuntu `frontend` job** 上 **Playwright + vite preview**（见 CONTRIBUTING）。 |
| **Rust 全量** | **`rust` job**：fmt、clippy `-D warnings`、`cargo test`（`src-tauri` 目录）。 |
| **`cargo-audit`** | **`continue-on-error: true`** | 可见性优先；失败不挡合入但应跟踪 **`KNOWN_VULNERABILITIES.md`**。 |

**失败处理策略**：先读 job 日志区分 **fmt/clippy/unit/OOCP/frontend**；OOCP 失败常与环境变量、mock LLM、端口占用有关；Windows 不跑 Playwright 时勿忽略 Ubuntu `frontend` 红。

---

## 9. 核心模块导航（按路径）

| 模块路径 | 核心文件 | 关键概念 | 修改时注意 |
|----------|----------|----------|------------|
| 主编排 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` | 单消息入口、Agent/异地分支 | 不改业务顺序请先读 [`DESIGN_DECISIONS.md`](../creator-docs/architecture/DESIGN_DECISIONS.md) |
| 共景 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs` | 回合阶段、`narrative_hint` | 槽位调用走 `SlotRunner`，勿直连 `pl.llm` |
| 多实例合并 | `kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs` | last-wins / memory 去重 | 新策略需补「为何」注释；Agent 合并在 `plugin_host` |
| 插件装配 | `kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs` | `ResolvedRolePlugins`、`PluginHostPort` | Remote 需 env；目录插件权限见 `high_risk_grants` |
| 蓝图解析 | `kernel/crates/oclive_kernel_host/src/domain/slot_resolver.rs` | `slot_registry` → `ResolvedRoleSlots` | 不手写 `module_relations` |
| 蓝图加载 | `kernel/crates/oclive_kernel_host/src/infrastructure/storage.rs` | `load_blueprint_v2_for_role_dir` | 校验失败看 `oclive_validation` 报错拼接 |
| 端口 trait | `kernel/crates/oclive_kernel_contracts/src/` | `LlmClient`、`MemoryRetrieval`… | 插件作者实现 trait，见各文件 **When to implement** |
| 纯类型 | `kernel/crates/oclive_kernel_types/` | DTO、`AppError` | 无 I/O；契约变更同步 validation |
| 蓝图校验 | `kernel/crates/oclive_validation/` | v2 schema、`slot_registry` | 改 JSON 形状必跑 `pack validate` + 单测 |
| 前端架构图 | `distros/shared/src/composables/useArchitectureGraphModel.ts` | `buildBlueprintEdges`、`groups` | 边只读派生，勿写回 blueprint |

---

## 10. 建议的「第一次读代码」顺序（半天内）

1. `DOCUMENTATION_INDEX.md` → `KERNEL_AND_MODULES_ARCHITECTURE.md`（总图）  
2. `process_message.rs` 全文 skim + `turn_pipeline.rs` 前 120 行  
3. `plugin_host.rs` 的 `resolve_for_role` 签名与返回类型  
4. `error.rs` + `KERNEL_ERROR_CODE_CONVENTION.md` 一页  
5. `migrations/001_init.sql` + 最新序号迁移扫一眼  
6. `OOCP_TEST_SUITE.md` 打开 CI 对齐表  

若仍卡：**在 `tracing` 日志里用 `stage` / `role_id` / `srid` 搜**，再回到上表对应文件。

---

## 11. 维护说明

- 本文 **不替代** PLUGIN_V1 / ERROR_CODES / ROLE_PACK_SPEC；契约以那些文档 + 代码为准。  
- 大重构后若入口搬迁：请更新本节路径并提 PR 链到 **`DOCUMENTATION_INDEX`**「工程纪律」小节。
