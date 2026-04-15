# 目录插件：聊天工具栏插槽（非整壳）

本示例演示 **`manifest.ui_slots`**：无 `shell` 段，仅在主界面 **聊天输入区上方** 嵌入 `chat_toolbar` iframe。

## 使用

1. 将本目录复制到仓库根下 `plugins/com.oclive.example.ui_slot_toolbar/`（或把本目录加入 `oclive_host_plugins.json` 的 `extra_plugin_roots`）。
2. 启动 oclive 主界面（勿配置 `shell_plugin_id` 抢占整壳）。
3. 对话页输入框上方应出现窄条 iframe；文案会调用 `OclivePluginBridge.invoke('list_roles')` 显示角色数量。

## 契约

- `bridge.invoke` 须列出允许的 Tauri 命令名（与 `plugin_bridge_invoke` 实现一致）。
- 整壳插件（含 `shell`）不参与插槽；见 [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)。
