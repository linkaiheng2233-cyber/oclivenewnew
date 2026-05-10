# 设置中心结构（拆分后）

沉浸模式下，右栏顶部有 **开发者模式** 总闸；关闭时侧栏不展示下列「门控」项。

## 侧栏树（逻辑分组）

```mermaid
flowchart TB
  subgraph L1["L1 常规"]
    OV[概览]
    LANG[语言与区域]
  end
  subgraph L2b["L2 行为"]
    SH[快捷键管理]
    DM[默认对话模型]
  end
  subgraph Data["角色与数据"]
    ROLES[角色管理]
  end
  subgraph Gated["开发者模式开启后可见"]
    MC[云端模型与密钥]
    MO[本机模型与 Ollama]
    ME[专家模型工作台]
    PD[目录插件 · 设置插槽]
    PI[已安装与市场]
    PS[界面插槽顺序]
    PB[后端模块]
    PV[V2 管理]
    MB[插件市场]
    SEC[安全与隐私]
    AE[插件管理 V2 实验]
    AD[扩展区 settings.advanced]
    SD[开发者模式与索引源]
    DD[诊断与调试]
    DA[Agent / MCP 调试]
  end
  OV --> LANG
  LANG --> SH
  SH --> DM
  DM --> ROLES
  ROLES --> MC
```

纯聊模式仍受 `filterSettingsNavRows` 约束（与既有 IA 一致）。

## 右栏主面板映射

| 侧栏 id | 主组件 / 说明 |
|---------|----------------|
| `generalOverview` | 概览 + 全局恢复默认 |
| `generalLanguage` | 语言选择 |
| `shortcutsManage` | `ShortcutsManagerPanel`（内置表 + launcher 槽 + `HotkeySettingsSection`） |
| `generalDefaultModel` | `ModelSelectorSettings`（模型中心 + 跳转本机页） |
| `dataRoles` | `RoleManagerSettings`（列表/搜索/导入导出/删除） |
| 门控项 | 各 `SettingsTierSection` + 插件嵌入 / 深链按钮（见 `SETTINGS_DEVELOPER_GATED_NAV_IDS`） |

## 相关源文件

- `src/lib/settingsNavKeys.ts` — `SETTINGS_NAV`、`SETTINGS_NAV_ROWS`、`SETTINGS_DEVELOPER_GATED_NAV_IDS`
- `src/views/SettingsView.vue` — 侧栏过滤、顶部开发者闸、`systemDeveloper` 仅索引源
- `src/components/settings/RoleManagerSettings.vue`
- `src/components/settings/ModelSelectorSettings.vue`
- `src/components/settings/ShortcutsManagerPanel.vue`
- `src/components/HotkeySettingsSection.vue` — 重复快捷键提示
