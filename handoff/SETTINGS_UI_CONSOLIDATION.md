# 设置「界面收束」用户路径（v0.2+）

目标：**不依赖快捷键**也能在 **设置（顶栏「更多」→ 设置，或既有打开方式）** 内完成常见配置；与撰写区模型选择器、顶栏角色切换、独立调试窗等行为一致。

## 侧栏入口（稳定 id 见 `src/lib/settingsNavKeys.ts`）

| 侧栏文案（中文） | 菜单 id | 说明 |
|------------------|---------|------|
| 行为与偏好 → 模型·语言·通知 | `settings.general.behavior` | 单页分卡片：`ModelSelectorSettings` + 语言 `select`；与撰写区模型同源（`useHostModelPick`）。 |
| 角色与数据 → 角色管理 | `settings.data.roles` | 角色列表 + **专家模型摘要卡**（`ExpertModelsSettingsHub` compact）；顶栏角色下拉仍保留快速切换。 |
| 系统与内核 → 开发者模式与索引源 | `settings.system.developer` | 插件市场开发者总闸 + 第三方索引 URL 列表（仅此一处配置）。 |
| 诊断与调试 | `settings.diagnostics.debug` | **本页嵌入**完整调试面板内容；仍可用按钮打开**独立调试窗**（与顶栏「更多」及既有快捷键行为一致）。 |
| 快捷键管理 | `settings.shortcuts.manage` | `ShortcutsManagerPanel`；与行为与偏好页文案互链。 |

## 未改动的原则

- **不新增**全局功能专属快捷键；`Ctrl/Cmd+Shift+D` 等既有快捷键仍可用，设置内提供等价入口。
- 插件管理 **V1 本页嵌套**、**V2 预览** 等既有能力保持；详见 `SETTINGS_IA_TRUTH_TABLE.md` 第三阶段表。

## 姊妹应用

- 角色包深度编辑：安装 **oclive-pack-editor**，先在设置「角色管理」中打开包目录，再用编写器指向该文件夹。
