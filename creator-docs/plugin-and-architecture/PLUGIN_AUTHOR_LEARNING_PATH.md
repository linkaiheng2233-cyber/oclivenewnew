# 插件作者学习路径

面向 **目录插件 / Remote 侧车 / 宿主槽扩展** 的开发者。契约权威：[PLUGIN_V1.md](PLUGIN_V1.md)；总架构图：[KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)。

---

## 入门（约 30 分钟）

| 步骤 | 做什么 | 读什么 |
|------|--------|--------|
| 0 | **快速开始**：一键生成插件骨架 | 在 oclivenewnew 根：`cargo run -p oclive-cli -- plugin create my-plugin --type directory --provides llm -o ./plugins/`；见 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) § `plugin create` |
| 0b | **依赖与市场** | `plugin install` / `plugin_dependencies`；`plugin test`；**`oclive market browse/search/install`**（推荐；`plugin search`/`update` 已 deprecated） |
| 1 | 建立「六宿主后端模块 + 设施模块」心智模型 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [PLUGIN_V1.md](PLUGIN_V1.md)（`complex_emotion` = **复杂情感专家模型设施子模块**，非宿主槽） |
| 2 | 理解 `plugin_backends` 与 `directory_plugins` | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |
| 3 | 三种后端差异 | **builtin**：进程内默认；**remote**：HTTP JSON-RPC 侧车；**directory**：`plugins/<id>/` 子进程 + 同 wire（见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)） |

**验收**：能口述每个槽可选的后端类型，并说明 `directory` 时 manifest `id` 如何与 `directory_plugins` 对齐。

---

## 进阶（约 1–2 小时）

| 主题 | 读什么 |
|------|--------|
| **目录插件** | [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)（manifest、`process`、整壳 / 插槽、`directory_plugin_invoke`、开发者模式） |
| **Remote 侧车** | [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)（JSON-RPC 形状与示例） |
| **权限与授权** | [PLUGIN_V1.md §权限规范](PLUGIN_V1.md) · 运行时与 grant：[handoff/A4_CLOSURE_SUMMARY.md](../../handoff/A4_CLOSURE_SUMMARY.md) |
| **桥接 `invoke`** | [BRIDGE_API_REFERENCE.md](BRIDGE_API_REFERENCE.md) |

**验收**：能列出自己的插件需要哪些 `permissions`，以及用户未授权时宿主会怎样降级/报错码。

---

## 高级（约半天）

| 主题 | 读什么 |
|------|--------|
| **市场发布流程** | [../roadmap/PLUGIN_WEB_SECTION.md](../roadmap/PLUGIN_WEB_SECTION.md) · [MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **`oclive_validation` 集成** | `crates/oclive_validation`（与宿主、编写器 wasm 同源）；三面一致测试 `src-tauri/tests/permission_three_way_consistency.rs` |
| **调试与排障** | 主应用插件管理 **Ctrl+Shift+F**；[FAQ.md](../FAQ.md)；错误码 [ERROR_CODES.md](../getting-started/ERROR_CODES.md)；目录 RPC 日志见 DIRECTORY_PLUGINS 与宿主 `tracing` |

**验收**：能独立起一个最小目录插件或 Remote demo，并在管理面板里完成启用 / RPC 探活 / 读日志排错。

---

## 延伸

- **LLM 目录插件 + llama.cpp**：[`examples/directory-plugin-llamacpp/README.md`](../../examples/directory-plugin-llamacpp/README.md)（与 Ollama 按角色包并存）  
- 替换内置后端（Rust）：[HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)  
- Monolith 焊接（无头/硬件）：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)
