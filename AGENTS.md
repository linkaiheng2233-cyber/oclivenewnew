# Agent / AI 协作说明（oclivenewnew）

本仓库为 **Tauri + Vue 3 + Rust** 桌面角色对话应用。自动化助手或外部 Agent 在修改代码前，请先阅读：

- **项目约束**：根目录 [`.cursor/rules/oclivenewnew.mdc`](.cursor/rules/oclivenewnew.mdc)（编排、持久化、Tauri 命令注册、DTO、Prompt 约定）。
- **创作者与架构文档**：[`creator-docs/README.md`](creator-docs/README.md) → [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- **愿景与路线**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)、[`creator-docs/roadmap/VISION_OPEN_LAB.md`](creator-docs/roadmap/VISION_OPEN_LAB.md)（开放实验场摘要）。

**契约优先**：角色包 `manifest.json` / `settings.json` 键与行为以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及校验 crate 为准；新增顶层键需同步 `crates/oclive_validation` 与文档。

**姊妹仓库**（同级目录常见）：`oclive-pack-editor`（角色包编写器）、`oclive-launcher`（启动器）、`oclive-plugin-market`（市场站）。各仓可有各自的 `AGENTS.md`，指向本仓文档索引即可。

**开发机磁盘**：本仓库根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **Cargo `target-dir`** 指到仓库外的 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`，与源码分离；发版安装包体积与此无关。姊妹仓 **oclive-pack-editor**、**oclive-launcher** 使用同级目录下的 `oclive-pack-editor-cargo-target/`、`oclive-launcher-cargo-target/`（各仓自有 `.cargo/config.toml`）。旧版留在仓库内的 `target/`、`src-tauri/target/` 可整夹删除。

### 前端：插件管理入口与 Tauri `invoke`

- **V1 / V2 路由**：`uiStore.experimentalPluginManagerV2`（Pinia 持久化）为唯一开关；顶栏「更多」与 **Ctrl+Shift+F** 的打开逻辑集中在 [`src/composables/usePluginManagerWindow.ts`](src/composables/usePluginManagerWindow.ts)。设置页与快捷键说明中的**用户可见文案**集中在 [`src/lib/pluginManagerEntryCopy.ts`](src/lib/pluginManagerEntryCopy.ts)，避免多处硬编码漂移（设置里需 `v-html` 的段落仅输出静态 HTML，勿拼接用户输入）。
- **V1 已安装区 UI**：侧栏 + 右侧「单插件配置 + 调试台」抽为 [`src/components/InstalledPluginWorkspaceDetail.vue`](src/components/InstalledPluginWorkspaceDetail.vue)，由 [`src/views/PluginManagerPanel.vue`](src/views/PluginManagerPanel.vue) 引用。
- **`invoke` 参数名**：Tauri 将 Rust 命令的 `snake_case` 形参映射为前端的 **camelCase** 键（如 `plugin_id` → `pluginId`）。[`src/utils/tauri-api.ts`](src/utils/tauri-api.ts) 中 `get_plugin_logs`、`spawn_plugin_for_test` 等须与之一致；若命令仍手写 `snake_case` 载荷，会出现「missing required key `pluginId`」类错误。
