# 扩展点索引（宿主 ↔ 可替换模块）

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)

与 [PLUGIN_V1.md](PLUGIN_V1.md) 一致：**v1 为编译期枚举**，经 `settings.json` → `plugin_backends` 选择实现；记忆 / 情绪 / 事件 / Prompt / **Agent** 默认均为 **builtin**，**`llm` 默认为 `ollama`**。上述六类另可选 **`remote`** / **`directory`**（`plugins/*/manifest.json` 子进程，见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）。

**操作指南（如何替换）**：[HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)。**HTTP 侧车协议**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

## 宿主聚合

- **`PluginHost`**：持有各后端一套 `Arc<dyn Trait>`，按枚举分发；[`crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)。**Remote** 槽位在设置 `OCLIVE_REMOTE_*` 时为 HTTP 客户端 [`src-tauri/src/infrastructure/remote_plugin/`](../../src-tauri/src/infrastructure/remote_plugin/)。**Directory** 槽位在 [`DirectoryPluginRuntime::ensure_rpc_url`](../../src-tauri/src/infrastructure/directory_plugins/runtime.rs) 懒启动子进程后，复用同一套 HTTP 客户端与 URL。
- **`ResolvedRolePlugins`**：`PluginHost::resolve_for_role(role)` 一次解析 **memory / emotion / event / prompt / llm / agent** 六条子系统线，**单次 `send_message` / `RoleManager` 回合内复用**，避免重复匹配枚举。

## Rust trait 与源文件

| 能力 | Trait / 类型 | 默认实现 | 源文件 |
|------|----------------|----------|--------|
| 记忆排序 / 上下文 | `MemoryRetrieval` | `BuiltinMemoryRetrieval`、`BuiltinMemoryRetrievalV2` | `crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs` |
| 用户句情绪 | `UserEmotionAnalyzer` | `BuiltinUserEmotionAnalyzer`、`BuiltinUserEmotionAnalyzerV2` | `crates/oclive_kernel_runtime/src/domain/user_emotion_analyzer.rs` |
| 事件影响估计 | `EventEstimator` | `BuiltinEventEstimator`、`BuiltinEventEstimatorV2` | `crates/oclive_kernel_host/src/domain/event_estimator.rs` |
| Prompt 组装 | `PromptAssembler` | `BuiltinPromptAssembler`、`BuiltinPromptAssemblerV2` | `crates/oclive_kernel_runtime/src/domain/prompt_assembler.rs` |
| LLM 调用 | `LlmClient`（`plugin_backends.llm`：`ollama` / `remote` / `directory`） | 进程注入的 `OllamaClient`；`remote` 在配置 `OCLIVE_REMOTE_LLM_URL` 时走 HTTP JSON-RPC；**`directory`** 使用 **`directory_plugins.llm`** 指向的插件 URL（见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）；否则回退进程内默认 LLM | `src-tauri/src/infrastructure/llm.rs`、`infrastructure/remote_plugin/` |
| Agent 编排 | `AgentProvider`（`plugin_backends.agent`：`builtin` / `remote` / `directory`） | `BuiltinReActAgent`；`directory` 需 `directory_plugins.agent`；MCP 配置根见 [`PluginHost::new`](../../crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) 的 `app_data_dir` | `crates/oclive_kernel_host/src/domain/agent.rs`、`infrastructure/mcp_client.rs` |
| 长期记忆持久化 | `MemoryRepository` | SQLite | `crates/oclive_kernel_host/src/domain/repository.rs`、`infrastructure/repositories` |
| 策略（情感 / 事件 / 记忆） | `EmotionPolicy` 等（trait：`crates/oclive_kernel_contracts/src/policy.rs`） | `Default*`（`crates/oclive_kernel_runtime/src/domain/policy.rs`） | wiring：`crates/oclive_kernel_host/src/infrastructure/policy_registry.rs` |

**世界观知识**（`roles/{id}/knowledge/*.md`、manifest 可选 `knowledge` 块）是 **角色包资源 + Prompt / 规则层补充**，**不**通过 `plugin_backends` 切换；见 [../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)。

## 运行时选择

- **`AppState::resolved_plugins_for(role)`**：一次解析记忆 / 情绪 / 事件 / Prompt / **LLM** / **Agent** 六条子系统线；**`chat_engine` 主路径优先使用**，见 [`src-tauri/src/state/mod.rs`](../../src-tauri/src/state/mod.rs)。
- **`memory_retrieval_for` / `user_emotion_analyzer_for` 等**：仅取单类后端时可用；内部按**完整** `role.plugin_backends` 解析（含 **`directory`** 与各槽 id），与 `resolved_plugins_for` 不叠加调用。
- **`RoleManager`**：持有 [`ResolvedRolePlugins`](../../crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)，`process_input` 与主对话同一套情绪与 Prompt 门面；[`with_memory_retrieval`](../../crates/oclive_kernel_host/src/domain/role_manager.rs) 可覆盖记忆后端做测试。

## 前端

- 回复展示派生：[src/utils/replyPresentation.ts](../../src/utils/replyPresentation.ts)（与 `SendMessageResponse` 对齐）。`get_role_info` / `load_role` 返回的 **`plugin_backends`** 与角色包 `settings.json` 一致，便于 UI 展示当前模块化配置。

## 外接（路线图）

- 侧车进程 / JSON-RPC 草案：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。  
- 目录式插件（扫描、整壳、invoke）：[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)。

## 契约扩展信封（V-CONTRACT Phase 0）

**原则**：内核**解释面最小、运载面无限**——内核只理解核心字段；插件专有状态经信封携带，不无限堆叠 `PromptInput` hint 字段。

| 类型 | 位置 | 用途 |
|------|------|------|
| `SlotExtension { schema_id, data }` | `oclive_kernel_types::slot_extension` | 槽插件输出的 opaque JSON 信封；`schema_id` 标识 payload 语义 |
| `EmotionResult.extension` | `emotion.rs` | 七维情绪之外的异构投射（如 CHS 三维）；`#[serde(default)]`，省略时无扩展 |
| `ComplexEmotionOutput.extension` | `complex_emotion.rs` | 复杂情感侧车可附带私有字段 |
| `PromptInput.extra_sections` | `prompt.rs` | 宿主编排的通用 Prompt 段 `{ title, body }[]`；在回复质量锚点**之前**按序渲染为 `【title】\nbody` |

Phase 1–3（能力协商 `plugin.describe`、槽私有状态 `slot_state` 表、融合提供者出版级）见 `handoff/TECHNICAL_DEBT_INVENTORY.md` **V-CONTRACT** / **V-FUSED** 条目；**不升** `SCHEMA_VERSION`，六槽枚举与蓝图 `slot_registry` 不变。

## 契约演化规则

与 [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md) 一致；扩展点相关补充：

1. **只增不删（additive-only）**：DTO / JSON-RPC 响应新增字段须 `#[serde(default)]` 或协议层 optional；旧客户端/插件省略新字段时必须仍能解析。
2. **枚举演进**：对外可见、可能扩展的 Rust 枚举优先 `#[non_exhaustive]`；match 侧须 `_` 兜底或显式降级，禁止假设变体集合已闭合。
3. **解释 vs 运载**：内核编排只依赖**文档化的核心字段**；新 hint 类能力优先 `SlotExtension` 或 `extra_sections`，而非再增 `PromptInput` 顶层字段（已有字段保持兼容，不再鼓励堆叠）。
4. **破坏性变更**：删字段、改语义、升 `SendMessageResponse.schema` / `SCHEMA_VERSION`、改六槽键名 → 走 Breaking 流程（兼容层、`oclive_validation`、契约文档、CHANGELOG 中英双更）。
5. **远程协议**：新方法（如 Phase 1 `plugin.describe`）为**可选**；未实现 = 零能力，不得逼升级。
6. **持久化**：挂 `extension` 于已有 DB 序列化类型（如 `Memory`）前须单独评估 migration（Phase 2 `slot_state` 为首选私有状态通道）。
