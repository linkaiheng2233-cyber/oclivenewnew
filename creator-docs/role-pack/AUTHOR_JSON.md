# author.json（创作者建议）

可选文件，位于角色包根目录（`distros/chat-pro/roles/{id}/author.json`）：当前蓝图包与 `pipeline.ocblueprint` 同级，legacy v1 包则与 `manifest.json`、`settings.json` 同级。

仓库内参考示例：`distros/chat-pro/roles/mumu/author.json`（文案 + `recommended_plugins`；插槽布局仍用同目录 `ui.json`，避免重复维护 `suggested_ui`）。

## 与 `ui.json` 的关系

- **`suggested_ui`**（可选，JSON 形状与 `ui.json` 相同）：若存在且**非空**（与运行时 `UiConfig::is_effectively_empty` 一致），则作为**插件 UI 状态**（`plugin_state.json`）首次种子与「重置为角色包推荐」的基线。
- 否则基线回退为 **`ui.json`**（与旧版行为一致）。
- 用户覆盖仍保存在应用数据目录的 **`plugin_state.json`**（按角色），不由 `author.json` 覆盖。

## 与角色包引擎配置的关系

- 当前蓝图包的引擎配置权威是 **`pipeline.ocblueprint`**：后端实例在 `slot_registry`，Stable v4 运行时配置在 `runtime_config`；`author.json` 不替代它。`settings.json` 仅是 legacy v1 权威。
- **`suggested_plugin_backends`**（可选）仍沿用六槽 `PluginBackends` 兼容形状。它只是作者建议；宿主在用户确认后，把建议映射到蓝图中名为 `memory` / `emotion` / `event` / `prompt` / `llm` / `agent` 的默认实例并写入**会话级槽位覆盖**，不修改磁盘角色包。不存在的默认实例键会跳过。

### 会话级 vs 未来的「用户默认后端」

- **当前实现**：在插件管理（或等价入口）中「应用作者建议后端」时，写入的是**当前会话命名空间**下的后端覆盖，随会话生命周期管理；**不会**把该选择持久化为「所有角色、所有会话」的全局默认。
- **若产品需要跨会话默认**：应另增应用数据文件或全局配置层，并在有效槽位解析链中插入「用户默认」；须与本文档的会话覆盖、角色包 `slot_registry` 默认值分开。目前没有这一层。

## 字段概要

| 字段 | 说明 |
|------|------|
| `schema_version` | 建议 `1` |
| `summary` / `detail_markdown` | 角色简介与详情（Markdown） |
| `recommended_plugins` | 推荐目录插件：`id`、`version_range`、可选 `slots`、`for_backends`、`optional`、`note` |
| `suggested_ui` | 同 `ui.json` |
| `suggested_plugin_backends` | 六槽 `PluginBackends` 兼容形状；映射默认实例键的会话建议，不是蓝图 SSOT |

详见实现：`oclivenewnew` 仓库 `kernel/crates/oclive_kernel_types/src/models/author_pack.rs`。
