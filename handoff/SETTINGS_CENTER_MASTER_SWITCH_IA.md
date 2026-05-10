# 设置中心结构（开发者总闸）

总闸 = 设置窗顶部粘性开关 **`settingsDeveloperMaster`**（`uiStore`，持久化）。  
沉浸模式下：总闸 **关** 时侧栏隐藏 V2 专用项；总闸 **开** 时与 `SETTINGS_NAV_ROWS` 过滤结果一致（仍受纯聊 / 沉浸行可见性约束）。

## 总闸关闭（沉浸 · `settingsDeveloperMaster === false`）

### 侧栏（相对「全开」减少的项）

隐藏以下 id（见 `SETTINGS_DEVELOPER_GATED_NAV_IDS` / `SETTINGS_CENTER_V2_ONLY_NAV_IDS`）：

- `dataExpertModels` — 专家模型设置（摘要 / Hub）  
- `dataExpertWorkbench` — 专家模型工作台（完整画布）  
- `pluginsV2Hub` — V2 插件管理（嵌入 `PluginManagerV2Panel`）  
- `marketBrowseV2` — 插件市场 V2（嵌入 `PluginMarketV2Panel`）  
- `advancedExperimental` — 插件管理 V2 实验开关页  
- `systemDeveloper` — 市场开发者模式与第三方索引源  
- `diagnosticsAgent` — Agent / MCP 调试说明页  

### 右栏

仍可与侧栏一一对应展示：**概览、语言、快捷键、默认模型、云端 / 本机模型、角色管理、目录插件、已安装·插槽·后端、插件市场（经典）、安全、扩展区、诊断与调试** 等。  
本页嵌入区：**本机模型管理**、**经典插件管理**、**经典市场**、**调试嵌入** 行为不变，仅通过上述可见侧栏进入。

---

## 总闸开启（沉浸 · `settingsDeveloperMaster === true`）

### 侧栏

在「关闭」基础上 **额外** 出现上述条目（分组标题仍按子项是否存在自动折叠）。

### 右栏（V2 已迁入设置 DOM，无 App 级独立浮层）

| 侧栏 id | 右栏 L4 嵌入 |
|---------|----------------|
| `pluginsV2Hub` | `PluginManagerV2Panel` **`embedded`**（需同时开启 `experimentalPluginManagerV2`） |
| `marketBrowseV2` | `PluginMarketV2Panel` **`embedded`**（需 `experimentalPluginManagerV2`） |
| `dataExpertWorkbench` | `ExpertModelsPanel` **`embedded`** |
| `dataExpertModels` | `ExpertModelsSettingsHub`；「打开工作台」跳转到 `dataExpertWorkbench` |
| `advancedExperimental` | V2 实验开关；按钮「打开 V2 插件管理」→ 侧栏 `pluginsV2Hub` |
| `systemDeveloper` | 市场开发者模式 + 第三方索引源 |
| `diagnosticsAgent` | 嵌入经典管理器「后端」页 |

**完成状态（界面闭环）**：`App.vue` 不再挂载 `PluginManagerV2Panel` / `PluginMarketV2Panel`；V2 仅通过设置侧栏进入，在右栏内流式布局渲染（内部子对话框使用 `Teleport :disabled="embedded"` 或等价样式，遮罩限制在嵌入容器内）。

---

## 与插件市场「开发者模式」的关系

- **总闸**（`uiStore.settingsDeveloperMaster`）：控制设置侧栏是否列出 V2 相关 **导航**。  
- **市场开发者模式**（`getPluginMarketSourcesConfig().developerMode` / `setPluginMarketDeveloperMode`）：控制是否可编辑 **第三方索引源**；开关位于 **系统与内核 → 开发者模式与索引源**（需总闸开启才见该侧栏项）。

---

## 快捷键（打开设置并定位）

| 快捷键 | 行为 |
|--------|------|
| `{Mod}+Shift+S` | 打开设置（不变） |
| `{Mod}+Shift+D` | 打开设置 → 诊断与调试，并刷新调试数据 |
| `{Mod}+Shift+F` | 总闸开且启用 V2 预览 → 设置 → `pluginsV2Hub`；否则 → 已安装与市场并嵌入经典「插件」页 |
| `{Mod}+Shift+A` | 总闸开且启用 V2 预览 → 设置 → `marketBrowseV2`；否则 → `marketBrowse` 并嵌入经典市场 |

纯聊模式下 `F` / `A` / `S` 仍不触发插件类快捷键（与原先一致）；`D` 仍可打开设置调试页。

---

## 收尾打磨（导航、总闸态、快捷键对齐）— 已完成

- **侧栏分组**：每个 `settings.cat.*` 分组标题旁增加 `HelpHint`，文案键 `settings.nav.groupHints.*`（中英），概括该组下常见能力，降低新用户扫侧栏时的认知成本。
- **高级区顶栏**：沉浸且 **`settingsDeveloperMaster === true`** 时，右栏顶部展示 `settings.advancedSurface.bannerLead`，并提供 **「返回常规设置」**（`goToRoutineSettings` → 概览 `generalOverview`）。
- **总闸关闭时的误导消除**：经典「插件市场」页内 **打开市场** 按钮文案（`unifiedOpenPluginMarketCta`）在设置内按 **`experimentalPluginManagerV2 && settingsDeveloperMaster`** 判定是否显示「V2」字样，避免总闸关仍显示 V2 市场 CTA。`marketBrowseV2` 侧栏引导句仍要求总闸 + V2 实验同时成立。
- **快捷键与设置页对齐**：在 **快捷键管理、已安装与市场、插槽、后端、插件市场、诊断与调试、Agent/MCP 调试** 等页增加 `settings.shortcuts.accelerator*` 灰字提示，与 `App.vue` 内置热键一致；详细对照见 `handoff/TOP_BAR_AND_SHORTCUTS_CONSOLIDATION.md`。
