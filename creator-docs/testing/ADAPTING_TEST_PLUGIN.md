# 为可替换后端编写 Rust 集成测试（ADAPTING_TEST_PLUGIN）

## `PluginHost` 与 `builtin_v2`

- **烟测示例**：[`src-tauri/tests/plugin_backends_v2_resolve.rs`](../../src-tauri/tests/plugin_backends_v2_resolve.rs)  
  验证在 `memory` / `emotion` / `event` / `prompt` 为 **`builtin_v2`**、`llm` 为 **`ollama`** 时，`PluginHost::resolve_for_role` 能解析 **六条子系统线**（含默认 **`agent`**）。
- **构造宿主**：`PluginHost::new(llm, None, std::env::temp_dir())`  
  第三参为 **应用数据根**（生产环境为 app data；测试可用 **临时目录**），供 MCP 配置扫描等子系统初始化。

## LLM 替身

- 使用 **`MockLlmClient`**（`src-tauri/src/infrastructure/`）实现 **`LlmClient`**，避免集成测试访问真实 Ollama。

## 目录 / Remote 插件

- **Remote**：可参考 CI job **`remote-plugin-demo`**（Python 最小侧车 + `memory.rank` JSON-RPC）。
- **Directory**：需 `DirectoryPluginRuntime` 与磁盘 `plugins/` 布局；集成测试成本较高，优先 **单测 + 烟测** 分层。

## 参阅

- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)  
- [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)
