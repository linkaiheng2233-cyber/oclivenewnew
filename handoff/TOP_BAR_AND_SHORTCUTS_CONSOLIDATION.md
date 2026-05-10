# 顶栏「更多」收束、设置页完善与快捷键对照（收尾记录）

## 一、顶栏「更多」精简（现状描述）

- **保留**：沉浸模式下「更多」面板中的叙事/场景/虚拟时间等沉浸专有区块；右侧 **设置入口** 磁贴含 **快捷键说明**、**设置** 主按钮，以及说明文案（`hubHint` / `pureChatHubHint`）。
- **移除（重复入口）**：本机模型、打开角色包文件夹、插件管理、插件市场、独立调试 tile、侧栏 `RolePackBar` 等；这些能力改由 **设置** 侧栏分区或深链面板承接。
- **纯聊**：「更多」侧栏与沉浸一致可见；纯聊底部辅助区主 CTA 为 **打开设置**（`app.pureChatAssist.openSettings`）。

## 二、设置页细节完善清单

| 区域 | 空状态 | 加载 / 错误 | 跨级引导 |
|------|--------|-------------|----------|
| 侧栏 | — | — | 搜索框 `settings.nav.filterLabel` / `filterPlaceholder` |
| 默认对话模型 `ModelSelectorSettings` | `emptyHint`（无 Ollama 且无可用云端选项时） | `loading`、`retry` + 错误文案与重试按钮 | 「云端模型与密钥」「本机模型与 Ollama」侧栏跳转按钮 |
| 云端模型 | — | 既有流程 | 「侧栏：默认对话模型…」 |
| 本机模型 | — | — | 「侧栏：默认对话模型…」 |
| 角色管理 `RoleManagerSettings` | `emptyLead` + 打开市场（`emit('openMarket')` → 设置 `deepLink` 插件市场） | — | — |
| 自定义快捷键 `HotkeySettingsSection` | — | 首屏 `getHotkeyBindings` 失败：`loadError` + **重新加载**；保存时用独立 `saving`，不遮挡已加载表单 | — |

## 三、快捷键与设置 UI 入口对照

| 快捷键 / 行为 | 作用 | 设置内 UI 入口 |
|---------------|------|----------------|
| `{m}+Shift+S` | 打开设置 | 顶栏「更多」→ **设置** |
| `{m}+Shift+F` | 打开插件管理（V1/V2 依实验开关） | **设置** → 插件与扩展（已安装 / 插槽 / 后端 / V2 等导航） |
| `{m}+Shift+A` | 打开插件市场 | **设置** → 插件市场；角色空态「打开插件市场」 |
| `{m}+Shift+D` | 开关调试面板 | **设置** → 诊断与调试（嵌入区 + 深链打开独立窗） |
| `{m}` 长按 ~1s（沉浸） | 打开快捷键说明对话框 | **设置** → 快捷键说明；顶栏「更多」→ **快捷键说明** |
| 用户自定义全局快捷键（`HotkeyHost`） | 打开启动器列表 / 插件插槽 | **设置** → 自定义快捷键（`HotkeySettingsSection`） |

说明：纯聊下 `F`/`A`/`S` 仅 `preventDefault`，避免误触；**调试** 与 **`{m}+Shift+D`** 在纯聊仍可用，且调试入口在设置 **诊断与调试** 中可查。

## 四、相关文件（便于复查）

- `src/App.vue` — 「更多」布局与设置入口 rail、`ShortcutHelp` 显示条件等。
- `src/views/SettingsView.vue` — 侧栏筛选、`RoleManagerSettings` 市场深链、模型相关跨级按钮。
- `src/components/settings/ModelSelectorSettings.vue`、`RoleManagerSettings.vue`
- `src/components/HotkeySettingsSection.vue`、`ShortcutHelp.vue`
- `src/i18n/locales/zh-CN.ts`、`en-US.ts`
