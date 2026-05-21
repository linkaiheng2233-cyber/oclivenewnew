# 创作者：从角色包到 oclive

**全库文档索引**：[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)  
**插件架构、HTTP 侧车、更新策略（完整版）**：[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)

## 双应用分工（运行时 + 工作室）

- **运行时（oclivenewnew 本仓库）**：加载角色包、校验、对话与持久化。  
- **oclive 工作室（[oclive-studio](https://github.com/oclive-app/oclive-studio)）**：统一创作者入口——**启动模式**（配置 `roles` 根、Ollama/Remote LLM、拉起运行时）与 **创作模式**（编辑并导出 **`roles/{角色id}/`** 树或 zip；`.ocpak` 与 `.zip` 均为 zip）。原独立仓库 **oclive-launcher**、**oclive-pack-editor** 已废弃，仅作归档。  
- **唯一接口**：磁盘上的包结构；契约以本仓库 **`creator-docs/`** 与 **`roles/README_MANIFEST.md`** 为准，工作室 README 链到此处即可。

**在 oclive 中安装包**：除把目录放进 `roles/` 或设置 **`OCLIVE_ROLES_DIR`** 外，可在应用内 **导入 `.ocpak`、`.zip`（与 `.ocpak` 同为 ZIP）或已解压的包目录**（结构须与 `roles/{角色id}/` 一致）。详见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) 中「在 oclive 中导入角色包」。

**使用工作室（推荐）**：[oclive-studio](https://github.com/oclive-app/oclive-studio) 在启动模式中提供 **从 zip 安装角色包** 到 `OCLIVE_ROLES_DIR`、**Ollama / Remote LLM** 配置（注入 **`OCLIVE_LLM_BACKEND`** 与 **`OCLIVE_REMOTE_*`**，运行时**覆盖**角色包中的 `plugin_backends.llm`，见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)）及一键拉起 **`oclivenewnew --api`** 供创作模式试聊。协议见 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)。

建议将 **oclive-studio** 与本仓库**同级**克隆（例如 `D:\oclivenewnew` 与 `D:\oclive-studio`）。

## 目录布局

每个角色一个文件夹：**`roles/{角色id}/`**，与文件夹名同名的 **`manifest.json`** 为门面；可选 **`settings.json`** 覆盖引擎字段。约定见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)。

**`OCLIVE_ROLES_DIR`**：指向 **roles 根**（即直接包含各 `角色id` 子文件夹的那一层）。编写器导出 zip 并解压、或「写入文件夹」时，所选目录即应对应这一层，使得存在 **`$OCLIVE_ROLES_DIR/<角色id>/manifest.json`**。

## 编写方式（当前）

1. 复制示例包（如 `roles/mumu/`）或 [manifest 模板](../roles/manifest.template.json)。
2. 编辑 `manifest.json` / `settings.json` / `core_personality.txt` 与场景资源。  
   - **`core_personality.txt`** 即包内 **核心性格档案**（运行时不可由模型改写）。若 `settings.json` 里 **`evolution.personality_source`** 为 **`profile`**，对话后的 **可变性格档案**仅存运行时数据库、由模型维护，包内不可手写，只能通过 `evolution`（如 `max_change_per_event`）调强弱；**七维**在该模式下多为视图，仍建议填写。详见 **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)** 与 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)。
3. 设置环境变量 **`OCLIVE_ROLES_DIR`** 指向含 `roles/` 的父目录，或把包放在项目/应用资源约定的 `roles/` 下。
4. 启动应用，**加载角色**后对话验证。

若角色包含 **`knowledge/`** 世界观 Markdown（约定与字段说明见 [WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)）：编辑或替换其中 `*.md`、或调整 manifest / `settings.json` 中与知识相关的开关后，须再次调用 **`load_role`**（或切换角色等会重新加载包的路径），对话编排才会使用磁盘上的最新知识索引；`get_role_info` 中的 **`knowledge_enabled` / `knowledge_chunk_count`** 可用来确认当前已加载的索引摘要。

## 校验

- 加载路径会执行 **manifest 校验**；错误见日志或界面提示。
- 插件后端见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)。记忆 / 情绪 / 事件 / Prompt 除 `builtin` 与 `remote` 外，宿主还提供 **`builtin_v2` 第二套内置实现**（用于可替换性验证与保守策略），详见该文档各枚举表。
- 自建 HTTP 侧车、环境变量与「本地 / 线上」更新边界见 [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)。

## 创作模式（工作室内置）

工作室 **创作模式**（原 `oclive-pack-editor` 前端迁入 `oclive-studio/src/create/`）与运行时**不同**进程。导出后仍以 **`load_role`** 为最终校验；轻量检查与中长期 **crate/CLI** 路线见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)。合并说明见 [RFC_STUDIO_MERGE.md](../rfc/RFC_STUDIO_MERGE.md)。产品里程碑仍见 [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)。
