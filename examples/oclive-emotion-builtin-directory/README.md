# 内置七维情绪分析 — 目录插件形态（Kernel V2 阶段 5-2）

将 **`UserEmotionAnalyzer::analyze`** 中与 `BuiltinUserEmotionAnalyzer` 等价的逻辑，以 **Node JSON-RPC 侧车** 提供，供 `plugin_backends.emotion = directory` 使用。

## 与进程内 `default-emotion-providers` 的关系

- 官方 **`full`** profile 通过 **`default-emotion-providers`** 链接 **官方默认情绪模块** **`oclive_emotion_builtin`**（进程内 Builtin）。
- 若构建时 **关闭** `default-emotion-providers`，可将角色包 `emotion` 设为 **`directory`**，并把本插件 id 填入 `directory_plugins.emotion`，以 **子进程 RPC** 恢复分析能力（需授予 **`process:spawn`**）。

## 安装

1. 将本目录复制到 `plugins/com.oclive.builtin.emotion/`，或把父目录加入 `oclive_host_plugins.json` 的 `extra_plugin_roots`。
2. 在角色 `settings.json` 中设置 `plugin_backends.emotion: "directory"`，并配置 `directory_plugins.emotion: "com.oclive.builtin.emotion"`。
3. 本机需 **Node.js 18+**。

## 协议

- 实现 **`emotion.analyze`**，参数 `{ "text": string }`，返回七维 `EmotionResult`（与 `oclive_kernel_core::models::EmotionResult` 字段一致）。
