# 设置项分级指南（L1–L4）

## 终端用户：统一设置中心怎么用

1. **纯聊 / 沉浸**：纯聊下侧栏条目较少；沉浸模式下侧栏展示完整分区（行为与偏好、模型、角色与数据、插件、内核与实验、系统与内核等）。顶栏「更多」或纯聊辅助区可打开同一套设置窗。
2. **设置中心开发者总闸**（仅沉浸、设置窗顶部）：关闭时侧栏隐藏 V2 专用项（专家模型工作台、V2 插件管理 / 市场、实验开关页、市场索引开发者区、Agent 调试说明等），避免日常用户误入高影响路径；开启后上述入口出现，且右栏顶部会显示 **高级功能** 提示条与 **返回常规设置**。
3. **侧栏分组提示**：分组标题旁的小帮助图标可一眼看到该组大致包含语言/快捷键、模型、角色数据、插件、实验、系统诊断等哪类内容。
4. **L1–L4 徽章**：侧栏每项旁的缩写与右栏 `SettingsTierSection` 标题对应风险级别；展开 **L4** 区块前通常需二次确认。

面向**开发者与创作者**：下文说明桌面应用内「设置」浮层如何按风险与契约影响划分四级，以及代码与文档中的对应关系。

## 四个级别（用户向名称）

| 级别 | 用户向区块标题（i18n） | 含义（摘要） |
|------|------------------------|----------------|
| **L1** | 外观与交互 | 主题、语言、快捷键**说明**、列表密度等；不触及内核契约与网络出站策略。 |
| **L2** | 行为与偏好 | 日常可调、多数可逆；影响习惯但一般不改变持久化安全模型。 |
| **L3** | 角色与数据 | 角色包路径、manifest / `settings.json` 强相关、目录插件在设置页的嵌入、市场浏览说明等。 |
| **L4** | 系统与内核 | 云端 LLM 凭据与入网、本机模型管理入口、`plugin_backends`、MCP/Agent 调试、强制 iframe、实验开关、**全局恢复默认**等。 |

侧栏导航行的 `L1`–`L4` 徽章与 `abbr` 说明来自 `settings.tiers.*`；右栏大区块标题来自 `settings.tiersUi.blockHeading.*`，与入口文案（如顶栏「更多」、纯聊模型表）应对齐使用 **「系统与内核」** 表述，避免与旧的「高级设置」「仅开发者」等碎片化说法混用。

## 代码落点

- **导航与侧栏徽章**：`src/lib/settingsNavKeys.ts`（`SETTINGS_NAV_ROWS`、`SettingsTier`）。
- **右栏分区与 L4 折叠 / 展开确认**：`src/views/SettingsView.vue` + `src/components/SettingsTierSection.vue`。
- **单源分级文案**：`src/lib/settingsNavCopy.ts`（`settingsTierBadge` / `settingsTierDescription`）。
- **全局「恢复默认」实现**：`src/lib/resetHostPreferencesToDefaults.ts`（仅 Tauri/DB 侧；Pinia 刷新由 `SettingsView` 调用方组合）。

## 使用与产品注意事项

1. **L4 默认折叠**：展开前会二次确认，避免误触高影响项。
2. **全局恢复默认**：仅覆盖「宿主偏好」子集（见设置页内 `settings.globalReset.scope` 文案），**不**卸载插件、**不**改写角色包目录文件；全局 Ollama 模型 id 会恢复为与空库时内核一致的默认值（当前为 `qwen2.5:7b`，与 `resetHostPreferencesToDefaults` 常量对齐）。
3. **同一控件勿跨级**：若某能力在简单模式下为 L2、展开后为 L4，应在 UI 上拆成两个区块并分别标注层级（参见「自定义快捷键」：L1 说明与 L4 编辑分区）。
4. **改 IA 时**：同步更新 `settingsNavKeys.ts` 末尾注释表、`SettingsView.vue` 与 i18n，并跑 `npm run build`。

## 相关文档

- 设置信息架构与深链：`handoff/SETTINGS_IA_TRUTH_TABLE.md`（仓库根 `handoff/`）。
- 配置文件位置（`plugin_state`、`ui.json` 等）：`creator-docs/guides/CONFIGURATION_FILES.md`。
