# 扩展点索引（宿主 ↔ 可替换模块）

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)

与 [PLUGIN_V1.md](PLUGIN_V1.md) 一致：**v1 为编译期枚举**，经 `settings.json` → `plugin_backends` 选择实现；记忆 / 情绪 / 事件 / Prompt 默认均为 **builtin**，**`llm` 默认为 `ollama`**。

**操作指南（如何替换）**：[HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)。**HTTP 侧车协议**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

## 宿主聚合

- **`PluginHost`**：持有各后端一套 `Arc<dyn Trait>`，按枚举分发；[`src-tauri/src/domain/plugin_host.rs`](../../src-tauri/src/domain/plugin_host.rs)。Remote 槽位在设置 `OCLIVE_REMOTE_*` 时为 HTTP 客户端 [`src-tauri/src/infrastructure/remote_plugin/`](../../src-tauri/src/infrastructure/remote_plugin/)。
- **`ResolvedRolePlugins`**：`PluginHost::resolve_for_role(role)` 一次解析 **memory / emotion / event / prompt / llm** 五条线，**单次 `send_message` / `RoleManager` 回合内复用**，避免重复匹配枚举。

## Rust trait 与源文件

| 能力 | Trait / 类型 | 默认实现 | 源文件 |
|------|----------------|----------|--------|
| 记忆排序 / 上下文 | `MemoryRetrieval` | `BuiltinMemoryRetrieval`、`BuiltinMemoryRetrievalV2` | `src-tauri/src/domain/memory_retrieval.rs` |
| 用户句情绪 | `UserEmotionAnalyzer` | `BuiltinUserEmotionAnalyzer`、`BuiltinUserEmotionAnalyzerV2` | `src-tauri/src/domain/user_emotion_analyzer.rs` |
| 事件影响估计 | `EventEstimator` | `BuiltinEventEstimator`、`BuiltinEventEstimatorV2` | `src-tauri/src/domain/event_estimator.rs` |
| Prompt 组装 | `PromptAssembler` | `BuiltinPromptAssembler`、`BuiltinPromptAssemblerV2` | `src-tauri/src/domain/prompt_assembler.rs` |
| LLM 调用 | `LlmClient`（`plugin_backends.llm`：`ollama` / `remote`） | 进程注入的 `OllamaClient`；`remote` 在配置 `OCLIVE_REMOTE_LLM_URL` 时走 HTTP JSON-RPC（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)），否则回退进程内默认 LLM | `src-tauri/src/infrastructure/llm.rs`、`infrastructure/remote_plugin/` |
| 长期记忆持久化 | `MemoryRepository` | SQLite | `src-tauri/src/domain/repository.rs`、`infrastructure/repositories` |
| 策略（情感 / 事件 / 记忆） | `EmotionPolicy` 等 | `Default*` | `src-tauri/src/domain/policy.rs`、`state` 加载 |

**世界观知识**（`roles/{id}/knowledge/*.md`、manifest 可选 `knowledge` 块）是 **角色包资源 + Prompt / 规则层补充**，**不**通过 `plugin_backends` 切换；见 [../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)。

## 运行时选择

- **`AppState::resolved_plugins_for(role)`**：一次解析记忆 / 情绪 / 事件 / Prompt / **LLM** 五条线；**`chat_engine` 主路径优先使用**，见 [`src-tauri/src/state/mod.rs`](../../src-tauri/src/state/mod.rs)。
- **`memory_retrieval_for` / `user_emotion_analyzer_for` 等**：仅取单类后端时可用；内部直接调 `PluginHost`，与 `resolved_plugins_for` 不叠加调用。
- **`RoleManager`**：持有 [`ResolvedRolePlugins`](../../src-tauri/src/domain/plugin_host.rs)，`process_input` 与主对话同一套情绪与 Prompt 门面；[`with_memory_retrieval`](../../src-tauri/src/domain/role_manager.rs) 可覆盖记忆后端做测试。

## 前端

- 回复展示派生：[src/utils/replyPresentation.ts](../../src/utils/replyPresentation.ts)（与 `SendMessageResponse` 对齐）。`get_role_info` / `load_role` 返回的 **`plugin_backends`** 与角色包 `settings.json` 一致，便于 UI 展示当前模块化配置。

## 外接（路线图）

- 侧车进程 / JSON-RPC 草案：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。
