# 创作者：从角色包到 oclive

**全库文档索引**：[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)  
**插件架构、HTTP 侧车、更新策略（完整版）**：[../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)

## 双软件分工（运行时 vs 编写器）

- **运行时（oclivenewnew 本仓库）**：加载角色包、校验、对话与持久化。  
- **角色包编写器（独立仓库）**：仅在本地编辑并导出 **`roles/{角色id}/`** 树或 zip（`.ocpak` 与 `.zip` 均为 zip）；**不**嵌入运行时源码。  
- **唯一接口**：磁盘上的包结构；契约以本仓库 **`creator-docs/`** 与 **`roles/README_MANIFEST.md`** 为准，编写器 README 链到此处即可。

**在 oclive 中安装包**：除把目录放进 `roles/` 或设置 **`OCLIVE_ROLES_DIR`** 外，可在应用内 **导入 `.ocpak`、`.zip`（与 `.ocpak` 同为 ZIP）或已解压的包目录**（结构须与 `roles/{角色id}/` 一致）。详见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) 中「在 oclive 中导入角色包」。

建议将编写器 checkout 为与本仓库**同级**目录（例如 `D:\oclive-pack-editor` 与 `D:\oclivenewnew`），在 Cursor / VS Code 中用 **`oclive-pack-editor.code-workspace`** 多根联开两项目。

## 目录布局

每个角色一个文件夹：**`roles/{角色id}/`**，与文件夹名同名的 **`manifest.json`** 为门面；可选 **`settings.json`** 覆盖引擎字段。约定见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md)。

**`OCLIVE_ROLES_DIR`**：指向 **roles 根**（即直接包含各 `角色id` 子文件夹的那一层）。编写器导出 zip 并解压、或「写入文件夹」时，所选目录即应对应这一层，使得存在 **`$OCLIVE_ROLES_DIR/<角色id>/manifest.json`**。

## 编写方式（当前）

1. 复制示例包（如 `roles/mumu/`）或 [manifest 模板](../roles/manifest.template.json)。
2. 编辑 `manifest.json` / `settings.json` / `core_personality.txt` 与场景资源。
3. 设置环境变量 **`OCLIVE_ROLES_DIR`** 指向含 `roles/` 的父目录，或把包放在项目/应用资源约定的 `roles/` 下。
4. 启动应用，**加载角色**后对话验证。

若角色包含 **`knowledge/`** 世界观 Markdown（约定与字段说明见 [WORLDVIEW_KNOWLEDGE.md](../role-pack/WORLDVIEW_KNOWLEDGE.md)）：编辑或替换其中 `*.md`、或调整 manifest / `settings.json` 中与知识相关的开关后，须再次调用 **`load_role`**（或切换角色等会重新加载包的路径），对话编排才会使用磁盘上的最新知识索引；`get_role_info` 中的 **`knowledge_enabled` / `knowledge_chunk_count`** 可用来确认当前已加载的索引摘要。

## 校验

- 加载路径会执行 **manifest 校验**；错误见日志或界面提示。
- 插件后端见 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)。记忆 / 情绪 / 事件 / Prompt 除 `builtin` 与 `remote` 外，宿主还提供 **`builtin_v2` 第二套内置实现**（用于可替换性验证与保守策略），详见该文档各枚举表。
- 自建 HTTP 侧车、环境变量与「本地 / 线上」更新边界见 [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)。

## 编写器（独立仓库）

独立应用 **`oclive-pack-editor`**（与运行时**不同** `package.json` / 仓库）。导出后仍以 **`load_role`** 为最终校验；编写器侧轻量检查与中长期 **crate/CLI** 路线见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)。产品里程碑仍见 [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md)。
