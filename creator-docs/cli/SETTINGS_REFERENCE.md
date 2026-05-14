# `settings.json` → `plugin_backends` 权威参考（内核向）

本文档描述 **桌面宿主（Tauri）** 与 **`oclive-cli` 脚手架** 共用的配置语义。单一事实来源以源码为准：

- 枚举与结构体：[`src-tauri/src/models/plugin_backends.rs`](../../src-tauri/src/models/plugin_backends.rs)
- 解析与绑定：[`src-tauri/src/domain/plugin_host.rs`](../../src-tauri/src/domain/plugin_host.rs)
- 协议与表格：[`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

**标准 JSON 无注释**：说明性文字请用 **`_` 前缀的键**（加载时忽略），或写在包外文档。`oclive-cli` 生成的示例包使用 `_comment_*` 键解释各槽。

---

## 一、六条宿主槽（`PluginBackends`）

运行时结构体 **`PluginBackends`** 含下列 **6** 个字段（Serde 反序列化时**忽略未知字段**，故 JSON 中可存在额外键如脚手架用的 `complex_emotion`，宿主不报错）。

| 字段 | 门面 trait（编排入口） | 常用内置实现（进程内） |
|------|-------------------------|-------------------------|
| `memory` | [`MemoryRetrieval`](../../src-tauri/src/domain/memory_retrieval.rs) | 默认 `MemoryBackend::Builtin` |
| `emotion` | 用户情绪分析（见 `plugin_host` / `EmotionAnalyzer`） | `EmotionBackend::Builtin` |
| `event` | 事件影响估计（`EventEstimator`） | `EventBackend::Builtin` |
| `prompt` | `PromptAssembler` / `PromptBuilder` | `PromptBackend::Builtin` |
| `llm` | `LlmClient` | **`LlmBackend::Ollama`**（默认本地客户端；**无 `builtin` 字面量**） |
| `agent` | [`AgentProvider`](../../src-tauri/src/domain/agent.rs) | `AgentBackend::Builtin` |

省略整段 `plugin_backends` 时：记忆 / 情绪 / 事件 / Prompt / Agent 为 **`builtin`**，**`llm` 为 `ollama`**（见 PLUGIN_V1 示例）。

### 1.1 各槽可选值（v1 枚举摘要）

完整表与 JSON-RPC 方法名见 **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**。下面是操作者最常用的取值：

| 槽位 | 常用值 | 选 `remote` / `directory` 时 |
|------|--------|------------------------------|
| memory | `builtin` / `builtin_v2` / `remote` / `directory` / `local` | `remote`：`OCLIVE_REMOTE_PLUGIN_URL`；`directory`：配置 `directory_plugins.memory` |
| emotion | `builtin` / `builtin_v2` / `remote` / `directory` | 同上 |
| event | `builtin` / `builtin_v2` / `remote` / `directory` | 同上 |
| prompt | `builtin` / `builtin_v2` / `remote` / `directory` | 同上 |
| llm | **`ollama`** / `remote` / `directory` | **`remote`**：`OCLIVE_REMOTE_LLM_URL`；可用 **`OCLIVE_LLM_BACKEND`** 在加载时覆盖 |
| agent | `builtin` / `remote` / `directory` | `remote`：侧车 JSON-RPC；`directory`：配置 `directory_plugins.agent` |

**不存在于 v1 枚举的字符串**（如字面量 `none`）会导致 **角色包解析失败**。若脚手架或文档写「none」，表示**逻辑上关闭/不声明**；写入主应用可加载的 JSON 时请 **省略该键**（回退默认）或改为合法枚举。

### 1.2 `directory_plugins` 对象

当任一槽为 **`directory`** 时，应在 `plugin_backends.directory_plugins` 中为对应槽填写 **`manifest.id`**（字符串）。详见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)。

---

## 二、第七条槽：`complex_emotion`（脚手架与路线图）

**当前 `PluginBackends` 不含此字段。** `oclive-cli` 将 `complex_emotion` 写在 **`plugin_backends` 对象内** 便于阅读；宿主反序列化时**忽略**该键，不影响 `load_role`。

- 若侧车实验需要独立进程：与 `remote` 其它子系统相同 wire，见 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)。
- 与 **`oclive-cli`** 的 `CONFIG_REFERENCE.md`、**`init --help`** 末尾矩阵一致。

---

## 三、`oclive-cli` 预设矩阵（逻辑 → JSON）

| 槽位 | minimal | mixed | full |
|------|---------|-------|------|
| memory / emotion / event / prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | **省略键**（语义 none） | builtin | builtin |
| complex_emotion | none | builtin | remote |

---

## 四、从 `builtin` / `ollama` 切换到 `remote`（步骤）

1. 准备侧车（HTTP JSON-RPC），实现 PLUGIN_V1 / REMOTE_PLUGIN_PROTOCOL 对应方法。
2. 在运行环境中设置 URL，例如 **`OCLIVE_REMOTE_PLUGIN_URL`**（多子系统共用侧车时）与 **`OCLIVE_REMOTE_LLM_URL`**（仅 LLM）。
3. 编辑 **`settings.json`** → `plugin_backends`：将目标槽改为 **`remote`**（`llm` 改为 **`remote`**，不是 `builtin`）。
4. 重启宿主或重新加载角色；观察日志中降级/回退提示（未配置 URL 时可能回退内置实现）。

---

## 五、相关文档索引

| 主题 | 文档 |
|------|------|
| CLI 使用与参数 | [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md) |
| 生成项目内预设表 | 运行 `init` 后的 **`CONFIG_REFERENCE.md`** |
| 插件与侧车总览 | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| 目录插件 | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| 编译期高耦合模式（Monolith，草案） | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)（`monolith.toml`，与 `plugin_backends` 运行时解耦） |
