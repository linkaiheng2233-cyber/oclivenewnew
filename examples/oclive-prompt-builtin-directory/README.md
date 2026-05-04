# 内置 Prompt 组装 — 目录插件形态（Kernel V2 阶段 7-1）

将 **`PromptAssembler::build_prompt`** 与进程内 **`oclive_prompt_builtin::PromptBuilder`** 等价的正文，以 **Node JSON-RPC 侧车 + Rust 子进程 `oclive_prompt_from_json`** 提供，供 `plugin_backends.prompt = directory` 使用。**`prompt.top_topic_hint`** 在本示例中用与 **`TopicHintContext::top_topic_name_for_scene`** 一致的轻量 JS 实现。

## 与进程内 `default-prompt-providers` 的关系

- 官方 **`full`** profile 通过 **`default-prompt-providers`** 链接 **官方默认 Prompt 模块** **`oclive_prompt_builtin`**（进程内 Builtin），无需安装本插件即可使用 `prompt = builtin` / `builtin_v2`。
- 若构建时 **关闭** `default-prompt-providers`，宿主 **无** 进程内 Builtin；此时可将角色包 `prompt` 设为 **`directory`**，并把本插件 id 填入 `directory_plugins.prompt`，以 **子进程 RPC** 恢复组装能力（需授予 **`process:spawn`**，且需本机 **Node.js 18+**）。

## 构建 `oclive_prompt_from_json`

与宿主 `prompt.build_prompt` 的 JSON `params` 兼容的辅助二进制（stdin 一帧 JSON → stdout `{"prompt":"..."}`）：

```bash
cargo build -p oclive_prompt_builtin --features prompt-from-json-bin --bin oclive_prompt_from_json
```

将生成可执行文件路径加入环境变量 **`OCLIVE_PROMPT_FROM_JSON`**（Windows 下为 `.exe` 完整路径），**`rpc_server.mjs`** 会在处理 `prompt.build_prompt` 时 `spawn` 该二进制。

## 安装

1. 将本目录复制到与 `roles` 同级的 `plugins/com.oclive.builtin.prompt/`，或把父目录加入 `oclive_host_plugins.json` 的 `extra_plugin_roots`（见 `DIRECTORY_PLUGINS.md`）。
2. 在角色 `settings.json` 的 `plugin_backends` 中设置 `prompt: "directory"`，并配置 `directory_plugins.prompt: "com.oclive.builtin.prompt"`。
3. 设置 **`OCLIVE_PROMPT_FROM_JSON`** 指向已构建的 `oclive_prompt_from_json`（见上）。

## 协议

- **`prompt.build_prompt`**：参数形状与宿主 `RemotePromptAssemblerHttp` 序列化的快照一致（含完整 `role` 等字段），返回 `{ "prompt": string }`（或兼容的纯字符串，本实现返回对象）。
- **`prompt.top_topic_hint`**：参数 `topic_hint_context`、`scene_id`，返回 `{ "hint": string | null }`。

## 市场 / 打包

- `manifest.json` 符合目录插件扫描约定；若上架 `.oclive-plugin`，请沿用本目录结构并补全市场元数据（见 `PLUGIN_V1.md`）。
