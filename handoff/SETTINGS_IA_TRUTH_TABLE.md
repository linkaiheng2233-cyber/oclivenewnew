# 设置中心信息架构真值表

与 [`settingsNavKeys.ts`](../src/lib/settingsNavKeys.ts) 中的 **稳定菜单 id** 对齐；L 级含义见 [`settingsNavCopy`](../src/lib/settingsNavCopy.ts) 引用的 `settings.tiers.*` 文案。

| 功能域（用户感知） | L 级 | 现有入口 / 实现文件 | 目标父菜单 id | 目标子菜单 id |
|-------------------|------|---------------------|---------------|---------------|
| 设置总览与纯聊边界说明；全局「恢复默认宿主偏好」 | L1+L4 | `SettingsView` 首段 + `resetHostPreferencesToDefaults.ts` | — | `settings.general.overview` |
| 语言与区域 | L1 | `SettingsView` + `uiStore.languagePref` | — | `settings.general.language` |
| 快捷键说明（沉浸） | L1 | `SettingsView` + `pluginManagerEntryCopy` | — | `settings.shortcuts.main` |
| 云端 LLM 说明 / 信任 / QuickSetup | L4 | `SettingsView`、`CloudLlmQuickSetup.vue`、`useCloudLlmTrustModal` | `settings.cat.models`（分组标题） | `settings.models.cloud` |
| 本机模型 / Ollama / 路径 | L4 | `LocalModelManagerPanel.vue`、`BuiltinLlamaModelManager.vue`；顶栏「更多」 | `settings.cat.models` | `settings.models.ollama`（深链打开本机模型管理） |
| 专家模型工作台 | L4 | `ExpertModelsPanel.vue`（`PluginManagerPanel` 内嵌）、`pluginStore.requestOpenExpertModelsWorkbench` | `settings.cat.models` | `settings.models.expert`（深链） |
| 目录插件 settings.panel 槽顺序 | L3 | `PluginSettingsPanelSlots.vue` | `settings.cat.plugins`（分组标题） | `settings.plugins.directory` |
| 自定义快捷键（L1 说明 + L4 编辑） | L1+L4 | `SettingsView` 分区 + `HotkeySettingsSection.vue`（`headless`） | `settings.cat.plugins` | `settings.plugins.hotkeys` |
| 已安装插件 / 持久化 / 市场入口（V1「插件」Tab） | L3 | `PluginManagerPanel.vue` → `plugins` | `settings.cat.plugins` | `settings.plugins.linkInstalled`（深链 `openPanel('plugins')`） |
| 界面插槽顺序（V1「插槽」Tab） | L3 | `PluginManagerPanel.vue` → `slots` | `settings.cat.plugins` | `settings.plugins.linkSlots`（深链 `openPanel('slots')`） |
| plugin_backends 后端模块（V1 Tab） | L4 | `PluginManagerPanel.vue` → `backends` | `settings.cat.plugins` | `settings.plugins.linkBackends`（深链） |
| V2 管理（槽位 · Git · 本地 Llama） | L4 | `PluginManagerV2Panel.vue`、`PluginManagerV2.vue` | `settings.cat.plugins` | `settings.plugins.v2Hub`（`emit('openPluginV2')`，关闭设置后打开） |
| 插件管理 V2 预览 | L4 | `PluginManagerV2Panel.vue`、`usePluginManagerWindow.openPluginManagerV2Preview` | `settings.cat.advanced`（分组标题） | `settings.advanced.experimental`（开关 + 打开预览） |
| 插件市场浏览 / 安装 | L3 | `PluginMarketPanel` / `PluginMarketV2Panel`、`pluginStore.openMarketPanel` | `settings.cat.market` | `settings.market.browse`（深链） |
| 市场开发者模式与索引源（L3 说明 + L4 控件） | L3+L4 | `SettingsView` + `getPluginMarketSourcesConfig` / `setPluginIndexSources` | `settings.cat.advanced` | `settings.advanced.marketSources` |
| 强制 iframe | L4 | `SettingsView` → `pluginStore.pluginState.force_iframe_mode` | `settings.cat.security` | `settings.security.host` |
| settings.advanced 嵌入 | L3+ | `PluginSlotEmbed` `SLOT_SETTINGS_ADVANCED` | `settings.cat.advanced` | `settings.advanced.embed` |
| 诊断 / 调试面板（L2 说明 + L4 深链） | L2+L4 | `SettingsView` + `DebugPanel.vue`、`debugStore` | `settings.cat.diagnostics` | `settings.diagnostics.debug`（深链） |
| Agent / MCP 调试台 | L4 | `AgentDebugPanel.vue`（嵌于 `PluginBackendSessionPanel` → 经典「后端」页） | `settings.cat.diagnostics` | `settings.diagnostics.agent`（深链 `openPanel('backends')`） |

## 纯聊 / 沉浸裁剪（与 `SettingsView` 一致）

| 菜单 id | 纯聊 | 沉浸 |
|---------|------|------|
| `settings.general.*` | 可见 | 可见 |
| `settings.models.cloud` | 可见 | 可见 |
| `settings.security.host` | 可见 | 可见 |
| 其余上表「深链」或插件相关子项 | 隐藏 | 可见 |

## 深链关闭约定

从设置发起深链（本机模型、插件管理、市场、专家工作台、调试）时 **先关闭设置浮层**，再打开目标面板；与既有「从云端区块打开后端」行为一致。侧栏 **V2 管理** 使用 `openPluginV2` 事件，宿主侧 `openPluginManagerV2Preview` 同样会先关设置再打开 V2 窗。

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
- 与顶栏「更多」对齐的按钮文案：`pluginManager.entry.unifiedOpenPluginMarketCtaV1/V2`、`unifiedOpenDebugCta`，以及 V1 各 Tab / V2 Hub / Agent 深链 CTA（`unifiedOpenPluginManager*`、`unifiedOpenPluginManagerV2HubCta`、`unifiedOpenAgentDebugFromBackendsCta`），均由 `pluginManagerEntryCopy.ts` 导出供 `App.vue` 与 `SettingsView.vue` 共用。
