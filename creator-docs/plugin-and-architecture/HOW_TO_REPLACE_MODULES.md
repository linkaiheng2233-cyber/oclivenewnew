# 日后如何替换模块（可替换框架速查）

本文说明 **宿主已拆成哪些块**、**换一块时要动哪里**。契约细节仍以 [PLUGIN_V1.md](PLUGIN_V1.md) 为准。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**创作者总览（环境变量、HTTP 方法、联调、热更新边界）**：[CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)  
**HTTP JSON-RPC 完整协议与示例**：[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)

---

## 一、分成了哪些「可替换模块」

| 模块 | 职责 | Rust trait | `settings.json` 字段（`plugin_backends` 下） | 默认实现 |
|------|------|------------|---------------------------------------------|----------|
| **记忆检索** | 长期记忆排序、上下文、关键词搜索 | `MemoryRetrieval` | `memory`: `builtin` / `builtin_v2` / `remote` / `directory` | `BuiltinMemoryRetrieval`、`BuiltinMemoryRetrievalV2`；**`directory`** 需 `directory_plugins.memory` 指向 `plugins/<id>/` |
| **用户句情绪** | 从文本得到七维情绪 | `UserEmotionAnalyzer` | `emotion`: `builtin` / `builtin_v2` / `remote` / `directory` | 同上；**`directory`** 需 `directory_plugins.emotion` |
| **事件影响** | LLM 估计事件类型与影响因子 | `EventEstimator` | `event`: `builtin` / `builtin_v2` / `remote` / `directory` | 同上；**`directory`** 需 `directory_plugins.event` |
| **Prompt 组装** | 主对话 system/user 字符串 | `PromptAssembler` | `prompt`: `builtin` / `builtin_v2` / `remote` / `directory` | 同上；**`directory`** 需 `directory_plugins.prompt` |
| **LLM 推理** | 调用大模型生成 | `LlmClient` | `llm`: `ollama` / `remote` / `directory` | `ollama`：进程注入的客户端；`remote`：`OCLIVE_REMOTE_LLM_URL` 的 JSON-RPC，未配置则委托默认 LLM；**`directory`** 需 `directory_plugins.llm`（子进程 URL，无需 `OCLIVE_REMOTE_LLM_URL`） |
| **长期记忆存储** | 读写 SQLite 中的记忆行 | `MemoryRepository` | *（未挂 plugin_backends，换库需改基础设施）* | `SqliteMemoryRepository` |
| **策略（情感/事件/记忆条）** | 是否写入、重要性等 | `EmotionPolicy` 等 | `config/policy.toml` 场景 profile | `Default*` |

**聚合入口**：[`PluginHost`](../../src-tauri/src/domain/plugin_host.rs) 按枚举挂具体实现；对话内用 **`ResolvedRolePlugins`**（`AppState::resolved_plugins_for`）一次取齐 **memory / emotion / event / prompt / llm** 五条线。`AppState.llm` 仍为进程级默认句柄（与 `plugin_backends.llm = ollama` 指向同一实现）。

---

## 二、替换「内置」实现（编译期，推荐先做）

1. **实现 trait**  
   在 `src-tauri/src/domain/` 下新增 `your_memory_retrieval.rs`（示例），实现 `MemoryRetrieval`（或其它对应 trait）。

2. **注册到 `PluginHost`**  
   在 [`plugin_host.rs`](../../src-tauri/src/domain/plugin_host.rs) 里：
   - 增加字段，如 `memory_foo: Arc<dyn MemoryRetrieval>`；
   - 在 `new()` 里 `Arc::new(YourMemoryRetrieval)`；
   - 在 `memory_retrieval()` 的 `match` 中增加新枚举分支。

3. **扩展枚举**  
   在 [`models/plugin_backends.rs`](../../src-tauri/src/models/plugin_backends.rs) 的 `MemoryBackend`（或对应 enum）中增加变体，**serde 用 `snake_case`**，与 JSON 一致。

4. **角色包**  
   在 `settings.json` 中写 `"plugin_backends": { "memory": "your_variant" }`（名称与枚举一致）。

5. **校验与文档**  
   更新 [PLUGIN_V1.md](PLUGIN_V1.md) 表格；必要时加单元测试。

---

## 三、替换 Remote（HTTP 侧车，已接入宿主）

- 设置 **`OCLIVE_REMOTE_PLUGIN_URL`**：记忆 / 情绪 / 事件 / Prompt 在角色包中选 `remote` 时走该端点（JSON-RPC 方法名见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）。
- 设置 **`OCLIVE_REMOTE_LLM_URL`**：`llm` 选 `remote` 时主对话与标签任务走该端点。
- 未设置 URL 时行为与此前一致：回退 builtin 或进程内 LLM，并记一次警告。
- 侧车实现可用任意语言，只要遵守同一 JSON-RPC 形状；无需改 `chat_engine` 主流程。

---

## 三 b、Directory（`plugins/` 目录插件，与 Remote 同协议）

- 在角色包 **`plugin_backends.* = directory`**，并在 **`directory_plugins`** 中为每个用到的槽填写 **`manifest.json` 的 `id`**（与 `plugins/<id>/` 目录名一致）。  
- 宿主扫描 `plugins/`、按 manifest 启动子进程、从 stdout 读取 JSON-RPC **base URL**，之后与 Remote 一样走 HTTP（方法名仍见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）。  
- 整壳 UI、`directory_plugin_invoke`、开发者模式与最小示例：**[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)**。

---

## 四、一般不通过 `plugin_backends` 换的部分

- **`LlmClient` 进程级实现**：换网关/云 API 可在 [`infrastructure/llm.rs`](../../src-tauri/src/infrastructure/llm.rs) 增加新实现并在 `AppState::new` 里注入；或通过 **`OCLIVE_REMOTE_LLM_URL`** 使用远程 JSON-RPC（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）。
- **`MemoryRepository`**：换向量库等属存储层，宜单独抽象或新 repository 实现，再考虑是否与 manifest 绑定。

---

## 五、相关文件索引

| 用途 | 路径 |
|------|------|
| 宿主聚合 | `src-tauri/src/domain/plugin_host.rs` |
| Remote HTTP 客户端 | `src-tauri/src/infrastructure/remote_plugin/` |
| 目录插件扫描 / 子进程 / RPC URL | `src-tauri/src/infrastructure/directory_plugins/` |
| 运行时解析 | `AppState::resolved_plugins_for` — `src-tauri/src/state/mod.rs` |
| 对话主链 | `src-tauri/src/domain/chat_engine/co_present.rs` 等 |
| 测试用演示 | `RoleManager::with_memory_retrieval` — `src-tauri/src/domain/role_manager.rs` |
