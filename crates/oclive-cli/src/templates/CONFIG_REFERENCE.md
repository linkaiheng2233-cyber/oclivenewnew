# oclive-cli 生成项目：`plugin_backends` 预设对照

本文件由 **`oclive-cli init`** 自动生成，与 `init --help` 中的预设矩阵一致。正式契约以主仓 **[PLUGIN_V1.md](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)** 与 **`src-tauri/src/models/plugin_backends.rs`** 为准；权威说明见 **[SETTINGS_REFERENCE.md](../../../creator-docs/cli/SETTINGS_REFERENCE.md)**。

## 预设矩阵（逻辑槽位）

| 槽位 | minimal | mixed | full |
|------|---------|-------|------|
| memory | builtin | builtin | builtin |
| emotion | builtin | builtin | builtin |
| event | builtin | builtin | builtin |
| prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | none（JSON 省略键，回退宿主默认 builtin） | builtin | builtin |
| complex_emotion | none | builtin | remote |

说明：

- **`llm`**：主应用 v1 枚举为 **`ollama` \| `remote` \| `directory`**，无字面量 `builtin`。对照表中「本地默认」对应 JSON 中的 **`ollama`**（进程内 Ollama 客户端）。
- **`agent` = none**：内核结构体无 `none` 变体；脚手架在 **`settings.json` 中省略 `agent` 键**，加载时与显式 **`builtin`** 等价（均为默认内置实现）。
- **`complex_emotion`**：当前桌面宿主 **`PluginBackends` 仅含六槽**；该键写在 **`plugin_backends` 内便于阅读**，宿主反序列化时会**忽略未知字段**，不影响 `load_role`。

## 各槽一句话

| 槽位 | 作用 |
|------|------|
| memory | 记忆检索与排序 |
| emotion | 用户情绪分析 |
| event | 事件影响估计 |
| prompt | 主 prompt 组装 |
| llm | 主对话与短分类生成 |
| agent | 工具编排 / ReAct |
| complex_emotion | 复杂情感扩展（路线图；键保留供侧车实验） |

## 切换后端（概要）

1. 编辑 **`roles/default/settings.json`** 的 `plugin_backends` 对应字段。
2. **`remote`**：配置 **`OCLIVE_REMOTE_PLUGIN_URL`** / **`OCLIVE_REMOTE_LLM_URL`** 等（见 PLUGIN_V1 与 REMOTE_PLUGIN_PROTOCOL）。
3. **`directory`**：在包内配置 **`plugin_backends.directory_plugins`** 各槽的 manifest **`id`**，并放置 **`plugins/<id>/`**（见 DIRECTORY_PLUGINS.md）。
