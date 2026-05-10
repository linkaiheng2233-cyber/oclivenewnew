# 设置中心结构（开发者总闸）

总闸 = 设置窗顶部粘性开关 **`settingsDeveloperMaster`**（`uiStore`，持久化）。  
沉浸模式下：总闸 **关** 时侧栏隐藏 V2 专用项；总闸 **开** 时与 `SETTINGS_NAV_ROWS` 过滤结果一致（仍受纯聊 / 沉浸行可见性约束）。

## 总闸关闭（沉浸 · `settingsDeveloperMaster === false`）

### 侧栏（相对「全开」减少的项）

隐藏以下 id（见 `SETTINGS_DEVELOPER_GATED_NAV_IDS` / `SETTINGS_CENTER_V2_ONLY_NAV_IDS`）：

- `dataExpertModels` — 专家模型设置  
- `pluginsV2Hub` — V2 管理入口  
- `advancedExperimental` — 插件管理 V2 实验开关页  
- `systemDeveloper` — 市场开发者模式与第三方索引源  
- `diagnosticsAgent` — Agent / MCP 调试说明页  

### 右栏

仍可与侧栏一一对应展示：**概览、语言、快捷键、默认模型、云端 / 本机模型、角色管理、目录插件、已安装·插槽·后端、插件市场、安全、扩展区、诊断与调试** 等。  
本页嵌入区：**本机模型管理**、**经典插件管理**、**经典市场**、**调试嵌入** 行为不变，仅通过上述可见侧栏进入。

---

## 总闸开启（沉浸 · `settingsDeveloperMaster === true`）

### 侧栏

在「关闭」基础上 **额外** 出现上述 5 个条目（分组标题仍按子项是否存在自动折叠）。

### 右栏

在对应 id 下增加：

- **专家模型**：`ExpertModelsSettingsHub`、工作台入口（嵌入后端页）  
- **V2 Hub**：打开 V2 预览（叠层窗，仍仅从设置进入）  
- **实验**：`experimentalPluginManagerV2` 开关与「打开 V2 预览」  
- **系统开发者**：**市场开发者模式**（`setPluginMarketDeveloperMode`）+ 第三方索引源文本框  
- **Agent 调试**：跳转并嵌入经典管理器「后端」页  

---

## 与插件市场「开发者模式」的关系

- **总闸**（`uiStore.settingsDeveloperMaster`）：控制设置侧栏是否列出 V2 相关 **导航**。  
- **市场开发者模式**（`getPluginMarketSourcesConfig().developerMode` / `setPluginMarketDeveloperMode`）：控制是否可编辑 **第三方索引源**；开关位于 **系统与内核 → 开发者模式与索引源**（需总闸开启才见该侧栏项）。

---

## 快捷键（降级为「打开设置并定位」）

| 快捷键 | 行为 |
|--------|------|
| `{Mod}+Shift+S` | 打开设置（不变） |
| `{Mod}+Shift+D` | 打开设置 → 诊断与调试，并刷新调试数据 |
| `{Mod}+Shift+F` | 总闸开且启用 V2 预览 → 设置 → V2 Hub；否则 → 已安装与市场并嵌入经典「插件」页 |
| `{Mod}+Shift+A` | 设置 → 插件市场并嵌入经典市场 |

纯聊模式下 `F` / `A` / `S` 仍不触发插件类快捷键（与原先一致）；`D` 仍可打开设置调试页。
