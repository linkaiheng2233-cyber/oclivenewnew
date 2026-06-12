# 创作者：从角色包到 oclive

**全库文档索引**：[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)  
**插件架构、HTTP 侧车、更新策略（完整版）**：[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)

## 双应用分工（编写器 + 运行时）

- **运行时（oclivenewnew 本仓库 · A.I.Live）**：加载角色包、校验、对话与持久化；**专家路由**、架构图 **`groups`**、蓝图 **`includes[]`** 在此配置。  
- **角色包编写器（[oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)）**：编辑人设、`pipeline.ocblueprint` 的 `meta` / `slot_registry`、场景与知识、导出 zip；保存时**保留**主应用写入的蓝图扩展字段。  
- **唯一接口**：磁盘上的 **`roles/{角色id}/`** 包结构；契约以本仓库 **`creator-docs/`** 与 **`roles/README_MANIFEST.md`** 为准。  
- **已退役**：**oclive-launcher**（归档）；长期统一入口见 [oclive-studio](https://github.com/linkaiheng2233-cyber/oclive-studio)，当前发版与文档以 **编写器 + 本运行时** 为准。

**在 oclive 中安装包**：除把目录放进 `roles/` 或设置 **`OCLIVE_ROLES_DIR`** 外，可在应用内 **导入 `.ocpak`、`.zip`（与 `.ocpak` 同为 ZIP）或已解压的包目录**（结构须与 `roles/{角色id}/` 一致）。详见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) 中「在 oclive 中导入角色包」。

**使用工作室（推荐）**：[oclive-studio](https://github.com/linkaiheng2233-cyber/oclive-studio) 在启动模式中提供 **从 zip 安装角色包** 到 `OCLIVE_ROLES_DIR`、**Ollama / Remote LLM** 配置（注入 **`OCLIVE_LLM_BACKEND`** 与 **`OCLIVE_REMOTE_*`**，运行时**覆盖**角色包中的 `plugin_backends.llm`，见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）及一键拉起 **`oclivenewnew --api`** 供创作模式试聊。配置权威文件为 **`studio-config.json`**（见 [工作室用户指南](../studio/USER_GUIDE.md)）。协议见 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)。

建议将 **oclive-studio** 与本仓库**同级**克隆（例如 `D:\oclivenewnew` 与 `D:\oclive-studio`）。

## 目录布局

每个角色一个文件夹 **`roles/{角色id}/`**。**v2 SSOT** 为 **`pipeline.ocblueprint`**（`meta` + `slot_registry` + 可选 `groups`）。**v1（已废弃）** 的 `manifest.json` / `settings.json` 仅作迁移，见 [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md)。规范：[ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)。

**`OCLIVE_ROLES_DIR`**：指向 **roles 根**。v2 包须存在 **`$OCLIVE_ROLES_DIR/<角色id>/pipeline.ocblueprint`**。

## 编写方式（当前 · v2）

1. 复制 v2 示例包（如 `roles/mumu/`）或 `oclive pack create` / 工作室创作模式导出。
2. 编辑 **`pipeline.ocblueprint`**、`core_personality.txt`、场景与可选资源。  
   - **`meta.personality`**：七维；**`meta.evolution.personality_source`** 为 **`profile`** 时可变档案仅存 DB。详见 [personality-archive-notes.md](../../docs/personality-archive-notes.md)。
3. 设置环境变量 **`OCLIVE_ROLES_DIR`** 指向含 `roles/` 的父目录，或把包放在项目/应用资源约定的 `roles/` 下。
4. 启动应用，**加载角色**后对话验证。

若角色包含 **`knowledge/`** 世界观 Markdown（见 [WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)）：编辑 `*.md` 或调整 **`meta.knowledge`** 后，须再次 **`load_role`**（或切换角色等会重新加载包的路径），对话编排才会使用磁盘上的最新知识索引；`get_role_info` 中的 **`knowledge_enabled` / `knowledge_chunk_count`** 可用来确认当前已加载的索引摘要。

## 校验

- v2 包由 **`oclive_validation`** 校验 `pipeline.ocblueprint`（含 `groups`）；`oclive pack validate` / 加载角色时报错见界面提示。
- 插件后端见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)。
- **主应用**：**Ctrl+Shift+F** 仅为「已安装插件」列表；`ui_slots` 插件启用时弹出位置选择。**无**架构图面板。
- **CLI**：manifest **`slot_attachment`** + **`oclive plugin install <id> --role roles/<pack>`** 自动写入蓝图；多槽位 / 架构总览：**`oclive plugin manage`**（`--tui` 可选）。记忆 / 情绪 / 事件 / Prompt 槽 **`builtin_v2` 为已废弃 wire alias**（serde 读入等同 `builtin`），详见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 各枚举表。
- 自建 HTTP 侧车、环境变量与「本地 / 线上」更新边界见 [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)。

## 创作模式（工作室内置）

工作室 **创作模式**（原 `oclive-pack-editor` 前端迁入 `oclive-studio/src/create/`）与运行时**不同**进程。导出后仍以 **`load_role`** 为最终校验；轻量检查与中长期 **crate/CLI** 路线见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)。合并说明见 [RFC_STUDIO_MERGE.md](../rfc/RFC_STUDIO_MERGE.md)。产品里程碑仍见 [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)。
