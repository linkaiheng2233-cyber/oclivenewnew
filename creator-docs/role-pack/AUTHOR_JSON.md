# author.json（创作者建议）

可选文件，与 `manifest.json`、`settings.json` 同位于角色包根目录（`roles/{id}/author.json`）。

仓库内参考示例：`roles/mumu/author.json`（文案 + `recommended_plugins`；插槽布局仍用同目录 `ui.json`，避免重复维护 `suggested_ui`）。

## 与 `ui.json` 的关系

- **`suggested_ui`**（可选，JSON 形状与 `ui.json` 相同）：若存在且**非空**（与运行时 `UiConfig::is_effectively_empty` 一致），则作为**插件 UI 状态**（`plugin_state.json`）首次种子与「重置为角色包推荐」的基线。
- 否则基线回退为 **`ui.json`**（与旧版行为一致）。
- 用户覆盖仍保存在应用数据目录的 **`plugin_state.json`**（按角色），不由 `author.json` 覆盖。

## 与 `settings.json` 的关系

- **`settings.json`** 仍是引擎侧 **`plugin_backends`** 等字段的权威来源；`author.json` 不替代它。
- **`suggested_plugin_backends`**（可选，形状同 `settings.json` → `plugin_backends`）：仅作建议；宿主 UI 可在用户确认后写入**会话级**后端覆盖（不写回磁盘上的 `settings.json`）。

### 会话级 vs 未来的「用户默认后端」

- **当前实现**：在插件管理（或等价入口）中「应用作者建议后端」时，写入的是**当前会话命名空间**下的后端覆盖，随会话生命周期管理；**不会**把该选择持久化为「所有角色、所有会话」的全局默认。
- **若产品需要跨会话默认**：可另增应用数据文件（例如 `user_plugin_backends.json`）或在现有全局配置中增加字段，并在 `effective_plugin_backends_for_session` 的解析链中插入一层「用户默认」；与本文档所述会话覆盖区分开即可。`settings.json` 仍保持为角色包随包分发的引擎默认值。

## 字段概要

| 字段 | 说明 |
|------|------|
| `schema_version` | 建议 `1` |
| `summary` / `detail_markdown` | 角色简介与详情（Markdown） |
| `recommended_plugins` | 推荐目录插件：`id`、`version_range`、可选 `slots`、`for_backends`、`optional`、`note` |
| `suggested_ui` | 同 `ui.json` |
| `suggested_plugin_backends` | 同 `plugin_backends` |

详见实现：`oclivenewnew` 仓库 `src-tauri/src/models/author_pack.rs`。
