# 日后如何替换模块（可替换框架速查）

本文说明 **宿主已拆成哪些块**、**换一块时要动哪里**。契约细节仍以 [PLUGIN_V1.md](PLUGIN_V1.md) 为准。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)
**创作者总览（环境变量、HTTP 方法、联调、热更新边界）**：[CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)
**HTTP JSON-RPC 完整协议与示例**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)

---

## 一、分成了哪些「可替换模块」

| 模块 | 职责 | Rust trait | 当前蓝图 `slot_registry` 实例 | 默认实现 |
|------|------|------------|-------------------------------|----------|
| **记忆检索** | 长期记忆排序、上下文、关键词搜索 | `MemoryRetrieval` | `type: memory`；`builtin` / `remote` / `directory` / `local` / `none` | `BuiltinMemoryRetrieval`；`directory` 以实例 `plugin` 指向 `manifest.id` |
| **用户句情绪** | 从文本得到七维情绪 | `UserEmotionAnalyzer` | `type: emotion`；`builtin` / `remote` / `directory` / `none` | `BuiltinUserEmotionAnalyzer` |
| **事件影响** | LLM 估计事件类型与影响因子 | `EventEstimator` | `type: event`；`builtin` / `remote` / `directory` / `none` | `BuiltinEventEstimator` |
| **Prompt 组装** | 主对话 system/user 字符串 | `PromptAssembler` | `type: prompt`；`builtin` / `remote` / `directory` / `none` | `BuiltinPromptAssembler`；共景健康路径不可缺有效 Prompt |
| **LLM 推理** | 调用大模型生成 | `LlmClient` | `type: llm`；`ollama` / `remote` / `directory` / `none` | `ollama` 为进程注入客户端；`remote` 用 `OCLIVE_REMOTE_LLM_URL`；`directory` 用实例 `plugin` |
| **Agent 编排** | 工具调度 / ReAct 等 | `AgentProvider` | `type: agent`；`builtin` / `remote` / `directory` / `none` | `BuiltinReActAgent`；当前执行折叠后的单一 Agent，`plugins[]` 工具并集尚未实现（`K-AGENT-MERGE-01`）；MCP 清单在 **`{app_data}/mcp-servers`** |
| **长期记忆存储** | 读写 SQLite 中的记忆行 | `MemoryRepository` | *不属于六槽；换库需改基础设施* | `SqliteMemoryRepository` |
| **策略（情感/事件/记忆条）** | 是否写入、重要性等 | `EmotionPolicy` 等 | `config/policy.toml` 场景 profile | `Default*` |

**聚合入口**：会话先得到有效 `slot_registry`，再将六种稳定类型折叠成 `PluginBackends` 兼容视图；[`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) 绑定具体实现，`SlotRunner` 保留实例语义。对话内的 **`ResolvedRolePlugins`** 一次取齐六条子系统线。`AppState.llm` 仍是进程级默认句柄，与有效 LLM 后端 `ollama` 指向同一实现。

---

## 二、替换「内置」实现（编译期，推荐先做）

1. **实现 trait**
   在 `kernel/crates/oclive_kernel_host/src/domain/` 下新增 `your_memory_retrieval.rs`（示例），实现 `MemoryRetrieval`（或其它对应 trait）。

2. **注册到 `PluginHost`**
   在 [`plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) 里：
   - 增加字段，如 `memory_foo: Arc<dyn MemoryRetrieval>`；
   - 在 `new()` 里 `Arc::new(YourMemoryRetrieval)`；
   - 在 `memory_retrieval()` 的 `match` 中增加新枚举分支。

3. **扩展枚举与校验**
   在 [`models/plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs) 的对应 enum 中增加变体，并同步 `oclive_validation` 的 `allowed_backends_for_type` 与解析/降级测试。**Serde 使用 `snake_case`**，与蓝图 wire 一致。

4. **角色包**
   在 `pipeline.ocblueprint.slot_registry` 的目标实例中写 `"type": "memory", "backend": "your_variant"`（名称与枚举、校验器一致）。这会扩展公开后端枚举，须按 Breaking 流程评估兼容性。

5. **校验与文档**
   更新 [PLUGIN_V1.md](PLUGIN_V1.md) 表格；必要时加单元测试。

---

## 三、替换 Remote（HTTP 侧车，已接入宿主）

- 设置 **`OCLIVE_REMOTE_PLUGIN_URL`**：记忆 / 情绪 / 事件 / Prompt 在角色包中选 `remote` 时走该端点（JSON-RPC 方法名见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）。
- 设置 **`OCLIVE_REMOTE_LLM_URL`**：`llm` 选 `remote` 时主对话与标签任务走该端点。
- 未设置 URL 时行为与此前一致：回退 builtin 或进程内 LLM，并记一次警告。
- 侧车实现可用任意语言，只要遵守同一 JSON-RPC 形状；无需改 `chat_engine` 主流程。

---

## 三 b、Directory（`distros/chat-pro/plugins/` 目录插件，与 Remote 同协议）

- 在角色包 `slot_registry` 中把目标实例设为 **`backend: directory`**，并用 `plugin`（单个）或允许合并时的 `plugins[]` 填 **`manifest.json` 的 `id`**。
- 宿主按 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) 的扫描根发现插件、按 manifest 启动子进程、从 stdout 读取 JSON-RPC **base URL**，之后与 Remote 一样走 HTTP。
- 整壳 UI、`directory_plugin_invoke`、开发者模式与最小示例：**[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)**。

---

## 四、一般不通过六槽后端切换的部分

- **`LlmClient` 进程级实现**：换网关/云 API 可在 [`infrastructure/llm.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/llm.rs) 增加新实现并在 `AppState::new` 里注入；或通过 **`OCLIVE_REMOTE_LLM_URL`** 使用远程 JSON-RPC（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）。
- **`MemoryRepository`**：换向量库等属存储层，宜单独抽象或新 repository 实现，再考虑是否与 manifest 绑定。

---

## 五、相关文件索引

| 用途 | 路径 |
|------|------|
| 宿主聚合 | `kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs` |
| Remote HTTP 客户端 | `kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/` |
| 目录插件扫描 / 子进程 / RPC URL | `kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/` |
| 运行时解析 | `AppState::resolved_plugins_for` — `kernel/crates/oclive_kernel_host/src/state/mod.rs` |
| 对话主链 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs` 等 |
| 测试用演示 | `RoleManager::with_memory_retrieval` — `kernel/crates/oclive_kernel_host/src/domain/role_manager.rs` |
