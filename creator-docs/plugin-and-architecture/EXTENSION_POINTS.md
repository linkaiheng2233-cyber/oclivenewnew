# 扩展点索引（宿主 �?可替换模块）

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)

�?[PLUGIN_V1.md](PLUGIN_V1.md) 一致：**v1 为编译期枚举**，经 `settings.json` �?`plugin_backends` 选择实现；记�?/ 情绪 / 事件 / Prompt 默认均为 **builtin**�?*`llm` 默认�?`ollama`**。五模块另可�?**`directory`**（`plugins/*/manifest.json` 子进程，�?[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）�?

**操作指南（如何替换）**：[HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)�?*HTTP 侧车协议**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)�?

## 宿主聚合

- **`PluginHost`**：持有各后端一�?`Arc<dyn Trait>`，按枚举分发；[`crates/oclive_kernel_runtime/src/domain/plugin_host.rs`](../../crates/oclive_kernel_runtime/src/domain/plugin_host.rs)�?*Remote** 槽位在设�?`OCLIVE_REMOTE_*` 时为 HTTP 客户�?[`src-tauri/src/infrastructure/remote_plugin/`](../../crates/oclive_kernel_runtime/src/infrastructure/remote_plugin/)�?*Directory** 槽位�?[`DirectoryPluginRuntime::ensure_rpc_url`](../../crates/oclive_kernel_runtime/src/infrastructure/directory_plugins/runtime.rs) 懒启动子进程后，复用同一�?HTTP 客户端与 URL�?
- **`ResolvedRolePlugins`**：`PluginHost::resolve_for_role(role)` 一次解�?**memory / emotion / event / prompt / llm** 五条线，**单次 `send_message` / `RoleManager` 回合内复�?*，避免重复匹配枚举�?

## Rust trait 与源文件

| 能力 | Trait / 类型 | 默认实现 | 源文�?|
|------|----------------|----------|--------|
| 记忆排序 / 上下�?| `MemoryRetrieval` | `BuiltinMemoryRetrieval`、`BuiltinMemoryRetrievalV2` | `crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs` |
| 用户句情�?| `UserEmotionAnalyzer` | `BuiltinUserEmotionAnalyzer`、`BuiltinUserEmotionAnalyzerV2` | `crates/oclive_kernel_runtime/src/domain/user_emotion_analyzer.rs` |
| 事件影响估计 | `EventEstimator` | `BuiltinEventEstimator`、`BuiltinEventEstimatorV2` | `crates/oclive_kernel_runtime/src/domain/event_estimator.rs` |
| Prompt 组装 | `PromptAssembler` | `BuiltinPromptAssembler`、`BuiltinPromptAssemblerV2`（**`oclive_prompt_builtin`**） | `crates/oclive_kernel_runtime/src/domain/prompt_assembler.rs`（槽位与 Remote 占位）；算法见 **`crates/oclive_prompt_builtin`** |
| LLM 调用 | `LlmClient`（`plugin_backends.llm`：`ollama` / `remote` / `directory`�?| 进程注入�?`OllamaClient`；`remote` 在配�?`OCLIVE_REMOTE_LLM_URL` 时走 HTTP JSON-RPC�?*`directory`** 使用 **`directory_plugins.llm`** 指向的插�?URL（见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）；否则回退进程内默�?LLM | `src-tauri/src/infrastructure/llm.rs`、`infrastructure/remote_plugin/` |
| 长期记忆持久�?| `MemoryRepository` | SQLite | `crates/oclive_kernel_runtime/src/domain/repository.rs`、`infrastructure/repositories` |
| 策略（情�?/ 事件 / 记忆�?| `EmotionPolicy` �?| `Default*` | `crates/oclive_kernel_runtime/src/domain/policy.rs`、`state` 加载 |

**世界观知�?*（`roles/{id}/knowledge/*.md`、manifest 可�?`knowledge` 块）�?**角色包资�?+ Prompt / 规则层补�?*�?*�?*通过 `plugin_backends` 切换；见 [../role-pack/WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)�?

## 运行时选择

- **`AppState::resolved_plugins_for(role)`**：一次解析记�?/ 情绪 / 事件 / Prompt / **LLM** 五条线；**`chat_engine` 主路径优先使�?*，见 [`src-tauri/src/state/mod.rs`](../../crates/oclive_kernel_runtime/src/state/mod.rs)�?
- **`memory_retrieval_for` / `user_emotion_analyzer_for` �?*：仅取单类后端时可用；内部按**完整** `role.plugin_backends` 解析（含 **`directory`** 与各�?id），�?`resolved_plugins_for` 不叠加调用�?
- **`RoleManager`**：持�?[`ResolvedRolePlugins`](../../crates/oclive_kernel_runtime/src/domain/plugin_host.rs)，`process_input` 与主对话同一套情绪与 Prompt 门面；[`with_memory_retrieval`](../../crates/oclive_kernel_runtime/src/domain/role_manager.rs) 可覆盖记忆后端做测试�?

## 前端

- 回复展示派生：[src/utils/replyPresentation.ts](../../src/utils/replyPresentation.ts)（与 `SendMessageResponse` 对齐）。`get_role_info` / `load_role` 返回�?**`plugin_backends`** 与角色包 `settings.json` 一致，便于 UI 展示当前模块化配置�?

## 外接（路线图�?

- 侧车进程 / JSON-RPC 草案：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)�? 
- 目录式插件（扫描、整壳、invoke）：[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)�?
