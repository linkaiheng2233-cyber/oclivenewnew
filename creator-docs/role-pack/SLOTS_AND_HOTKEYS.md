# 语义插槽、多外观与快捷键

## 官方插槽名（嵌入 `ui_slots`）

宿主识别以下 **10** 个语义插槽（`manifest.json` → `ui_slots[].slot`）：

| 插槽名 | 典型挂载位置 |
|--------|----------------|
| `chat_toolbar` | 聊天输入区上方工具栏 |
| `settings.panel` | 设置 → 插件扩展 |
| `role.detail` | 左侧角色详情区 |
| `sidebar` | 侧栏扩展 |
| `chat.header` | 聊天列顶部 |
| `settings.plugins` | 插件管理面板内 |
| `settings.advanced` | 设置对话框「常规」扩展 |
| `overlay.floating` | 主界面右下浮层 |
| `launcher.palette` | 快捷键说明浮层内 |
| `debug.dock` | 调试面板内 |

## 多外观（`appearance_id`）

同一插件可在**同一 `slot`** 下声明多条 `ui_slots`，每条需唯一 **`appearance_id`**（空字符串表示「默认」单外观，同一槽至多一条空 id）。

- 用户选择保存在 `plugin_state` 的 `slot_appearance`：`plugin_id` → `slot` → `appearance_id`。
- 角色包 `ui.json` / `author.suggested_ui` 可在各槽的 `slots.<key>.appearance` 中写入 **插件 id → 默认 appearance_id**（多外观时编写器可自动填首项）。

## 全局快捷键

配置存于应用数据目录 **`hotkey_bindings.json`**（与 `plugin_state.json` 并列）。每条绑定含 **`enabled`**：为 `false` 时不注册系统全局快捷键。

动作类型：

- **`openPluginSlot`**：`plugin_id`、`slot`、可选 `appearance_id`，打开对应 bootstrap 页（浮层 iframe）。
- **`openLauncherList`**：打开简易插件目录列表。

创作者可在文档或市场说明中给出「建议快捷键」，**不**占用系统键位；用户于设置中自行启用。

## 市场元数据（可选）

上架描述 JSON 可包含 **`uiSlotVariants`**（与宿主目录 API 形状类似）：`{ slot, appearanceId, label }[]`，便于检索「多外观」插件；非强制。
