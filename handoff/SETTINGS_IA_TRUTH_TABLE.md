# 设置中心信息架构真值表

与 [`settingsNavKeys.ts`](../src/lib/settingsNavKeys.ts) 中的 **稳定菜单 id**（`SETTINGS_NAV` / `SETTINGS_NAV_CAT` / `SETTINGS_NAV_ROWS`）对齐；L 级含义见 [`settingsNavCopy`](../src/lib/settingsNavCopy.ts) 引用的 `settings.tiers.*` 文案。

## 侧栏稳定 id 与层级（与 `SETTINGS_NAV_ROWS` 一致）

下表为 **可选侧栏项**（不含仅作分组标题的 `SETTINGS_NAV_CAT.*`）。`depth`：`0` = 与分组同级顶格，`1` = 分组下缩进子项。`visibility`：`always` = 纯聊与沉浸均参与 `filterSettingsNavRows`；`immersive` = 仅沉浸模式显示。

| 侧栏 id（`SETTINGS_NAV` 值） | tier | depth | visibility | 说明 |
|-----------------------------|------|-------|--------------|------|
| `settings.general.overview` | L1 | 0 | always | 概览与纯聊边界 |
| `settings.general.language` | L1 | 1 | always | 语言与区域 |
| `settings.shortcuts.manage` | L2 | 1 | immersive | 快捷键管理（`ShortcutsManagerPanel` → `HotkeySettingsSection`） |
| `settings.general.defaultModel` | L2 | 1 | always | 默认对话模型 |
| `settings.models.cloud` | L4 | 1 | always | 云端模型与密钥 |
| `settings.models.ollama` | L4 | 1 | immersive | 本机模型与 Ollama（深链） |
| `settings.data.roles` | L3 | 1 | immersive | 角色管理 |
| `settings.data.expertModels` | L3 | 1 | immersive | 专家模型设置（`ExpertModelsSettingsHub.vue` + 工作台深链） |
| `settings.plugins.directory` | L3 | 1 | immersive | 目录插件 · 设置页插槽 |
| `settings.plugins.linkInstalled` | L3 | 1 | immersive | 已安装与市场（深链） |
| `settings.plugins.linkSlots` | L3 | 1 | immersive | 界面插槽顺序（深链） |
| `settings.plugins.linkBackends` | L4 | 1 | immersive | 后端模块（深链） |
| `settings.plugins.v2Hub` | L4 | 1 | immersive | V2 管理（`openPluginV2`） |
| `settings.market.browse` | L3 | 0 | immersive | 插件市场（深链） |
| `settings.security.host` | L4 | 0 | always | 安全与隐私 |
| `settings.advanced.experimental` | L4 | 1 | immersive | 插件管理 V2 实验 |
| `settings.advanced.embed` | L3 | 1 | immersive | 扩展区 `settings.advanced` |
| `settings.system.developer` | L4 | 1 | immersive | 开发者模式与索引源 |
| `settings.diagnostics.debug` | L2 | 0 | immersive | 诊断与调试 |
| `settings.diagnostics.agent` | L4 | 0 | immersive | Agent / MCP 调试（深链） |

分组标题行（不切换右栏，仅 i18n 展示）：`settings.cat.behavior`、`settings.cat.models`、`settings.cat.data`、`settings.cat.plugins`、`settings.cat.advanced`、`settings.cat.system`。

## 功能域与实现（补充映射）

| 功能域（用户感知） | L 级 | 现有入口 / 实现文件 | 目标父菜单 id | 目标子菜单 id |
|-------------------|------|---------------------|---------------|---------------|
| 设置总览与纯聊边界说明；全局「恢复默认宿主偏好」 | L1+L4 | `SettingsView` 首段 + `resetHostPreferencesToDefaults.ts` | — | `settings.general.overview` |
| 语言与区域 | L1 | `SettingsView` + `uiStore.languagePref` | — | `settings.general.language` |
| 快捷键管理（内置表 + 全局绑定编辑） | L2 | `SettingsView` + `ShortcutsManagerPanel.vue` → `HotkeySettingsSection.vue` | `settings.cat.behavior` | `settings.shortcuts.manage` |
| 默认对话模型（与撰写区同步） | L2 | `ModelSelectorSettings.vue` → `HostModelPickRow` + `useHostModelPick` | `settings.cat.behavior` | `settings.general.defaultModel` |
| 云端 LLM 说明 / 信任 / QuickSetup | L4 | `SettingsView`、`CloudLlmQuickSetup.vue`、`useCloudLlmTrustModal` | `settings.cat.models` | `settings.models.cloud` |
| 本机模型 / Ollama / 路径 | L4 | `LocalModelManagerPanel.vue` 等；**设置**侧栏深链 | `settings.cat.models` | `settings.models.ollama` |
| 专家模型（生效图、工作台、恢复包默认） | L3+L4 | `ExpertModelsSettingsHub.vue`；工作台 `ExpertModelsPanel`（`PluginManagerPanel` 内嵌）、`pluginStore.requestOpenExpertModelsWorkbench` | `settings.cat.data` | `settings.data.expertModels`（侧栏）+ 深链 |
| 角色切换 / 简介 / 打开包目录 | L3 | `RoleManagerSettings.vue`；顶栏 `RoleSelector` 仍保留快速切换 | `settings.cat.data` | `settings.data.roles` |
| 目录插件 settings.panel 槽顺序 | L3 | `PluginSettingsPanelSlots.vue` | `settings.cat.plugins` | `settings.plugins.directory` |
| 已安装插件 / 持久化 / 市场入口（V1「插件」Tab） | L3 | `PluginManagerPanel.vue` → `plugins` | `settings.cat.plugins` | `settings.plugins.linkInstalled` |
| 界面插槽顺序（V1「插槽」Tab） | L3 | `PluginManagerPanel.vue` → `slots` | `settings.cat.plugins` | `settings.plugins.linkSlots` |
| plugin_backends 后端模块（V1 Tab） | L4 | `PluginManagerPanel.vue` → `backends` | `settings.cat.plugins` | `settings.plugins.linkBackends` |
| V2 管理（槽位 · Git · 本地 Llama） | L4 | `PluginManagerV2Panel.vue`、`PluginManagerV2.vue` | `settings.cat.plugins` | `settings.plugins.v2Hub` |
| 插件管理 V2 预览 | L4 | `PluginManagerV2Panel.vue`、`usePluginManagerWindow.openPluginManagerV2Preview` | `settings.cat.advanced` | `settings.advanced.experimental` |
| 插件市场浏览 / 安装 | L3 | `PluginMarketPanel` / `PluginMarketV2Panel`、`pluginStore.openMarketPanel` | —（侧栏顶层项） | `settings.market.browse` |
| 市场开发者模式与索引源（L3 说明 + L4 控件） | L3+L4 | `SettingsView` + `getPluginMarketSourcesConfig` / `setPluginIndexSources` | `settings.cat.system` | `settings.system.developer` |
| 强制 iframe | L4 | `SettingsView` → `pluginStore.pluginState.force_iframe_mode` | —（侧栏顶层项） | `settings.security.host` |
| settings.advanced 嵌入 | L3+ | `PluginSlotEmbed` `SLOT_SETTINGS_ADVANCED` | `settings.cat.advanced` | `settings.advanced.embed` |
| 诊断 / 调试（L2 说明 + L4 本页嵌入 + 独立窗） | L2+L4 | `SettingsDebugEmbed.vue` → `DebugPanel.vue`；`debugStore`；深链 | — | `settings.diagnostics.debug` |
| Agent / MCP 调试台 | L4 | `AgentDebugPanel.vue`（嵌于后端页） | — | `settings.diagnostics.agent` |

## 纯聊 / 沉浸裁剪（与 `SettingsView.visibleNavRows` 一致）

1. **`filterSettingsNavRows(immersive, rows)`**：仅 `visibility === "always"`，或 `visibility === "immersive"` 且当前为沉浸模式时，可选行可见。
2. **开发者总闸**：沉浸模式且插件市场「开发者模式」**关闭**时，从侧栏移除 `SETTINGS_DEVELOPER_GATED_NAV_IDS` 中的项（与 `SettingsView` 一致）；**不修改** `SETTINGS_NAV_ROWS` 源数据。

纯聊下（非沉浸）通常可见的 **子菜单 id** 包括：`settings.general.overview`、`settings.general.language`、`settings.general.defaultModel`、`settings.models.cloud`、`settings.security.host`。`settings.shortcuts.manage` 及插件/数据/诊断等 `immersive` 项在纯聊下隐藏。

## 深链关闭约定

从设置发起深链（本机模型、插件管理、市场、专家工作台、调试）时 **先关闭设置浮层**，再打开目标面板。侧栏 **V2 管理** 使用 `openPluginV2` 事件，宿主侧 `openPluginManagerV2Preview` 同样会先关设置再打开 V2 窗。

## 第三阶段：经典插件管理「本页嵌套」

| 机制 | 说明 |
|------|------|
| `pluginStore.panelEmbedHost` | `null`：由 `App.vue` 挂载 `PluginManagerPanel`（Teleport 全屏层）；`"settings"`：由 `SettingsView` 在设置窗底部挂载 `<PluginManagerPanel embedded />`，`App` 侧不挂载。 |
| `openPanelInSettingsEmbed(tab)` | 设置「已安装 / 插槽 / 后端」三页中「在本页打开」调用；`openPanel()` 与深链会先 `panelEmbedHost = null` 回到独立窗。 |
| `PluginManagerPanel` `embedded` | `Teleport` 禁用、根容器为 `pm-embed-outer`，不响应背景点击关闭。 |
| 关闭 | 设置窗关闭、`openPanel`、深链、或离开三 Tab 侧栏项时 `closePanel()` 会清空 `panelEmbedHost`。 |

## 全局恢复默认（概览 · L4）

| 机制 | 说明 |
|------|------|
| 入口 | `SettingsView` 概览页底部 `SettingsTierSection` L4：`settings.globalReset.*` 文案 + 确认框。 |
| 实现 | `resetHostPreferencesToDefaults`：清 `host_cloud_llm`、默认 `host_chat_model`、空快捷键文件、关市场开发者并清空索引 URL、`saveGlobalPluginState` 中 `force_iframe_mode: false`；随后 `uiStore` 关 V2 实验 + 语言 `system`、`pluginStore.refresh()`。 |

## i18n 前缀

- 侧栏标题：`settings.nav.items.<camelFromId>`（见 `settingsNavKeys.ts` 中 `settingsNavLabelKey`）。
- L 级徽章与说明：`settings.tiers.L1` … `settings.tiers.L4`（见 `settingsNavCopy.ts`）。
- 右栏大区块标题与 L4 折叠提示：`settings.tiersUi.*`；开发者向总览：`creator-docs/kernel/SETTINGS_TIERING.md`。
- 与设置深链、插件管理 CTA 对齐的文案：`pluginManagerEntryCopy.ts`（`App.vue` 与 `SettingsView.vue` 共用）。

## 2026-05-10 界面收束（记录）

本次收束与文档对齐的范围概览：

- **顶栏「更多」**：沉浸专有区块（互动模式、身份、外观、纯聊模型表 / 沉浸虚拟时间与叙事场景等）保留；右侧 **设置入口** 条仅保留 **快捷键参考** 与 **设置**，插件管理、市场、本机模型、调试、角色包文件夹等入口已迁出至 **设置中心** 侧栏或深链。
- **设置中心**：默认模型、云端/本机引导、角色管理空态与加载、**专家模型** 统一为 `settings.data.expertModels`（`ExpertModelsSettingsHub`）；侧栏 id 以 `settingsNavKeys.ts` 为准。
- **快捷键**：`App.vue` 内 `{m}+Shift+S/F/A/D` 等与 `TOP_BAR_AND_SHORTCUTS_CONSOLIDATION.md` 对照；全局自定义快捷键仍由 **快捷键管理** 页持久化并由后端注册。
- **本表**：与 `SETTINGS_NAV_ROWS` 全量对账，修正原 `settings.shortcuts.main`、`settings.plugins.hotkeys`、`settings.models.expert`（旧侧栏）等与实现不一致之处。
