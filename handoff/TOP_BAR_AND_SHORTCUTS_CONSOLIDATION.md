# 顶栏「更多」收束、设置页完善与快捷键对照（收尾记录）

## 一、顶栏「更多」精简（现状描述）

- **保留**：沉浸模式下「更多」面板中的叙事/场景/虚拟时间等沉浸专有区块；右侧 **设置入口** 磁贴仅 **快捷键参考**、**设置** 主按钮，以及说明文案（`hubHint` / `pureChatHubHint`）。纯聊下额外保留 **模型（纯聊）** 表（`pureChatModels`）打开模型表。
- **移除（重复入口）**：本机模型、打开角色包文件夹、插件管理、插件市场、独立调试 tile、侧栏 `RolePackBar` 等；这些能力改由 **设置** 侧栏分区或深链面板承接。
- **纯聊**：「更多」侧栏与沉浸一致可见；纯聊底部辅助区主 CTA 为 **打开设置**（`app.pureChatAssist.openSettings`）。

**文案与设置中心**：顶栏 **设置** 按钮与 `settings.title` 一致（中文均为「设置」）；顶栏 **快捷键参考** 打开 `ShortcutHelp` 总表弹窗，与侧栏 **快捷键管理**（`settings.nav.items.shortcutsManage` → `ShortcutsManagerPanel` 内含绑定编辑）区分职责——前者速查，后者配置。

## 二、设置页细节完善清单

| 区域 | 空状态 | 加载 / 错误 | 跨级引导 |
|------|--------|-------------|----------|
| 侧栏 | — | — | 搜索框 `settings.nav.filterLabel` / `filterPlaceholder` |
| 默认对话模型 `ModelSelectorSettings` | `emptyHint`（无 Ollama 且无可用云端选项时）；无本机模型时的软卡片 + 跳转本机页 | 骨架屏、`retry` + 错误文案与重试按钮 | 来源说明、配置云端、L4 云端提示；「云端模型与密钥」「本机模型与 Ollama」侧栏按钮 |
| 云端模型 | — | 既有流程 | 「侧栏：默认对话模型…」 |
| 本机模型（侧栏页） | `settings.modelsOllama.downloadHint` | — | 「侧栏：默认对话模型…」 |
| 角色管理 `RoleManagerSettings` | `emptyTitle` + `emptyLead` + 导入 CTA | 列表加载骨架屏；`loadRoles` 失败：错误 + **重试** | 打开市场（`emit('openMarket')` → 设置 `deepLink`） |
| 专家模型设置 `ExpertModelsSettingsHub` | 无当前角色提示 | `expertModelsStore` loading / error + 重试 | 工作台深链、恢复包默认 |
| 自定义快捷键 `HotkeySettingsSection` | — | 首屏 `getHotkeyBindings` 失败：`loadError` + **重新加载**；保存时用独立 `saving` | — |

## 三、快捷键与设置 UI 入口对照

内置组合键在 `App.vue` 的 `onHotkey` 中处理（与 Tauri 全局快捷键文件中的 **用户自定义** 绑定分离）。

| 快捷键 / 行为 | 作用 | 设置内 UI 入口 | 备选加速器 |
|---------------|------|----------------|------------|
| `{m}+Shift+S` | 打开设置 | 顶栏「更多」→ **设置**（同 `settings.title`） | **保留作为备选**（关设置窗后快速打开） |
| `{m}+Shift+F` | 打开插件管理（V1/V2 依实验开关） | **设置** → 插件与扩展（已安装 / 插槽 / 后端 / V2 等导航） | **保留作为备选** |
| `{m}+Shift+A` | 打开插件市场 | **设置** → 插件市场；角色空态「打开插件市场」 | **保留作为备选** |
| `{m}+Shift+D` | 开关调试面板 | **设置** → 诊断与调试（嵌入区 + 深链打开独立窗） | **保留作为备选** |
| `{m}` 长按 ~1s（沉浸） | 打开快捷键说明对话框 | **设置** → **快捷键管理**（内置表）；顶栏「更多」→ **快捷键参考** | **保留作为备选**（与设置页入口并列） |
| 用户自定义全局快捷键（`HotkeyHost` + `hotkeys.rs`） | 打开启动器列表 / 插件插槽等 | **设置** → **快捷键管理**（`HotkeySettingsSection`） | 用户自选；无内置重复则不在此表移除 |

**说明**：上述 `{m}+Shift+*` 与长按 `{m}` 均已能在设置 UI 中找到对应能力；组合键 **保留作为备选**，以降低仅键盘用户的路径成本。纯聊下 `F`/`A`/`S` 仅 `preventDefault`，避免误触；`{m}+Shift+D` 在纯聊仍可用于调试。

## 四、相关文件（便于复查）

- `src/App.vue` — 「更多」布局与设置入口 rail、`onHotkey`、设置深链分发。
- `src/views/SettingsView.vue` — 侧栏筛选、`RoleManagerSettings` 市场深链、模型相关跨级按钮、`SETTINGS_NAV_ROWS` 裁剪逻辑。
- `src/lib/settingsNavKeys.ts` — 侧栏稳定 id 与 tier / visibility 源真值。
- `src/components/settings/ModelSelectorSettings.vue`、`RoleManagerSettings.vue`、`ExpertModelsSettingsHub.vue`
- `src/components/HotkeySettingsSection.vue`、`ShortcutHelp.vue`
- `src/i18n/locales/zh-CN.ts`、`en-US.ts`
