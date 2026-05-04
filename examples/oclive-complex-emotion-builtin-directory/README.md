# 内置复杂情感关键词 — 目录插件形态（Kernel V2 阶段 5-3）

将 **`ComplexEmotionProvider::resolve_turn`** 中与 `BuiltinKeywordComplexEmotionProvider` 等价的逻辑，以 **Node JSON-RPC 侧车** 提供。

## 与进程内 `default-complex-emotion-providers` 的关系

- 官方 **`full`** profile 通过 **`default-complex-emotion-providers`** 链接 **官方默认复杂情感模块** **`oclive_complex_emotion_builtin`**。
- 关闭该 feature 后，可将 `complex_emotion` 设为 **`directory`** 并指向本插件 id（需 **`process:spawn`**）。

## 协议

- 方法 **`complex_emotion.resolve_turn`**，`params` 为 `ComplexEmotionInput` 扁平对象（见 `REMOTE_PLUGIN_PROTOCOL.md` §4.8），`result` 为 `ComplexEmotionOutput`。
